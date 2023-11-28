use crate::*;
use near_sdk::{near_bindgen};
use crate::entity::GridType::EqOffset;

#[near_bindgen]
impl GridBotContract {

    pub fn take_order(&mut self, bot_id: String, forward_or_reverse: bool, level: usize, taker_order: Order) -> (U128C, U128C) {
        let (mut maker_order, in_orderbook) = self.query_order(bot_id.clone(), forward_or_reverse, level);
        // matching check
        GridBotContract::internal_check_order_match(maker_order.clone(), taker_order.clone());
        // calculate
        let (taker_sell, taker_buy) = GridBotContract::internal_calculate_matching(maker_order.clone(), taker_order.clone());
        if maker_order.fill_buy_or_sell {
            maker_order.filled += taker_sell.clone();
        } else {
            maker_order.filled += taker_buy.clone();
        }
        if !in_orderbook {
            // add into orderbook
            self.internal_place_order(bot_id.clone(), maker_order.clone(), forward_or_reverse.clone(), level.clone());
        }
        let bot = self.bot_map.get(&bot_id.clone()).unwrap();
        let reverse_order = GridBotContract::internal_get_reserve_order(maker_order, bot.clone(), level.clone());
        self.internal_place_order(bot_id.clone(), reverse_order.clone(), !forward_or_reverse.clone(), level.clone());
        return (taker_sell, taker_buy);
    }
}
