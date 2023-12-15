use near_sdk::require;
use crate::*;
use std::ops::{Div};
use crate::big_decimal::{BigDecimal};
use crate::entity::GridType::EqOffset;

impl GridBotContract {
    pub fn internal_place_order(&mut self, bot_id: String, order: Order, forward_or_reverse: bool, level: usize) {
        require!(self.bot_map.contains_key(&bot_id), INVALID_BOT_ID_FOR_BOT_MAP);
        require!(self.order_map.contains_key(&bot_id), INVALID_BOT_ID_FOR_ORDER_MAP);
        require!(self.order_map.get(&bot_id).unwrap().len() == ORDER_POSITION_SIZE.clone() as usize, INVALID_ORDER_POSITION_LEN);

        let mut bot_orders = self.order_map.get(&bot_id).unwrap();
        let orders = if forward_or_reverse { &mut bot_orders[FORWARD_ORDERS_INDEX.clone()] } else { &mut bot_orders[REVERSE_ORDERS_INDEX.clone()] };
        GridBotContract::private_place_order(order, orders, level.clone());
        self.order_map.insert(&bot_id, &bot_orders);
    }

    pub fn internal_take_order(&mut self, bot_id: String, forward_or_reverse: bool, level: usize, taker_order: &Order, took_sell: U128C, took_buy: U128C) -> (U128C, U128C) {
        let bot = self.bot_map.get(&bot_id.clone()).unwrap().clone();
        let pair = self.pair_map.get(&bot.pair_id).unwrap().clone();
        let (maker_order, in_orderbook) = self.query_order(bot_id.clone(), forward_or_reverse, level);
        // matching check
        GridBotContract::internal_check_order_match(maker_order.clone(), taker_order.clone());

        // calculate
        let (taker_sell, taker_buy, current_filled, made_order) = GridBotContract::internal_calculate_matching(maker_order.clone(), taker_order.clone(), took_sell, took_buy);

        // place into orderbook
        if !in_orderbook {
            self.internal_place_order(bot_id.clone(), maker_order.clone(), forward_or_reverse.clone(), level.clone());
        }
        // update filled
        let maker_order = self.internal_update_order_filled(bot_id.clone(), forward_or_reverse.clone(), level.clone(), current_filled);

        // place opposite order
        let opposite_order = GridBotContract::internal_get_opposite_order(&made_order, bot.clone(), forward_or_reverse.clone(), level.clone());
        self.internal_place_order(bot_id.clone(), opposite_order.clone(), !forward_or_reverse.clone(), level.clone());

        // calculate bot's revenue
        let (revenue_token, revenue, protocol_fee) = self.internal_calculate_bot_revenue(forward_or_reverse.clone(), maker_order.clone(), opposite_order, current_filled.as_u128());
        // add revenue
        // let bot_mut = self.bot_map.get_mut(&bot_id.clone()).unwrap();
        let mut bot = self.bot_map.get(&bot_id.clone()).unwrap();
        bot.revenue += U128C::from(revenue);
        // update bot asset
        GridBotContract::internal_update_bot_asset(&mut bot, &pair, taker_order.token_buy.clone(), taker_buy.as_u128(), taker_sell.as_u128());

        // bot asset transfer
        self.internal_reduce_locked_assets(&(bot.user), &(taker_order.token_buy), &taker_buy);
        self.internal_increase_locked_assets(&(bot.user), &(taker_order.token_sell), &taker_sell);

        // handle protocol fee
        self.internal_add_protocol_fee(&mut bot, &revenue_token, protocol_fee, &pair);

        // update bot
        self.bot_map.insert(&bot_id, &bot);

        return (taker_sell, taker_buy);
    }

    pub fn internal_update_order_filled(&mut self, bot_id: String, forward_or_reverse: bool, level: usize, current_filled: U128C) -> Order {
        let mut bot_orders = self.order_map.get(&bot_id).unwrap();
        let order;
        {
            let orders = if forward_or_reverse { &mut bot_orders[FORWARD_ORDERS_INDEX.clone()] } else { &mut bot_orders[REVERSE_ORDERS_INDEX.clone()] };
            let tmp_order = orders.get_mut(level).unwrap();
            tmp_order.filled += current_filled;
            order = tmp_order.clone();
        }
        self.order_map.insert(&bot_id, &bot_orders);
        return order.clone();
    }

