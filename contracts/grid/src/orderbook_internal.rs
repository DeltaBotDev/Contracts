use near_sdk::require;
use crate::*;
use crate::entity::GridType::EqOffset;

impl GridBotContract {
    pub fn internal_place_order(&mut self, bot_id: String, order: Order, forward_or_reverse: bool, level: usize) {
        require!(self.bot_map.contains_key(&bot_id), INVALID_BOT_ID_FOR_BOT_MAP);
        require!(self.order_map.contains_key(&bot_id), INVALID_BOT_ID_FOR_ORDER_MAP);
        require!(self.order_map.get(&bot_id).unwrap().len() == ORDER_POSITION_SIZE.clone() as usize, INVALID_ORDER_POSITION_LEN);

        let bot_orders = self.order_map.get_mut(&bot_id).unwrap();
        let orders = if forward_or_reverse { &mut bot_orders[FORWARD_ORDERS_INDEX.clone()] } else { &mut bot_orders[REVERSE_ORDERS_INDEX.clone()] };
        GridBotContract::private_place_order(order, orders, level.clone());
    }

    pub fn internal_take_order(&mut self, bot_id: String, forward_or_reverse: bool, level: usize, taker_order: &Order) -> (U128C, U128C) {
        let bot = self.bot_map.get(&bot_id.clone()).unwrap().clone();
        let pair = self.pair_map.get(&bot.pair_id).unwrap().clone();
        let (mut maker_order, in_orderbook) = self.query_order(bot_id.clone(), forward_or_reverse, level);
        // matching check
        GridBotContract::internal_check_order_match(maker_order.clone(), taker_order.clone());

        // calculate
        let (taker_sell, taker_buy, current_filled) = GridBotContract::internal_calculate_matching(maker_order.clone(), taker_order.clone());
        maker_order.filled += current_filled;

        // place into orderbook
        if !in_orderbook {
            self.internal_place_order(bot_id.clone(), maker_order.clone(), forward_or_reverse.clone(), level.clone());
        }
        // place opposite order
        let opposite_order = GridBotContract::internal_get_reserve_order(maker_order.clone(), bot.clone(), level.clone());
        self.internal_place_order(bot_id.clone(), opposite_order.clone(), !forward_or_reverse.clone(), level.clone());

        // calculate bot's revenue
        let (revenue_token, revenue, protocol_fee) = self.internal_calculate_bot_revenue(forward_or_reverse.clone(), maker_order, opposite_order, current_filled.as_u128());
        // add revenue
        let bot_mut = self.bot_map.get_mut(&bot_id.clone()).unwrap();
        bot_mut.revenue += revenue;
        // update bot asset
        GridBotContract::internal_update_bot_asset(bot_mut, &pair, taker_order.token_buy.clone(), taker_buy.as_u128(), taker_sell.as_u128());

        // bot asset transfer
        self.internal_reduce_locked_assets(&(bot.user), &(taker_order.token_buy), &taker_buy);
        self.internal_increase_locked_assets(&(bot.user), &(taker_order.token_sell), &taker_sell);
        // update global
        self.internal_reduce_global_asset(&(taker_order.token_buy), &taker_buy);
        self.internal_increase_global_asset(&(taker_order.token_sell), &taker_sell);

        // handle protocol fee
        self.internal_add_protocol_fee(&revenue_token, protocol_fee, bot_id, &pair);

        return (taker_sell, taker_buy);
    }

    pub fn internal_check_order_match(maker_order: Order, taker_order: Order) {
        require!(maker_order.token_buy == taker_order.token_sell, INVALID_ORDER_TOKEN);
        require!(maker_order.token_sell == taker_order.token_buy, INVALID_ORDER_TOKEN);
        require!(taker_order.token_sell != taker_order.token_buy, INVALID_ORDER_TOKEN);
        require!(taker_order.amount_sell != U128C::from(0), INVALID_ORDER_AMOUNT);
        require!(taker_order.amount_buy != U128C::from(0), INVALID_ORDER_AMOUNT);

        require!(taker_order.amount_sell/taker_order.amount_buy <= maker_order.amount_sell/maker_order.amount_buy, INVALID_PRICE);
    }

