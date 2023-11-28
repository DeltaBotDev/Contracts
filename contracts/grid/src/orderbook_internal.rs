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

    pub fn internal_check_order_match(marker_order: Order, taker_order: Order) {
        assert_eq!(marker_order.token_buy, taker_order.token_sell, "VALID_ORDER_TOKEN");
        assert_eq!(marker_order.token_sell, taker_order.token_buy, "VALID_ORDER_TOKEN");
        assert_ne!(taker_order.token_sell, taker_order.token_buy, "VALID_ORDER_TOKEN");
        assert_ne!(taker_order.amount_sell, U128C::from(0), "VALID_ORDER_AMOUNT");
        assert_ne!(taker_order.amount_buy, U128C::from(0), "VALID_ORDER_AMOUNT");

        assert!(taker_order.amount_sell/taker_order.amount_buy <= marker_order.amount_sell/marker_order.amount_buy, "VALID_PRICE");
    }

    pub fn internal_calculate_matching(marker_order: Order, taker_order: Order) -> (U128C, U128C) {
        // calculate marker max amount
        let max_fill_sell;
        let max_fill_buy;
        if marker_order.fill_buy_or_sell {
            max_fill_buy = marker_order.amount_buy - marker_order.filled;
            max_fill_sell = marker_order.amount_sell / marker_order.amount_buy * max_fill_buy;
        } else {
            max_fill_sell = marker_order.amount_sell - marker_order.filled;
            max_fill_buy = marker_order.amount_buy / marker_order.amount_sell * max_fill_sell;
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
        return (taker_sell, taker_buy);
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
