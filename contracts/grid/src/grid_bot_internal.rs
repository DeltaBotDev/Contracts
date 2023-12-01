use std::ops::{Add, Div, Mul, Sub};
use crate::*;
use near_sdk::{env, Gas, Promise};
use near_sdk::json_types::U128;
use serde_json::json;
use crate::{GridBotContract, SLIPPAGE_DENOMINATOR, U256C};
use crate::big_decimal::BigDecimal;
use crate::entity::GridType;
use crate::entity::GridType::EqOffset;
use crate::events::emit;

impl GridBotContract {
    // pub fn internal_transfer_token(&self, receiver_id: AccountId, amount: Balance, token_contract_id: AccountId) {
    //     Promise::new(token_contract_id)
    //         .function_call(
    //             "ft_transfer".to_string(),
    //             json!({
    //                 "receiver_id": receiver_id,
    //                 "amount": amount
    //             }).to_string().into_bytes(),
    //             1_000_000_000_000_000_000_000, // 需要附加的GAS量
    //             0, // 附加的NEAR数量，如果是转账 NEP-141 代币，则为 0
    //         );
    // }

    pub fn internal_get_next_bot_id(&self) -> u128 {
        return self.next_bot_id;
    }

    pub fn internal_get_and_use_next_bot_id(&mut self) -> u128 {
        let next_id = self.next_bot_id;

        assert_ne!(self.next_bot_id.checked_add(1), None, "VALID_NEXT_BOT_ID");

        self.next_bot_id += 1;

        return next_id;
    }

    pub fn internal_get_user_balance(&self, user: &AccountId, token: &AccountId) -> U128C {
        return self.user_balances_map.get(user)
            .and_then(|balances| balances.get(token).cloned())
            .unwrap_or(U128C::from(0));
    }

    // pub fn internal_get_and_use_next_pair_id(&mut self) -> u128 {
    //     let next_id = self.next_pair_id;
    //
    //     assert_ne!(self.next_pair_id.checked_add(1), None, "VALID_NEXT_PAIR_ID");
    //
    //     self.next_pair_id += 1;
    //
    //     return next_id;
    // }

    pub fn internal_init_bot_status(&self, bot: &mut GridBot, entry_price: U128C) {
        if bot.trigger_price == U128C::from(0) {
            bot.active = true;
            return;
        }
        if entry_price >= bot.trigger_price {
            bot.trigger_price_above_or_below = false;
        } else {
            bot.trigger_price_above_or_below = true;
        }
    }

    pub fn internal_get_oracle_price(&self, pair_id: String) -> U128C {
        if !self.oracle_price_map.contains_key(&pair_id) {
            return U128C::from(0);
        }
        let price_info = self.oracle_price_map.get(&pair_id).unwrap();
        if price_info.valid_timestamp < env::block_timestamp() {
            // oracle price expired
            return U128C::from(0);
        }
        return price_info.price;
    }

    pub fn internal_check_oracle_price(&self, entry_price: U128C, pair_id: String, slippage: u16) -> bool {
        if !self.oracle_price_map.contains_key(&pair_id) {
            return false;
        }
        let price_info = self.oracle_price_map.get(&pair_id).unwrap();
        if price_info.valid_timestamp < env::block_timestamp() {
            // oracle price expired
            return false;
        }

        let recorded_price = price_info.price;
        if entry_price >= recorded_price {
            return (entry_price - recorded_price) / entry_price * SLIPPAGE_DENOMINATOR <= U128C::from(slippage);
        } else {
            return (recorded_price - entry_price) / entry_price * SLIPPAGE_DENOMINATOR <= U128C::from(slippage);
        }
    }

    pub fn internal_check_bot_close_permission(&self, user: &AccountId, bot: &GridBot) -> bool {
        if user == &(bot.user) {
            return true;
        }
        let oracle_price = self.internal_get_oracle_price(bot.pair_id.clone());
        if oracle_price >= bot.take_profit_price {
            return true;
        }
        if oracle_price <= bot.stop_loss_price {
            return true;
        }
        return false;
    }

