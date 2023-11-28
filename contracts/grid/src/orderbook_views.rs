use crate::*;
use near_sdk::{near_bindgen};

#[near_bindgen]
impl GridBotContract {

    pub fn query_order(&self, bot_id: String, forward_or_reverse: bool, level: usize) -> Order {
        assert!(self.order_map.contains_key(&bot_id), "VALID_BOT_ID");
        let bot_orders = self.order_map.get(&bot_id).unwrap();
        let orders = if forward_or_reverse { &bot_orders[FORWARD_ORDERS_INDEX.clone()] } else { &bot_orders[REVERSE_ORDERS_INDEX.clone()] };
        return orders[level].clone();
    }

    pub fn query_orders(&self, bot_ids: Vec<String>, forward_or_reverses: Vec<bool>, levels: Vec<usize>) -> Vec<Order> {
        assert_eq!(bot_ids.len(), forward_or_reverses.len(), "VALID_PARAM");
        assert_eq!(levels.len(), forward_or_reverses.len(), "VALID_PARAM");

        let mut orders: Vec<Order> = Vec::with_capacity(bot_ids.len());
        for (index, bot_id) in bot_ids.iter().enumerate() {
            let order = self.query_order(bot_id.clone(), forward_or_reverses[index], levels[index]);
            orders.push(order);
        }
        return orders;
    }

    pub fn estimate_calculate(&self, bot_id: String, forward_or_reverse: bool, level: usize, taker_order: Order) -> (U128C, U128C) {
        let maker_order = self.query_order(bot_id, forward_or_reverse, level);
        // matching check
        GridBotContract::internal_check_order_match(maker_order.clone(), taker_order.clone());
        // calculate
        return GridBotContract::internal_calculate_matching(maker_order.clone(), taker_order.clone());
    }
}
