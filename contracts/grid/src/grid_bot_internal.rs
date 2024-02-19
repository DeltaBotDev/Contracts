use std::ops::{Add, Div, Mul, Sub};
use crate::*;
use near_sdk::{env, require};
use near_sdk::json_types::U128;
use uint::hex;
use crate::{GridBotContract};
use crate::big_decimal::BigDecimal;
use crate::entity::GridType;
use crate::entity::GridType::EqOffset;
use crate::events::emit;
use crate::oracle::{Price, PriceIdentifier};

impl GridBotContract {

    pub fn internal_create_bot(&mut self,
                               base_price: Price,
                               quote_price: Price,
                               user: &AccountId,
                               slippage: u16,
                               entry_price: &U256C,
                               pair: &Pair,
                               grid_bot: &mut GridBot) -> bool {
        if self.status != GridStatus::Running {
            self.internal_create_bot_refund_with_near(&user, &pair, STORAGE_FEE, PAUSE_OR_SHUTDOWN);
            return false;
        }
        if !self.internal_check_oracle_price(*entry_price, base_price.clone(), quote_price.clone(), slippage) {
            self.internal_create_bot_refund_with_near(&user, &pair, STORAGE_FEE, INVALID_PRICE);
            return false;
        }
        // check balance
        if self.internal_get_user_balance(user, &(pair.base_token)) < grid_bot.total_base_amount {
            self.internal_create_bot_refund_with_near(&user, &pair, STORAGE_FEE, LESS_BASE_TOKEN);
            return false;
        }
        if self.internal_get_user_balance(user, &(pair.quote_token)) < grid_bot.total_quote_amount {
            self.internal_create_bot_refund_with_near(&user, &pair, STORAGE_FEE, LESS_QUOTE_TOKEN);
            return false;
        }

        // create bot id
        let next_bot_id = format!("GRID:{}", self.internal_get_and_use_next_bot_id().to_string());
        grid_bot.bot_id = next_bot_id;

        // initial orders space, create empty orders
        let grid_count = grid_bot.grid_sell_count.clone() + grid_bot.grid_buy_count.clone();
        self.create_default_orders(grid_bot.bot_id.clone(), grid_count);

        // transfer assets
        self.internal_transfer_assets_to_lock(&user, &pair.base_token, grid_bot.total_base_amount);
        self.internal_transfer_assets_to_lock(&user, &pair.quote_token, grid_bot.total_quote_amount);

        // init active status of bot
        self.internal_init_bot_status(grid_bot, entry_price);

        // insert bot
        self.bot_map.insert(&(grid_bot.bot_id), &grid_bot);

        emit::create_bot(&grid_bot.user, grid_bot.bot_id.clone(), base_price.price.0.to_string(), quote_price.price.0.to_string(), base_price.expo.to_string(), quote_price.expo.to_string());
        return true;
    }

    pub fn internal_take_orders(&mut self, user: &AccountId, take_order: &Order, maker_orders: Vec<OrderKeyInfo>) -> (U256C, U256C) {
        require!(self.status == GridStatus::Running, PAUSE_OR_SHUTDOWN);
        require!(maker_orders.len() > 0, INVALID_MAKER_ORDERS);
        require!(take_order.amount_sell != U256C::from(0), INVALID_ORDER_AMOUNT);
        require!(take_order.amount_buy != U256C::from(0), INVALID_ORDER_AMOUNT);
        require!(self.internal_get_user_balance(&user, &(take_order.token_sell)) >= take_order.amount_sell, LESS_TOKEN_SELL);
        let mut took_amount_sell = U256C::from(0);
        let mut took_amount_buy = U256C::from(0);
        let mut total_took_fee = U256C::from(0);
        // loop take order
        for maker_order in maker_orders.iter() {
            if take_order.amount_sell.as_u128() == took_amount_sell.as_u128() {
                // over
                break;
            }
            let (taker_sell, taker_buy, maker, maker_fee, current_revenue, maker_left_revenue, maker_total_revenue) = self.internal_take_order(maker_order.bot_id.clone(), maker_order.forward_or_reverse.clone(), maker_order.level.clone(), &take_order, took_amount_sell.clone(), took_amount_buy.clone());
            // calculate taker fee
            let (real_taker_buy, taker_fee) = self.internal_calculate_taker_fee(taker_buy);
            took_amount_sell += taker_sell;
            took_amount_buy += real_taker_buy;
            total_took_fee += taker_fee;
            // send event
            emit::take_order(user, &maker, maker_order.bot_id.clone(), maker_order.forward_or_reverse.clone(), maker_order.level.clone(), &taker_sell, &taker_buy, &maker_fee, &taker_fee, &current_revenue, &maker_left_revenue, &maker_total_revenue);
        }
        require!(take_order.amount_sell >= took_amount_sell, INVALID_ORDER_MATCHING);

        // transfer taker's asset
        self.internal_reduce_asset(&user, &(take_order.token_sell), &took_amount_sell);
        self.internal_increase_asset(&user, &(take_order.token_buy), &took_amount_buy);
        // add protocol fee
        self.internal_increase_protocol_fee(&(take_order.token_buy), &(total_took_fee));

        // log!("Success take orders, sell token:{}, buy token:{}, sell amount:{}, buy amount:{}", take_order.token_sell, take_order.token_buy, take_order.amount_sell, take_order.amount_buy);
        return (took_amount_sell, took_amount_buy);
    }

