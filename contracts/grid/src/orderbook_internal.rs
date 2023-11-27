use crate::*;

impl GridBotContract {
    pub fn internal_place_order(&mut self, bot_id: String, order_id: String, token_sell: AccountId, token_buy: AccountId, amount_sell: U128C, amount_buy:U128C, fill_buy_or_sell: bool, forward_or_reverse: bool, level: u16) {
        assert!(self.bot_map.contains_key(&bot_id), "INVALID_BOT_ID_FOR_BOT_MAP");
        assert!(self.order_map.contains_key(&bot_id), "INVALID_BOT_ID_FOR_ORDER_MAP");
        assert_eq!(self.order_map.get(&bot_id).unwrap().len(), ORDER_POSITION_SIZE.clone() as usize, "INVALID_ORDER_POSITION_LEN");

        let order = Order{
            order_id,
            token_sell,
            token_buy,
            amount_sell,
            amount_buy,
            fill_buy_or_sell,
            filled: U128C::from(0),
        };
        let bot_orders = self.order_map.get_mut(&bot_id).unwrap();
        let orders = if forward_or_reverse { &mut bot_orders[FORWARD_ORDERS_INDEX.clone()] } else { &mut bot_orders[REVERSE_ORDERS_INDEX.clone()] };
        GridBotContract::private_place_order(order, orders, level.clone() as usize);
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
