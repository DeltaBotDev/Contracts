use crate::*;
use near_sdk::{near_bindgen, require};
use near_sdk::json_types::U128;

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

    pub fn query_protocol_fee(&self, token: AccountId) -> U128 {
        require!(self.protocol_fee_map.contains_key(&token), INVALID_TOKEN);
        return U128::from(self.internal_get_protocol_fee(&token).as_u128());
    }

    pub fn query_global_balance(&self, token: AccountId) -> U128 {
        require!(self.global_balances_map.contains_key(&token), INVALID_TOKEN);
        return U128::from(self.internal_get_global_balance(&token).as_u128());
    }

    pub fn query_user_balance(&self, user: AccountId, token: AccountId) -> U128 {
        require!(self.user_balances_map.contains_key(&user), INVALID_USER);
        return U128::from(self.internal_get_user_balance(&user, &token).as_u128());
    }

    pub fn query_user_locked_balance(&self, user: AccountId, token: AccountId) -> U128 {
        require!(self.user_locked_balances_map.contains_key(&user), INVALID_USER);
        return U128::from(self.internal_get_user_locked_balance(&user, &token).as_u128());
    }

    pub fn query_pair_info(&self, pair_id: String) -> Pair {
        require!(self.pair_map.contains_key(&pair_id), INVALID_PAIR_ID);
        return self.pair_map.get(&pair_id).unwrap().clone();
    }

    pub fn query_protocol_fee_rate(&self) -> U128 {
        return U128::from(self.protocol_fee_rate.clone());
    }

    pub fn query_market_user(&self, user: AccountId) -> bool {
        if !self.market_user_map.contains_key(&user) {
            return false;
        }
        return self.market_user_map.get(&user).unwrap();
    }

    pub fn query_min_deposit(&self, token: AccountId) -> U128 {
        if !self.deposit_limit_map.contains_key(&token) {
            return U128::from(0);
        }
        return U128::from(self.protocol_fee_rate.clone());
    }

    pub fn query_refer_fee(&self, user: AccountId, token: AccountId) -> U128 {
        if !self.refer_fee_map.contains_key(&user) {
            return U128::from(0);
        }
        let token_map = self.refer_fee_map.get(&user).unwrap();
        if !token_map.contains_key(&token) {
            return U128::from(0);
        }
        return token_map.get(&token).unwrap();
    }

    // start and end, is start from 1
    pub fn query_invited_users(&self, user: AccountId, start: U128, end: U128) -> Vec<AccountId> {
        require!(start.0 >= 1, INVALID_NUM);
        require!(end.0 >= start.0, INVALID_NUM);
        if !self.refer_recommender_user_map.contains_key(&user) {
            return vec![];
        }
        let invited_users = self.refer_recommender_user_map.get(&user).unwrap();
        if (invited_users.len() as u128) < start.0 {
            return vec![];
        }
        let mut queried_users: Vec<AccountId>;
        queried_users = vec![];
        for index in start.0..(end.0 + 1) {
            if (invited_users.len() as u128) < index {
                break;
            }
            queried_users.push(invited_users.get((index - 1) as u64).unwrap());
        }
        return queried_users;
    }

    // pub fn query_storage_fee(&self) -> U256C {
    //     return U256C::from(self.storage_fee.clone());
    // }
}