    pub fn internal_close_bot(&mut self, user: &AccountId, bot_id: &String, bot: &mut GridBot, pair: &Pair) {
        // sign closed
        bot.closed = true;

        // harvest revenue, must fist execute, will split revenue from bot's asset
        let (revenue_token, revenue) = self.internal_harvest_revenue(bot, pair);
        // unlock token
        self.internal_transfer_assets_to_unlock(&(bot.user), &(pair.base_token), bot.total_base_amount.clone());
        self.internal_transfer_assets_to_unlock(&(bot.user), &(pair.quote_token), bot.total_quote_amount.clone());

        // withdraw token
        self.internal_withdraw(&(bot.user), &(pair.base_token), bot.total_base_amount);
        self.internal_withdraw(&(bot.user), &(pair.quote_token), bot.total_quote_amount);
        self.internal_withdraw(&(bot.user), &revenue_token, revenue);
        self.bot_map.insert(bot_id, &bot);

        // send claim event
        if revenue.as_u128() > 0 {
            // claim event
            emit::claim(&user, &(bot.user), bot_id.clone(), &revenue_token, revenue);
        }
        emit::close_bot(user, bot_id.clone());
    }

    pub fn internal_auto_close_bot(&mut self, base_price: Price, quote_price: Price, user: &AccountId, bot_id: &String, bot: &mut GridBot, pair: &Pair) {
        require!(self.internal_check_bot_close_permission(base_price.clone(), quote_price.clone(), bot), INVALID_PRICE_OR_NO_PERMISSION);
        emit::close_bot_price(base_price.price.0.to_string(), quote_price.price.0.to_string(), base_price.expo.to_string(), quote_price.expo.to_string());
        self.internal_close_bot(user, bot_id, bot, pair);
    }

    pub fn internal_trigger_bot(&mut self, base_price: Price, quote_price: Price, bot_id: &String, bot: &mut GridBot) {
        require!(base_price.publish_time as u64 * 1000 + self.oracle_valid_time.clone() >= env::block_timestamp_ms(), INVALID_PRICE);
        require!(quote_price.publish_time as u64 * 1000 + self.oracle_valid_time.clone() >= env::block_timestamp_ms(), INVALID_PRICE);
        let oracle_pair_price = (BigDecimal::from(base_price.price.0 as u64) / BigDecimal::from(quote_price.price.0 as u64) * BigDecimal::from(PRICE_DENOMINATOR)).round_down_u128();

        if bot.trigger_price_above_or_below.clone() && bot.trigger_price.clone().as_u128() <= oracle_pair_price {
            // self.bot_map.get_mut(&bot_id).unwrap().active = true;
            bot.active = true;
            self.bot_map.insert(&bot_id, &bot);
            emit::trigger_bot(bot_id.clone(), base_price.price.0.to_string(), quote_price.price.0.to_string(), base_price.expo.to_string(), quote_price.expo.to_string());
        } else if !bot.trigger_price_above_or_below.clone() && bot.trigger_price.clone().as_u128() >= oracle_pair_price {
            // self.bot_map.get_mut(&bot_id).unwrap().active = true;
            bot.active = true;
            self.bot_map.insert(&bot_id, &bot);
            emit::trigger_bot(bot_id.clone(), base_price.price.0.to_string(), quote_price.price.0.to_string(), base_price.expo.to_string(), quote_price.expo.to_string());
        } else {
            env::panic_str(CAN_NOT_TRIGGER);
        }
    }