    pub fn internal_check_order_match(maker_order: Order, taker_order: Order) {
        require!(maker_order.token_buy == taker_order.token_sell, INVALID_ORDER_TOKEN);
        require!(maker_order.token_sell == taker_order.token_buy, INVALID_ORDER_TOKEN);
        require!(taker_order.token_sell != taker_order.token_buy, INVALID_ORDER_TOKEN);
        // TODO can move to outside
        require!(taker_order.amount_sell != U128C::from(0), INVALID_ORDER_AMOUNT);
        require!(taker_order.amount_buy != U128C::from(0), INVALID_ORDER_AMOUNT);

        require!(BigDecimal::from(taker_order.amount_sell.as_u128()).div(BigDecimal::from(taker_order.amount_buy.as_u128())) >= BigDecimal::from(maker_order.amount_buy.as_u128()).div(BigDecimal::from(maker_order.amount_sell.as_u128())), INVALID_PRICE);
    }

    pub fn internal_calculate_matching(maker_order: Order, taker_order: Order, took_sell: U128C, took_buy: U128C) -> (U128C, U128C, U128C, Order) {
        // calculate marker max amount
        let max_fill_sell;
        let max_fill_buy;
        if maker_order.fill_buy_or_sell {
            max_fill_buy = maker_order.amount_buy - maker_order.filled;
            max_fill_sell = U128C::from((U256C::from(maker_order.amount_sell.as_u128()) * U256C::from(max_fill_buy.as_u128()) / U256C::from(maker_order.amount_buy.as_u128())).as_u128());
        } else {
            max_fill_sell = maker_order.amount_sell - maker_order.filled;
            max_fill_buy = U128C::from((U256C::from(maker_order.amount_buy.as_u128()) * U256C::from(max_fill_sell.as_u128()) / U256C::from(maker_order.amount_sell.as_u128())).as_u128());
        }
        // calculate matching amount
        let taker_sell;
        let taker_buy;
        if taker_order.fill_buy_or_sell {
            let max_taker_buy = taker_order.amount_buy - took_buy;
            if max_taker_buy >= max_fill_sell {
                // taker all maker
                taker_buy = max_fill_sell;
                taker_sell = max_fill_buy;
            } else {
                taker_buy = max_taker_buy;
                taker_sell = U128C::from((U256C::from(max_fill_buy.as_u128()) * U256C::from(taker_buy.as_u128()) / U256C::from(max_fill_sell.as_u128())).as_u128());
            }
        } else {
            let max_taker_sell = taker_order.amount_sell - took_sell;
            if max_taker_sell >= max_fill_buy {
                // taker all maker
                taker_buy = max_fill_sell;
                taker_sell = max_fill_buy;
            } else {
                taker_sell = max_taker_sell;
                taker_buy = U128C::from((U256C::from(max_fill_sell.as_u128()) * U256C::from(taker_sell.as_u128()) / U256C::from(max_fill_buy.as_u128())).as_u128());
            }
        }
        let current_filled= if maker_order.fill_buy_or_sell {
            taker_sell.clone()
        } else {
            taker_buy.clone()
        };
        let mut made_order = maker_order.clone();
        made_order.amount_sell = taker_buy.clone();
        made_order.amount_buy = taker_sell.clone();
        made_order.filled = U128C::from(0);

        return (taker_sell, taker_buy, current_filled, made_order);
    }

    pub fn internal_order_is_empty(order: &Order) -> bool {
        return order.amount_buy == U128C::from(0) || order.amount_sell == U128C::from(0) || order.order_id == ""
    }

