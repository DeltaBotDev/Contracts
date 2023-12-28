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

    pub fn query_protocol_fee(&self, token: AccountId) -> U256C {
        require!(self.protocol_fee_map.contains_key(&token), INVALID_TOKEN);
        return self.internal_get_protocol_fee(&token);
    }

    pub fn query_global_balance(&self, token: AccountId) -> U256C {
        require!(self.global_balances_map.contains_key(&token), INVALID_TOKEN);
        return self.internal_get_global_balance(&token);
    }

    pub fn query_user_balance(&self, user: AccountId, token: AccountId) -> U256C {
        require!(self.user_balances_map.contains_key(&user), INVALID_USER);
        return self.internal_get_user_balance(&user, &token);
    }

    pub fn query_user_locked_balance(&self, user: AccountId, token: AccountId) -> U256C {
        require!(self.user_locked_balances_map.contains_key(&user), INVALID_USER);
        return self.internal_get_user_locked_balance(&user, &token);
    }

    pub fn query_pair_info(&self, pair_id: String) -> Pair {
        require!(self.pair_map.contains_key(&pair_id), INVALID_PAIR_ID);
        return self.pair_map.get(&pair_id).unwrap().clone();
    }

    pub fn query_protocol_fee_rate(&self) -> U256C {
        return U256C::from(self.protocol_fee_rate.clone());
    }

    pub fn query_market_user(&self, user: AccountId) -> bool {
        if !self.market_user_map.contains_key(&user) {
            return false;
        }
        return self.market_user_map.get(&user).unwrap();
    }

    // pub fn query_storage_fee(&self) -> U256C {
    //     return U256C::from(self.storage_fee.clone());
    // }
}
