use near_sdk::{AccountId, env, Gas};
use near_sdk::json_types::U128;
use crate::{GridBot, GridBotContract, GridBotOutput, Pair, U256C};

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

    pub fn internal_get_grid_bot_output(&self, grid: &GridBot) -> GridBotOutput {
        GridBotOutput {
            name: grid.name.clone(),
            active: grid.active.clone(),
            user: grid.user.clone(),
            bot_id: grid.bot_id.clone(),
            closed: grid.closed.clone(),
            pair_id: grid.pair_id.clone(),
            grid_type: grid.grid_type.clone(),
            grid_sell_count: grid.grid_sell_count.clone(),
            grid_buy_count: grid.grid_buy_count.clone(),
            grid_rate: grid.grid_rate.clone(),
            grid_offset: U128::from(grid.grid_offset.as_u128()),
            first_base_amount: U128::from(grid.first_base_amount.as_u128()),
            first_quote_amount: U128::from(grid.first_quote_amount.as_u128()),
            last_base_amount: U128::from(grid.last_base_amount.as_u128()),
            last_quote_amount: U128::from(grid.last_quote_amount.as_u128()),
            fill_base_or_quote: grid.fill_base_or_quote.clone(),
            trigger_price: U128::from(grid.trigger_price.as_u128()),
            trigger_price_above_or_below: grid.trigger_price_above_or_below.clone(),
            take_profit_price: U128::from(grid.take_profit_price.as_u128()),
            stop_loss_price: U128::from(grid.stop_loss_price.as_u128()),
            valid_until_time: U128::from(grid.valid_until_time.as_u128()),
            total_quote_amount: U128::from(grid.total_quote_amount.as_u128()),
            total_base_amount: U128::from(grid.total_base_amount.as_u128()),
            revenue: U128::from(grid.revenue.as_u128()),
            total_revenue: U128::from(grid.total_revenue.as_u128()),
        }
    }
}