    pub fn internal_get_opposite_order(made_order: &Order, bot: GridBot, forward_or_reverse: bool, level: usize) -> Order {
        let mut reverse_order = Order{
            order_id: made_order.order_id.clone(),
            token_sell: made_order.token_buy.clone(),
            token_buy: made_order.token_sell.clone(),
            amount_sell: U128C::from(0),
            amount_buy: U128C::from(0),
            fill_buy_or_sell: !made_order.fill_buy_or_sell.clone(),
            filled: U128C::from(0),
        };
        if made_order.fill_buy_or_sell {
            // reverse_order fill sell, fixed sell
            reverse_order.amount_sell = made_order.amount_buy.clone();
            reverse_order.amount_buy = if bot.grid_type == EqOffset {
                let fixed_amount_sell = if bot.grid_buy_count > level.clone() as u16 {
                    // buy grid and marker is forward and maker fixed buy => fixed base
                    // buy grid and marker is reverse and maker fixed buy => forward fixed sell => fixed quote
                    if forward_or_reverse {
                        bot.first_base_amount
                    } else {
                        bot.first_quote_amount
                    }
                } else {
                    // sell grid and maker is forward and maker fixed buy => fixed quote
                    // sell grid and maker is reverse and maker fixed buy => forward fixed sell => fixed base
                    if forward_or_reverse {
                        bot.last_quote_amount
                    } else {
                        bot.last_base_amount
                    }
                };
                made_order.amount_sell.clone() + U128C::from((U256C::from(bot.grid_offset.clone().as_u128()) * U256C::from(reverse_order.amount_sell.as_u128()) / U256C::from(fixed_amount_sell.as_u128())).as_u128())
            } else {
                // made_order.amount_sell.clone() * (GRID_RATE_DENOMINATOR + bot.grid_rate.clone()) / GRID_RATE_DENOMINATOR
                U128C::from((U256C::from(made_order.amount_sell.clone().as_u128()) * (GRID_RATE_DENOMINATOR + bot.grid_rate.clone()) / GRID_RATE_DENOMINATOR).as_u128())
            };
        } else {
            // reverse_order fill buy, fixed buy
            reverse_order.amount_buy = made_order.amount_sell.clone();
            reverse_order.amount_sell = if bot.grid_type == EqOffset {
                let fixed_amount_buy = if bot.grid_buy_count > level.clone() as u16 {
                    // buy grid and maker is forward and maker fixed sell => fixed quote
                    // buy grid and maker is reverse and maker fixed sell => forward fixed buy => fixed base
                    if forward_or_reverse {
                        bot.first_quote_amount
                    } else {
                        bot.first_base_amount
                    }
                } else {
                    // sell grid and fixed buy => fixed quote
                    // sell grid and maker is forward and maker fixed sell => fixed base
                    // sell grid and maker is reverse and maker fixed sell => forward fixed buy => fixed quote
                    if forward_or_reverse {
                        bot.last_base_amount
                    } else {
                        bot.last_quote_amount
                    }
                };
                made_order.amount_buy.clone() - U128C::from((U256C::from(bot.grid_offset.clone().as_u128()) * U256C::from(reverse_order.amount_buy.as_u128()) / U256C::from(fixed_amount_buy.as_u128())).as_u128())
            } else {
                // made_order.amount_buy.clone() * (GRID_RATE_DENOMINATOR - bot.grid_rate.clone()) / GRID_RATE_DENOMINATOR
                // U128C::from((U256C::from(made_order.amount_buy.clone().as_u128()) * (GRID_RATE_DENOMINATOR - bot.grid_rate.clone()) / GRID_RATE_DENOMINATOR).as_u128())
                U128C::from((U256C::from(made_order.amount_buy.clone().as_u128()) * GRID_RATE_DENOMINATOR / (GRID_RATE_DENOMINATOR + bot.grid_rate.clone())).as_u128())
            };
        }
        return reverse_order;
    }

    pub fn internal_calculate_bot_revenue(&self, forward_or_reverse: bool, order: Order, opposite_order: Order, current_filled: Balance) -> (AccountId, Balance, Balance) {
        if forward_or_reverse {
            return (opposite_order.token_sell, 0, 0);
        }
        // let forward_order = GridBotContract::internal_get_first_forward_order(bot, pair, level);
        let revenue_token;
        let mut revenue;
        // TODO had made_order, maybe can use mad_order
        if opposite_order.fill_buy_or_sell {
            // current_filled token is forward_order's buy token
            // revenue token is forward_order's sell token
            let forward_sold = (U256C::from(current_filled.clone()) * U256C::from(opposite_order.amount_sell.as_u128()) / U256C::from(opposite_order.amount_buy.as_u128())).as_u128();
            let reverse_bought = (U256C::from(current_filled.clone()) * U256C::from(order.amount_buy.as_u128()) / U256C::from(order.amount_sell.as_u128())).as_u128();
            require!(reverse_bought >= forward_sold, INVALID_REVENUE);
            revenue_token = opposite_order.token_sell;
            revenue = reverse_bought - forward_sold;
        } else {
            // current_filled token is forward_order's sell token
            // revenue token is forward_order's buy token
            let forward_bought = (U256C::from(current_filled.clone()) * U256C::from(opposite_order.amount_buy.as_u128()) / U256C::from(opposite_order.amount_sell.as_u128())).as_u128();
            let reverse_sold = (U256C::from(current_filled.clone()) * U256C::from(order.amount_sell.as_u128()) / U256C::from(order.amount_buy.as_u128())).as_u128();
            require!(forward_bought >= reverse_sold, INVALID_REVENUE);
            revenue_token = opposite_order.token_buy;
            revenue = forward_bought - reverse_sold;
        };
        let protocol_fee = (U256C::from(revenue) * U256C::from(self.protocol_fee_rate.clone()) / U256C::from(PROTOCOL_FEE_DENOMINATOR)).as_u128();
        revenue -= protocol_fee;
        return (revenue_token, revenue.clone(), protocol_fee.clone());
    }

    fn private_place_order(order: Order, placed_orders: &mut Vec<Order>, level: usize) {
        let placed_order = &mut placed_orders[level.clone()];
        if GridBotContract::internal_order_is_empty(placed_order) {
            placed_orders[level.clone()] = order;
            return;
        }
        // merge order
        placed_order.amount_sell += order.amount_sell;
        placed_order.amount_buy += order.amount_buy;
    }

}
