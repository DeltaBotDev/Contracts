use crate::*;
use near_sdk::{near_bindgen, require};

#[near_bindgen]
impl GridBotContract {
    pub fn query_bot(&self, bot_id: String) -> GridBot {
        require!(self.bot_map.contains_key(&bot_id), INVALID_BOT_ID);
        return self.bot_map.get(&bot_id).unwrap().clone();
    }

    pub fn query_bots(&self, bot_ids: Vec<String>) -> Vec<GridBot> {
        let mut grid_bots: Vec<GridBot> = Vec::with_capacity(bot_ids.len());
        for (_, bot_id) in bot_ids.iter().enumerate() {
            let grid_bot = self.query_bot(bot_id.clone());
            grid_bots.push(grid_bot);
        }
        return grid_bots;
    }
}