    pub fn internal_get_and_use_next_bot_id(&mut self) -> u128 {
        let next_id = self.next_bot_id.clone();

        require!(self.next_bot_id.checked_add(1) != None, INVALID_NEXT_BOT_ID);

        self.next_bot_id += 1;

        return next_id;
    }

    pub fn internal_init_bot_status(&self, bot: &mut GridBot, entry_price: &U256C) {
        if bot.trigger_price == U256C::from(0) {
            bot.active = true;
            return;
        }
        if entry_price.clone() >= bot.trigger_price {
            bot.trigger_price_above_or_below = false;
        } else {
            bot.trigger_price_above_or_below = true;
        }
    }

    // TODO need check again, and test
    pub fn internal_get_first_forward_order(grid_bot: GridBot, pair: Pair, level: usize) -> Order {
        let mut order = Order{
            token_sell: pair.base_token.clone(),
            token_buy: pair.quote_token.clone(),
            amount_sell: U256C::from(0),
            amount_buy: U256C::from(0),
            fill_buy_or_sell: false,
            filled: U256C::from(0),
        };
        // let grid_rate_denominator_128 = U256C::from(GRID_RATE_DENOMINATOR);
        let grid_rate_denominator_256 = U256C::from(GRID_RATE_DENOMINATOR);
        if grid_bot.grid_buy_count > (level.clone() as u16) {
            // buy grid
            order.token_sell = pair.quote_token.clone();
            order.token_buy = pair.base_token.clone();
            order.fill_buy_or_sell = grid_bot.fill_base_or_quote.clone();
            if grid_bot.fill_base_or_quote {
                // fixed base
                order.amount_buy = grid_bot.first_base_amount.clone();
                order.amount_sell = if grid_bot.grid_type == EqOffset {
                    // arithmetic grid
                    grid_bot.first_quote_amount.clone() + grid_bot.grid_offset * U256C::from(level.clone() as u16)
                } else {
                    // proportional grid
                    // grid_bot.first_quote_amount.clone() * (grid_rate_denominator_128 + U256C::from(grid_bot.grid_rate)).pow(U256C::from(level.clone() as u16)) / grid_rate_denominator_128.pow(U256C::from(level.clone() as u16))
                    grid_bot.first_quote_amount.clone() * (grid_rate_denominator_256 + U256C::from(grid_bot.grid_rate)).pow(U256C::from(level.clone() as u16)) / grid_rate_denominator_256.pow(U256C::from(level.clone() as u16))
                };
            } else {
                // fixed quote
                order.amount_sell = grid_bot.first_quote_amount.clone();
                order.amount_buy = if grid_bot.grid_type == EqOffset {
                    // arithmetic grid
                    grid_bot.first_base_amount.clone() - grid_bot.grid_offset * U256C::from(level.clone() as u16)
                } else {
                    // proportional grid
                    // grid_bot.first_base_amount.clone() * (grid_rate_denominator_128 - U256C::from(grid_bot.grid_rate)).pow(U256C::from(level.clone() as u16)) / grid_rate_denominator_128.pow(U256C::from(level.clone() as u16))
                    grid_bot.first_base_amount.clone() * grid_rate_denominator_256.pow(U256C::from(level.clone() as u16)) / ((grid_rate_denominator_256 + U256C::from(grid_bot.grid_rate)).pow(U256C::from(level.clone() as u16)))
                };
            }
        } else {
            // sell grid
            order.token_sell = pair.base_token.clone();
            order.token_buy = pair.quote_token.clone();
            order.fill_buy_or_sell = !grid_bot.fill_base_or_quote.clone();
            let coefficient = U256C::from(grid_bot.grid_buy_count.clone() + grid_bot.grid_sell_count.clone() - 1 - level.clone() as u16);
            if grid_bot.fill_base_or_quote {
                // fixed base
                order.amount_sell = grid_bot.last_base_amount.clone();
                order.amount_buy = if grid_bot.grid_type == EqOffset {
                    grid_bot.last_quote_amount.clone() - grid_bot.grid_offset * U256C::from(coefficient.clone().as_u128())
                } else {
                    // grid_bot.last_quote_amount.clone() * (grid_rate_denominator_128 - U256C::from(grid_bot.grid_rate)).pow(coefficient.clone()) / grid_rate_denominator_128.pow(coefficient.clone())
                    grid_bot.last_quote_amount.clone() * grid_rate_denominator_256.pow(coefficient.clone()) / ((grid_rate_denominator_256 + U256C::from(grid_bot.grid_rate)).pow(coefficient.clone()))
                };
            } else {
                // fixed quote
                order.amount_buy = grid_bot.last_quote_amount.clone();
                order.amount_sell = if grid_bot.grid_type == EqOffset {
                    grid_bot.last_base_amount.clone() + grid_bot.grid_offset * U256C::from(coefficient.clone().as_u128())
                } else {
                    // grid_bot.last_base_amount.clone() * (grid_rate_denominator_256 + U256C::from(grid_bot.grid_rate)).pow(coefficient.clone()) / grid_rate_denominator_256.pow(coefficient.clone())
                    grid_bot.last_base_amount.clone() * (grid_rate_denominator_256 + U256C::from(grid_bot.grid_rate)).pow(coefficient.clone()) / grid_rate_denominator_256.pow(coefficient.clone())
                };
            }
        }
        return order;
    }

