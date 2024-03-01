use near_sdk::{AccountId, env, Gas};
use crate::{GridBot, GridBotContract, Pair, U256C};

impl GridBotContract {
    pub fn internal_get_next_bot_id(&self) -> u128 {
        return self.next_bot_id.clone();
    }

    pub fn internal_get_user_balance(&self, user: &AccountId, token: &AccountId) -> U256C {
        return self.user_balances_map.get(user)
            .and_then(|balances| balances.get(token))
            .unwrap_or(U256C::from(0));
    }

    pub fn internal_get_user_locked_balance(&self, user: &AccountId, token: &AccountId) -> U256C {
        return self.user_locked_balances_map.get(user)
            .and_then(|balances| balances.get(token))
            .unwrap_or(U256C::from(0));
    }

    pub fn internal_get_global_balance(&self, token: &AccountId) -> U256C {
        if !self.global_balances_map.contains_key(token) {
            return U256C::from(0);
        }
        return self.global_balances_map.get(token).unwrap().clone();
    }

    pub fn internal_get_protocol_fee(&self, token: &AccountId) -> U256C {
        if !self.protocol_fee_map.contains_key(token) {
            return U256C::from(0);
        }
        return self.protocol_fee_map.get(token).unwrap().clone();
    }

    pub fn internal_get_pair_key(base_token: AccountId, quote_token: AccountId) -> String {
        return format!("{}:{}", base_token.clone().to_string(), quote_token.clone().to_string());
    }

    pub fn internal_get_recommender(&self, user: &AccountId) -> Option<AccountId> {
        return self.refer_user_recommender_map.get(user);
    }

    pub fn internal_get_remaining_gas(&self) -> Gas {
        let prepaid_gas = env::prepaid_gas();
        let used_gas = env::used_gas();
        return prepaid_gas - used_gas;
    }

    pub fn internal_get_bot_near_amount(&self, grid: &GridBot, pair: &Pair) -> u128 {
        if pair.base_token == self.wnear {
            return grid.total_base_amount.as_u128();
        } else if pair.quote_token == self.wnear {
            return grid.total_quote_amount.as_u128();
        }
        return 0 as u128;
    }

}