    pub fn internal_calculate_matching(maker_order: Order, taker_order: Order) -> (U128C, U128C, U128C) {
        // calculate marker max amount
        let max_fill_sell;
        let max_fill_buy;
        if maker_order.fill_buy_or_sell {
            max_fill_buy = maker_order.amount_buy - maker_order.filled;
            max_fill_sell = maker_order.amount_sell / maker_order.amount_buy * max_fill_buy;
        } else {
            max_fill_sell = maker_order.amount_sell - maker_order.filled;
            max_fill_buy = maker_order.amount_buy / maker_order.amount_sell * max_fill_sell;
        }
        // calculate matching amount
        let taker_sell;
        let taker_buy;
        if taker_order.fill_buy_or_sell {
            if taker_order.amount_buy >= max_fill_sell {
                // taker all maker
                taker_buy = max_fill_sell;
                taker_sell = max_fill_buy;
            } else {
                taker_buy = taker_order.amount_buy;
                taker_sell = max_fill_buy / max_fill_sell * taker_buy;
            }
        } else {
            if taker_order.amount_sell >= max_fill_buy {
                // taker all maker
                taker_buy = max_fill_sell;
                taker_sell = max_fill_buy;
            } else {
                taker_sell = taker_order.amount_sell;
                taker_buy = max_fill_sell / max_fill_buy * taker_sell;
            }
        }
        let current_filled= if maker_order.fill_buy_or_sell {
            taker_sell.clone()
        } else {
            taker_buy.clone()
        };
        return (taker_sell, taker_buy, current_filled);
    }

    pub fn internal_order_is_empty(order: &Order) -> bool {
        return order.amount_buy == U128C::from(0) || order.amount_sell == U128C::from(0) || order.order_id == ""
    }

    pub fn internal_get_reserve_order(maker_order: Order, bot: GridBot, level: usize) -> Order {
        let mut reverse_order = Order{
            order_id: maker_order.order_id.clone(),
            token_sell: maker_order.token_buy.clone(),
            token_buy: maker_order.token_sell.clone(),
            amount_sell: U128C::from(0),
            amount_buy: U128C::from(0),
            fill_buy_or_sell: !maker_order.fill_buy_or_sell.clone(),
            filled: U128C::from(0),
        };
        if maker_order.fill_buy_or_sell {
            // reverse_order fill sell, fixed sell
            reverse_order.amount_sell = maker_order.amount_buy.clone();
            reverse_order.amount_buy = if bot.grid_type == EqOffset {
                let fixed_amount_sell = if bot.grid_buy_count > level.clone() as u16 {
                    // buy grid and fixed sell => fixed quote
                    bot.first_quote_amount
                } else {
                    // sell grid and fixed sell => fixed base
                    bot.first_base_amount
                };
                maker_order.amount_sell.clone() + bot.grid_offset.clone() / fixed_amount_sell * reverse_order.amount_sell
            } else {
                maker_order.amount_sell.clone() * (GRID_RATE_DENOMINATOR + bot.grid_rate.clone()) / GRID_RATE_DENOMINATOR
            };
        } else {
            // reverse_order fill buy, fixed buy
            reverse_order.amount_buy = maker_order.amount_sell.clone();
            reverse_order.amount_sell = if bot.grid_type == EqOffset {
                let fixed_amount_buy = if bot.grid_buy_count > level.clone() as u16 {
                    // buy grid and fixed buy => fixed base
                    bot.first_base_amount
                } else {
                    // sell grid and fixed buy => fixed quote
                    bot.first_quote_amount
                };
                maker_order.amount_buy.clone() - bot.grid_offset.clone() / fixed_amount_buy * reverse_order.amount_buy
            } else {
                maker_order.amount_buy.clone() * (GRID_RATE_DENOMINATOR - bot.grid_rate.clone()) / GRID_RATE_DENOMINATOR
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
        if opposite_order.fill_buy_or_sell {
            // current_filled token is forward_order's buy token
            // revenue token is forward_order's sell token
            let forward_sold = current_filled.clone() * opposite_order.amount_sell.as_u128() / opposite_order.amount_buy.as_u128();
            let reverse_bought = current_filled.clone() * order.amount_buy.as_u128() / order.amount_sell.as_u128();
            require!(reverse_bought >= forward_sold, INVALID_REVENUE);
            revenue_token = opposite_order.token_sell;
            revenue = reverse_bought - forward_sold;
        } else {
            // current_filled token is forward_order's sell token
            // revenue token is forward_order's buy token
            let forward_bought = current_filled.clone() * opposite_order.amount_buy.as_u128() / opposite_order.amount_sell.as_u128();
            let reverse_sold = current_filled.clone() * order.amount_sell.as_u128() / order.amount_buy.as_u128();
            require!(forward_bought >= reverse_sold, INVALID_REVENUE);
            revenue_token = opposite_order.token_buy;
            revenue = forward_bought - reverse_sold;
        };
        let protocol_fee = revenue * self.protocol_fee_rate.clone() / PROTOCOL_FEE_DENOMINATOR;
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
