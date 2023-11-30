use crate::*;
use crate::entity::GridType::EqOffset;

impl GridBotContract {
    // pub fn internal_place_order(&mut self, bot_id: String, order_id: String, token_sell: AccountId, token_buy: AccountId, amount_sell: U128C, amount_buy:U128C, fill_buy_or_sell: bool, forward_or_reverse: bool, level: u16) {
    //     assert!(self.bot_map.contains_key(&bot_id), "INVALID_BOT_ID_FOR_BOT_MAP");
    //     assert!(self.order_map.contains_key(&bot_id), "INVALID_BOT_ID_FOR_ORDER_MAP");
    //     assert_eq!(self.order_map.get(&bot_id).unwrap().len(), ORDER_POSITION_SIZE.clone() as usize, "INVALID_ORDER_POSITION_LEN");
    //
    //     let order = Order{
    //         order_id,
    //         token_sell,
    //         token_buy,
    //         amount_sell,
    //         amount_buy,
    //         fill_buy_or_sell,
    //         filled: U128C::from(0),
    //     };
    //     let bot_orders = self.order_map.get_mut(&bot_id).unwrap();
    //     let orders = if forward_or_reverse { &mut bot_orders[FORWARD_ORDERS_INDEX.clone()] } else { &mut bot_orders[REVERSE_ORDERS_INDEX.clone()] };
    //     GridBotContract::private_place_order(order, orders, level.clone() as usize);
    // }
    pub fn internal_place_order(&mut self, bot_id: String, order: Order, forward_or_reverse: bool, level: usize) {
        assert!(self.bot_map.contains_key(&bot_id), "INVALID_BOT_ID_FOR_BOT_MAP");
        assert!(self.order_map.contains_key(&bot_id), "INVALID_BOT_ID_FOR_ORDER_MAP");
        assert_eq!(self.order_map.get(&bot_id).unwrap().len(), ORDER_POSITION_SIZE.clone() as usize, "INVALID_ORDER_POSITION_LEN");

        let bot_orders = self.order_map.get_mut(&bot_id).unwrap();
        let orders = if forward_or_reverse { &mut bot_orders[FORWARD_ORDERS_INDEX.clone()] } else { &mut bot_orders[REVERSE_ORDERS_INDEX.clone()] };
        GridBotContract::private_place_order(order, orders, level.clone());
    }

    pub fn internal_take_order(&mut self, bot_id: String, forward_or_reverse: bool, level: usize, taker_order: Order) -> (U128C, U128C) {
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
        // place reverse order
        let reverse_order = GridBotContract::internal_get_reserve_order(maker_order.clone(), bot.clone(), level.clone());
        self.internal_place_order(bot_id.clone(), reverse_order.clone(), !forward_or_reverse.clone(), level.clone());

        // calculate bot's revenue
        let revenue = self.internal_calculate_bot_revenue(bot.clone(), pair.clone(), forward_or_reverse.clone(), maker_order, current_filled.as_u128(), level.clone());
        // add revenue
        let bot_mut = self.bot_map.get_mut(&bot_id.clone()).unwrap();
        bot_mut.revenue += revenue;
        // update bot asset
        GridBotContract::internal_update_bot_asset(bot_mut, pair, taker_order.token_buy.clone(), taker_order.token_sell.clone(), taker_buy.as_u128(), taker_sell.as_u128());

        // bot asset transfer
        self.internal_add_bot_assets(bot.user.clone(), taker_order.token_sell, taker_sell);
        self.internal_reduce_bot_assets(bot.user.clone(), taker_order.token_buy, taker_buy);

        return (taker_sell, taker_buy);
    }

    pub fn internal_update_bot_asset(bot: &mut GridBot, pair: Pair, token_sell: AccountId, token_buy: AccountId, amount_sell: Balance, amount_buy: Balance) {
        if pair.base_token == token_sell {
            bot.total_base_amount = bot.total_base_amount.checked_sub(amount_sell).expect("Base amount underflow");
            bot.total_quote_amount = bot.total_quote_amount.checked_add(amount_buy).expect("Quote amount overflow");
        } else {
            bot.total_base_amount = bot.total_base_amount.checked_add(amount_buy).expect("Base amount overflow");
            bot.total_quote_amount = bot.total_quote_amount.checked_sub(amount_sell).expect("Quote amount underflow");
        }
    }

    pub fn internal_add_bot_assets(&mut self, user: AccountId, token: AccountId, amount: U128C) {
        if amount == U128C::from(0) {
            return;
        }
        let user_locked_balances = self.user_locked_balances_map.get_mut(&user).unwrap();
        let locked_balance = user_locked_balances.get_mut(&token).unwrap();
        *locked_balance += amount;
    }

    pub fn internal_reduce_bot_assets(&mut self, user: AccountId, token: AccountId, amount: U128C) {
        if amount == U128C::from(0) {
            return;
        }
        let user_locked_balances = self.user_locked_balances_map.get_mut(&user).unwrap();
        let locked_balance = user_locked_balances.get_mut(&token).unwrap();
        *locked_balance -= amount;
    }

    pub fn internal_check_order_match(maker_order: Order, taker_order: Order) {
        assert_eq!(maker_order.token_buy, taker_order.token_sell, "VALID_ORDER_TOKEN");
        assert_eq!(maker_order.token_sell, taker_order.token_buy, "VALID_ORDER_TOKEN");
        assert_ne!(taker_order.token_sell, taker_order.token_buy, "VALID_ORDER_TOKEN");
        assert_ne!(taker_order.amount_sell, U128C::from(0), "VALID_ORDER_AMOUNT");
        assert_ne!(taker_order.amount_buy, U128C::from(0), "VALID_ORDER_AMOUNT");

        assert!(taker_order.amount_sell/taker_order.amount_buy <= maker_order.amount_sell/maker_order.amount_buy, "VALID_PRICE");
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

    pub fn internal_order_is_empty(order: Order) -> bool {
        return order.amount_buy == U128C::from(0) || order.amount_sell == U128C::from(0)
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

    pub fn internal_calculate_bot_revenue(&self, bot: GridBot, pair: Pair, forward_or_reverse: bool, order: Order, current_filled: Balance, level: usize) -> Balance {
        if forward_or_reverse {
            return 0;
        }
        let forward_order = GridBotContract::internal_get_first_forward_order(bot, pair, level);
        if forward_order.fill_buy_or_sell {
            // current_filled token is forward_order's buy token
            // revenue token is forward_order's sell token
            let forward_sold = current_filled.clone() * forward_order.amount_sell.as_u128() / forward_order.amount_buy.as_u128();
            let reverse_bought = current_filled.clone() * order.amount_buy.as_u128() / order.amount_sell.as_u128();
            assert!(reverse_bought >= forward_sold, "VALID_REVENUE");
            reverse_bought - forward_sold
        } else {
            // current_filled token is forward_order's sell token
            // revenue token is forward_order's buy token
            let forward_bought = current_filled.clone() * forward_order.amount_buy.as_u128() / forward_order.amount_sell.as_u128();
            let reverse_sold = current_filled.clone() * order.amount_sell.as_u128() / order.amount_buy.as_u128();
            assert!(forward_bought >= reverse_sold, "VALID_REVENUE");
            forward_bought - reverse_sold
        }
    }

    fn private_place_order(order: Order, placed_orders: &mut Vec<Order>, level: usize) {
        let placed_order = &mut placed_orders[level.clone()];
        if placed_order.order_id == "" {
            placed_orders[level.clone()] = order;
            return;
        }
        // merge order
        placed_order.amount_sell += order.amount_sell;
        placed_order.amount_buy += order.amount_buy;
    }

}