    pub fn internal_calculate_bot_assets(first_quote_amount: U256C, last_base_amount: U256C, grid_sell_count: u16, grid_buy_count: u16,
                                         grid_type: GridType, grid_rate: u16, grid_offset: U256C, fill_base_or_quote: bool) -> (U256C, U256C) {
        // calculate quote
        let grid_buy_count_u256 = U256C::from(grid_buy_count);
        let quote_amount_buy = if grid_buy_count == 0 {
            U256C::from(0)
        } else if fill_base_or_quote {
            if grid_type == EqOffset {
                first_quote_amount * grid_buy_count_u256.clone() + grid_offset * (grid_buy_count_u256.clone() - U256C::from(1)) * grid_buy_count_u256.clone() / U256C::from(2)
            } else {
                // gridRate=0.1, 1.1
                // 1.1^0 + 1.1^1 + 1.1^2 + ... + 1.1^n
                let geometric_series_sum = GridBotContract::private_calculate_rate_bot_geometric_series_sum(grid_buy_count.clone() as u64, grid_rate.clone() as u64);
                U256C::from(BigDecimal::from(first_quote_amount.clone().as_u128()).mul(geometric_series_sum).round_down_u128())
            }
        } else {
            first_quote_amount * grid_buy_count_u256.clone()
        };

        // calculate base
        let grid_sell_count_u256 = U256C::from(grid_sell_count);
        let base_amount_sell = if grid_sell_count == 0 {
            U256C::from(0)
        } else if fill_base_or_quote {
            last_base_amount * grid_sell_count_u256.clone()
        } else {
            if grid_type == EqOffset {
                last_base_amount * grid_sell_count_u256.clone() + grid_offset * (grid_sell_count_u256.clone() - U256C::from(1)) * grid_sell_count_u256.clone() / U256C::from(2)
            } else {
                // let geometric_series_sum = GridBotContract::private_calculate_rate_bot_geometric_series_sum_for_sell(grid_sell_count.clone() as u64, grid_rate.clone() as u64);
                // U256C::from(BigDecimal::from(last_base_amount.clone().as_u128()).mul(geometric_series_sum).round_down_u128())
                let geometric_series_sum = GridBotContract::private_calculate_rate_bot_geometric_series_sum(grid_sell_count.clone() as u64, grid_rate.clone() as u64);
                U256C::from(BigDecimal::from(last_base_amount.clone().as_u128()).mul(geometric_series_sum).round_down_u128())
            }
        };
        return (base_amount_sell, quote_amount_buy);
    }

