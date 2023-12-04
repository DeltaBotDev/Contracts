use crate::*;
use near_sdk::{near_bindgen};

#[near_bindgen]
impl GridBotContract {

    /// return (order, in_orderbook)
    pub fn query_order(&self, bot_id: String, forward_or_reverse: bool, level: usize) -> (Order, bool) {
        assert!(self.order_map.contains_key(&bot_id), "VALID_BOT_ID");
        assert!(self.bot_map.contains_key(&bot_id), "VALID_BOT_ID");
        let bot = self.bot_map.get(&bot_id).unwrap();
        assert!(!(bot.closed.clone()), "BOT_CLOSED");
        assert!(bot.active.clone(), "BOT_DISABLE");
        assert!(self.pair_map.contains_key(&(bot.pair_id.clone())), "VALID_PAIR_ID");
        let bot_orders = self.order_map.get(&bot_id).unwrap();
        let orders = if forward_or_reverse { &bot_orders[FORWARD_ORDERS_INDEX.clone()] } else { &bot_orders[REVERSE_ORDERS_INDEX.clone()] };
        // check order
        if GridBotContract::internal_order_is_empty(&(orders[level])) {
            // The current grid order has not been placed yet
            let pair = self.pair_map.get(&(bot.pair_id.clone())).unwrap();
            return (GridBotContract::internal_get_first_forward_order(bot.clone(), pair.clone(), level.clone()), false);
        }
        return (orders[level.clone()].clone(), true);
    }

    pub fn query_orders(&self, bot_ids: Vec<String>, forward_or_reverses: Vec<bool>, levels: Vec<usize>) -> Vec<Order> {
        assert_eq!(bot_ids.len(), forward_or_reverses.len(), "VALID_PARAM");
        assert_eq!(levels.len(), forward_or_reverses.len(), "VALID_PARAM");

        let mut orders: Vec<Order> = Vec::with_capacity(bot_ids.len());
        for (index, bot_id) in bot_ids.iter().enumerate() {
            let (order, _) = self.query_order(bot_id.clone(), forward_or_reverses[index.clone()].clone(), levels[index].clone());
            orders.push(order);
        }
        return orders;
    }

    pub fn estimate_calculate(&self, bot_id: String, forward_or_reverse: bool, level: usize, taker_order: &Order) -> (U128C, U128C, U128C) {
        let (maker_order, _) = self.query_order(bot_id, forward_or_reverse, level);
        // matching check
        GridBotContract::internal_check_order_match(maker_order.clone(), taker_order.clone());
        // calculate
        return GridBotContract::internal_calculate_matching(maker_order.clone(), taker_order.clone());
    }
}
