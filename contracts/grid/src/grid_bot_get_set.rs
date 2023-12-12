use near_sdk::{AccountId, env};
use crate::{GridBotContract, U128C};

impl GridBotContract {
    pub fn internal_get_next_bot_id(&self) -> u128 {
        return self.next_bot_id.clone();
    }

    pub fn internal_get_user_balance(&self, user: &AccountId, token: &AccountId) -> U128C {
        return self.user_balances_map.get(user)
            .and_then(|balances| balances.get(token).cloned())
            .unwrap_or(U128C::from(0));
    }

    pub fn internal_get_global_balance(&self, token: &AccountId) -> U128C {
        if !self.global_balances_map.contains_key(token) {
            return U128C::from(0);
        }
        return self.global_balances_map.get(token).unwrap().clone();
    }

    pub fn internal_get_protocol_fee(&self, token: &AccountId) -> U128C {
        return self.protocol_fee_map.get(token).unwrap().clone();
    }

    pub fn internal_get_oracle_price(&self, pair_id: String) -> U128C {
        if !self.oracle_price_map.contains_key(&pair_id) {
            return U128C::from(0);
        }
        let price_info = self.oracle_price_map.get(&pair_id).unwrap();
        if price_info.valid_timestamp < env::block_timestamp_ms() {
            // oracle price expired
            return U128C::from(0);
        }
        return price_info.price;
    }

    pub fn internal_get_pair_key(base_token: AccountId, quote_token: AccountId) -> String {
        return format!("{}:{}", base_token.clone().to_string(), quote_token.clone().to_string());
    }
}