    // pub fn create_default_orders(bot_id: String, grid_count: u16) -> Vector<Vector<Order>> {
    //     let mut outer_vector = Vector::new(StorageKey::OrdersMainKey(bot_id));
    //     for i in 0..2 {
    //         let mut inner_vector = Vector::new(StorageKey::OrdersSubKey(i as u64));
    //         for _ in 0..grid_count {
    //             inner_vector.push(&Order::default());
    //         }
    //         outer_vector.push(&inner_vector);
    //     }
    //     return outer_vector;
    // }

    pub fn create_default_orders(&mut self, bot_id: String, grid_count: u16) {
        let forward_key = bot_id.clone() + "forward";
        let reverse_key = bot_id.clone() + "reverse";
        let mut order_storage = OrdersStorage {
            forward_orders: Vector::new(forward_key.as_bytes().to_vec()),
            reverse_orders: Vector::new(reverse_key.as_bytes().to_vec()),
        };
        for _ in 0..grid_count {
            order_storage.forward_orders.push(&Order::default());
            order_storage.reverse_orders.push(&Order::default());
        }

        self.order_map.insert(&bot_id, &order_storage);
    }

    pub fn internal_init_token(&mut self, token: AccountId, min_deposit: U128) -> U256C {
        if self.global_balances_map.contains_key(&token) {
            return U256C::from(0);
        }
        self.global_balances_map.insert(&token, &U256C::from(0));
        self.protocol_fee_map.insert(&token, &U256C::from(0));
        self.deposit_limit_map.insert(&token, &U256C::from(min_deposit.0));
        self.internal_storage_deposit(&env::current_account_id(), &token, DEFAULT_TOKEN_STORAGE_FEE);
        return U256C::from(DEFAULT_TOKEN_STORAGE_FEE);
    }

    pub fn internal_format_price_identifier(&self, oracle_id: String) -> PriceIdentifier {
        // 32bytes, hex len = 64
        require!(oracle_id.clone().len() == 64, INVALID_ORACLE_ID);
        let oracle_bytes;
        match hex::decode(oracle_id) {
            Ok(bytes) => oracle_bytes = bytes,
            Err(e) => env::panic_str(INVALID_ORACLE_ID),
        }
        let oracle_bytes_slice = &oracle_bytes[..];
        let mut array = [0u8; 32];
        array.copy_from_slice(oracle_bytes_slice);
        return PriceIdentifier(array);
    }

    pub fn internal_need_wrap_near(&self, user: &AccountId, pair: &Pair, base_amount: U256C, quote_amount: U256C) -> bool {
        if pair.base_token != self.wnear && pair.quote_token != self.wnear {
            return false;
        }
        let wnear_balance = self.internal_get_user_balance(&user, &self.wnear);
        if pair.base_token == self.wnear {
            // query balance
            if wnear_balance >= base_amount {
                return false;
            }
            return true
        } else {
            // query balance
            if wnear_balance >= quote_amount {
                return false;
            }
            return true
        }
    }

    fn private_calculate_rate_bot_geometric_series_sum(n: u64, delta_r: u64) -> BigDecimal {
        let scale = BigDecimal::from(1 as u64);
        let a = scale;   // 1.0 * scale
        let r = BigDecimal::from(delta_r).div(BigDecimal::from(GRID_RATE_DENOMINATOR as u128)).add(BigDecimal::from(1 as u64));
        let sum = a.mul(r.pow(n).sub(scale)).div(r.sub(scale));
        return sum;
    }

    // fn private_calculate_rate_bot_geometric_series_sum_for_sell(n: u64, delta_r: u64) -> BigDecimal {
    //     let scale = BigDecimal::from(1 as u64);
    //     let a = scale;   // 1.0 * scale
    //     let r = BigDecimal::from(1 as u64).div(BigDecimal::from(delta_r).div(BigDecimal::from(GRID_RATE_DENOMINATOR as u128)).add(BigDecimal::from(1 as u64)));
    //     let sum = a.mul(r.pow(n).sub(scale)).div(r.sub(scale));
    //     return sum;
    // }
}