    pub fn internal_get_first_forward_order(grid_bot: GridBot, pair: Pair, level: usize) -> Order {
        let mut order = Order{
            order_id: level.to_string(),
            token_sell: pair.base_token.clone(),
            token_buy: pair.quote_token.clone(),
            amount_sell: U128C::from(0),
            amount_buy: U128C::from(0),
            fill_buy_or_sell: false,
            filled: U128C::from(0),
        };
        let grid_rate_denominator_128 = U128C::from(GRID_RATE_DENOMINATOR);
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
                    grid_bot.first_quote_amount.clone() + grid_bot.grid_offset * U128C::from(level.clone() as u16)
                } else {
                    // proportional grid
                    grid_bot.first_quote_amount.clone() * (grid_rate_denominator_128 + U128C::from(grid_bot.grid_rate)).pow(U128C::from(level.clone() as u16)) / grid_rate_denominator_128.pow(U128C::from(level.clone() as u16))
                };
            } else {
                // fixed quote
                order.amount_sell = grid_bot.first_quote_amount.clone();
                order.amount_buy = if grid_bot.grid_type == EqOffset {
                    // arithmetic grid
                    grid_bot.first_base_amount.clone() - grid_bot.grid_offset * U128C::from(level.clone() as u16)
                } else {
                    // proportional grid
                    grid_bot.first_base_amount.clone() * (grid_rate_denominator_128 - U128C::from(grid_bot.grid_rate)).pow(U128C::from(level.clone() as u16)) / grid_rate_denominator_128.pow(U128C::from(level.clone() as u16))
                };
            }
        } else {
            // sell grid
            order.token_sell = pair.base_token.clone();
            order.token_buy = pair.quote_token.clone();
            order.fill_buy_or_sell = !grid_bot.fill_base_or_quote.clone();
            let coefficient = U128C::from(grid_bot.grid_buy_count.clone() + grid_bot.grid_sell_count.clone() - 1 - level.clone() as u16);
            if grid_bot.fill_base_or_quote {
                // fixed base
                order.amount_sell = grid_bot.last_base_amount.clone();
                order.amount_buy = if grid_bot.grid_type == EqOffset {
                    grid_bot.last_quote_amount.clone() - grid_bot.grid_offset * coefficient.clone()
                } else {
                    grid_bot.last_quote_amount.clone() * (grid_rate_denominator_128 - U128C::from(grid_bot.grid_rate)).pow(coefficient.clone()) / grid_rate_denominator_128.pow(coefficient.clone())
                };
            } else {
                // fixed quote
                order.amount_buy = grid_bot.last_quote_amount.clone();
                order.amount_sell = if grid_bot.grid_type == EqOffset {
                    grid_bot.last_base_amount.clone() + grid_bot.grid_offset * coefficient.clone()
                } else {
                    grid_bot.last_base_amount.clone() * (grid_rate_denominator_128 + U128C::from(grid_bot.grid_rate)).pow(coefficient.clone()) / grid_rate_denominator_128.pow(coefficient.clone())
                };
            }
        }
        return order;
    }

    pub fn internal_calculate_bot_assets(first_quote_amount: U128C, last_base_amount: U128C, grid_sell_count: u16, grid_buy_count: u16,
                                         grid_type: GridType, grid_rate: u16, grid_offset: U128C, fill_base_or_quote: bool) -> (U128C, U128C) {
        // calculate quote
        let grid_buy_count_u128 = U128C::from(grid_buy_count);
        let quote_amount_buy = if grid_buy_count == 0 {
            U128C::from(0)
        } else if fill_base_or_quote {
            if grid_type == EqOffset {
                first_quote_amount * grid_buy_count_u128.clone() + grid_offset * (grid_buy_count_u128.clone() - U128C::from(1)) * grid_buy_count_u128.clone() / U128C::from(2)
            } else {
                let geometric_series_sum = GridBotContract::private_calculate_rate_bot_geometric_series_sum(grid_buy_count.clone() as u64, grid_rate.clone() as u64);
                U128C::from(BigDecimal::from(first_quote_amount.clone().as_u128()).mul(geometric_series_sum).div(BigDecimal::from(GRID_RATE_DENOMINATOR as u64)).round_down_u128())
            }
        } else {
            first_quote_amount * grid_buy_count_u128.clone()
        };

        // calculate base
        let grid_sell_count_u128 = U128C::from(grid_sell_count);
        let base_amount_sell = if grid_sell_count == 0 {
            U128C::from(0)
        } else if fill_base_or_quote {
            last_base_amount * grid_sell_count_u128.clone()
        } else {
            if grid_type == EqOffset {
                last_base_amount * grid_sell_count_u128.clone() + grid_offset * (grid_sell_count_u128.clone() - U128C::from(1)) * grid_sell_count_u128.clone() / U128C::from(2)
            } else {
                let geometric_series_sum = GridBotContract::private_calculate_rate_bot_geometric_series_sum(grid_sell_count.clone() as u64, grid_rate.clone() as u64);
                U128C::from(BigDecimal::from(last_base_amount.clone().as_u128()).mul(geometric_series_sum).div(BigDecimal::from(GRID_RATE_DENOMINATOR as u64)).round_down_u128())
            }
        };
        return (base_amount_sell, quote_amount_buy);
    }

    pub fn internal_get_pair_key(base_token: AccountId, quote_token: AccountId) -> String {
        return format!("{}:{}", base_token.clone().to_string(), quote_token.clone().to_string());
    }

    pub fn internal_transfer_assets_to_lock(&mut self, user: AccountId, token: AccountId, amount: U128C) {
        if amount == U128C::from(0) {
            return;
        }
        let user_balances = self.user_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        let balance = user_balances.entry(token.clone()).or_insert(U128C::from(0));
        *balance -= amount;

        let user_locked_balances = self.user_locked_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        let locked_balance = user_locked_balances.entry(token.clone()).or_insert(U128C::from(0));
        *locked_balance += amount;
    }

    pub fn internal_transfer_assets_to_unlock(&mut self, user: &AccountId, token: &AccountId, amount: Balance) {
        if amount == 0 {
            return;
        }
        let user_balances = self.user_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        let balance = user_balances.entry(token.clone()).or_insert(U128C::from(0));
        *balance += U128C::from(amount.clone());

        let user_locked_balances = self.user_locked_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        let locked_balance = user_locked_balances.entry(token.clone()).or_insert(U128C::from(0));
        *locked_balance -= U128C::from(amount);
    }

    pub fn internal_reduce_asset(&mut self, user: &AccountId, token: &AccountId, amount: Balance) {
        if amount == 0 {
            return;
        }
        let user_balances = self.user_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        let balance = user_balances.entry(token.clone()).or_insert(U128C::from(0));
        *balance -= U128C::from(amount);
    }

    pub fn internal_increase_asset(&mut self, user: &AccountId, token: &AccountId, amount: Balance) {
        if amount == 0 {
            return;
        }
        let user_balances = self.user_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        let balance = user_balances.entry(token.clone()).or_insert(U128C::from(0));
        *balance += U128C::from(amount);
    }

    pub fn internal_remove_revenue_from_bot(&mut self, bot: &GridBot) {
        if bot.fill_base_or_quote {
            self.bot_map.get_mut(&(bot.bot_id)).unwrap().total_base_amount -= bot.revenue.clone();
        } else {
            self.bot_map.get_mut(&(bot.bot_id)).unwrap().total_quote_amount -= bot.revenue.clone();
        }
    }

    pub fn internal_harvest_revenue(&mut self, bot: &GridBot, pair: &Pair, user: &AccountId) -> (AccountId, Balance) {
        let revenue_token = if bot.fill_base_or_quote {
            pair.base_token.clone()
        } else {
            pair.quote_token.clone()
        };
        let revenue = bot.revenue.clone();
        // transfer out from bot asset
        self.internal_remove_revenue_from_bot(&bot);
        // transfer to available asset
        self.internal_increase_asset(&user, &revenue_token, revenue.clone());
        // sign to 0
        self.bot_map.get_mut(&(bot.bot_id)).unwrap().revenue = 0;
        return (revenue_token, revenue);
    }

    pub fn internal_increase_global_asset(&mut self, token: &AccountId, amount: &U128C) {
        let balance = self.token_map.get_mut(token).unwrap();
        *balance += *amount;
    }

    pub fn internal_reduce_global_asset(&mut self, token: &AccountId, amount: &U128C) {
        let balance = self.token_map.get_mut(token).unwrap();
        *balance -= *amount;
    }

    pub fn internal_withdraw(&mut self, user: &AccountId, token: &AccountId, amount: Balance) {
        self.internal_ft_transfer(&user, &token, amount.clone());
        emit::withdraw_started(&user, amount.clone(), &token);
    }

    fn private_calculate_rate_bot_geometric_series_sum(n: u64, delta_r: u64) -> BigDecimal {
        let scale = BigDecimal::from(GRID_RATE_DENOMINATOR as u64);
        let a = scale;   // 1.0 * scale
        let r = BigDecimal::from(delta_r).add(BigDecimal::from(GRID_RATE_DENOMINATOR as u64));
        let sum = a.mul(scale.sub(r.pow(n))).div(scale.sub(r));
        return sum;
    }
}
