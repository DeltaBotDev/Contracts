use crate::*;
use near_sdk::{near_bindgen, require};

#[near_bindgen]
impl GridBotContract {

    /// return (order, in_orderbook)
    pub fn query_order(&self, bot_id: String, forward_or_reverse: bool, level: usize) -> (Order, bool) {
        require!(self.order_map.contains_key(&bot_id), INVALID_BOT_ID);
        require!(self.bot_map.contains_key(&bot_id), INVALID_BOT_ID);
        let bot = self.bot_map.get(&bot_id).unwrap();
        require!(!(bot.closed.clone()), BOT_CLOSED);
        require!(bot.active.clone(), BOT_DISABLE);
        require!(self.pair_map.contains_key(&(bot.pair_id.clone())), INVALID_PAIR_ID);
        // check timestamp
        require!(bot.valid_until_time >= U256C::from(env::block_timestamp_ms()), BOT_EXPIRED);
        let bot_orders = self.order_map.get(&bot_id).unwrap();
        let orders = if forward_or_reverse {
            bot_orders.get(FORWARD_ORDERS_INDEX).unwrap()
        } else {
            bot_orders.get(REVERSE_ORDERS_INDEX).unwrap()
        };
        // check order
        let order = &orders.get(level as u64).unwrap();
        if GridBotContract::internal_order_is_empty(order) {
            require!(forward_or_reverse, INVALID_FORWARD_OR_REVERSE);
            // The current grid order has not been placed yet
            let pair = self.pair_map.get(&(bot.pair_id.clone())).unwrap();
            return ((GridBotContract::internal_get_first_forward_order(bot.clone(), pair.clone(), level.clone())), false);
        }
        return (order.clone(), true);
    }

    pub fn query_orders(&self, bot_ids: Vec<String>, forward_or_reverses: Vec<bool>, levels: Vec<usize>) -> Vec<Order> {
        require!(bot_ids.len() == forward_or_reverses.len(), INVALID_PARAM);
        require!(levels.len() == forward_or_reverses.len(), INVALID_PARAM);

        let mut orders: Vec<Order> = Vec::with_capacity(bot_ids.len());
        for (index, bot_id) in bot_ids.iter().enumerate() {
            let (order, _) = self.query_order(bot_id.clone(), forward_or_reverses[index.clone()].clone(), levels[index].clone());
            orders.push(order);
        }
        return orders;
    }

    pub fn estimate_calculate(&self, bot_id: String, forward_or_reverse: bool, level: usize, taker_order: &Order) -> (U256C, U256C, U256C, Order) {
        let (maker_order, _) = self.query_order(bot_id, forward_or_reverse, level);
        // matching check
        GridBotContract::internal_check_order_match(maker_order.clone(), taker_order.clone());
        // calculate
        return GridBotContract::internal_calculate_matching(maker_order.clone(), taker_order.clone(), U256C::from(0), U256C::from(0));
    }
}
