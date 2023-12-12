use std::collections::HashMap;
use near_sdk::{AccountId, Balance};
use crate::{GridBot, GridBotContract, U128C};
use crate::entity::Pair;
use crate::events::emit;

impl GridBotContract {
    // ############################### Increase or Reduce Asset ####################################
    pub fn internal_reduce_asset(&mut self, user: &AccountId, token: &AccountId, amount: &U128C) {
        if *amount == U128C::from(0) {
            return;
        }
        let user_balances = self.user_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        let balance = user_balances.entry(token.clone()).or_insert(U128C::from(0));
        *balance -= *amount;
    }

    pub fn internal_increase_asset(&mut self, user: &AccountId, token: &AccountId, amount: &U128C) {
        if *amount == U128C::from(0) {
            return;
        }
        let user_balances = self.user_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        let balance = user_balances.entry(token.clone()).or_insert(U128C::from(0));
        *balance += *amount;
    }

    pub fn internal_increase_global_asset(&mut self, token: &AccountId, amount: &U128C) {
        let balance = self.global_balances_map.get_mut(token).unwrap();
        *balance += *amount;
    }

    pub fn internal_reduce_global_asset(&mut self, token: &AccountId, amount: &U128C) {
        let balance = self.global_balances_map.get_mut(token).unwrap();
        *balance -= *amount;
    }

    pub fn internal_increase_protocol_fee(&mut self, token: &AccountId, amount: &U128C) {
        let balance = self.protocol_fee_map.get_mut(token).unwrap();
        *balance += *amount;
    }

    pub fn internal_reduce_protocol_fee(&mut self, token: &AccountId, amount: &U128C) {
        let balance = self.protocol_fee_map.get_mut(token).unwrap();
        *balance -= *amount;
    }

    pub fn internal_increase_locked_assets(&mut self, user: &AccountId, token: &AccountId, amount: &U128C) {
        if *amount == U128C::from(0) {
            return;
        }
        let user_locked_balances = self.user_locked_balances_map.get_mut(&user).unwrap();
        let locked_balance = user_locked_balances.get_mut(&token).unwrap();
        *locked_balance += *amount;
    }

    pub fn internal_reduce_locked_assets(&mut self, user: &AccountId, token: &AccountId, amount: &U128C) {
        if *amount == U128C::from(0) {
            return;
        }
        let user_locked_balances = self.user_locked_balances_map.get_mut(&user).unwrap();
        let locked_balance = user_locked_balances.get_mut(&token).unwrap();
        *locked_balance -= *amount;
    }

    //################################### Asset Transfer ###########################################
    pub fn internal_transfer_assets_to_lock(&mut self, user: AccountId, token: AccountId, amount: U128C) {
        if amount == U128C::from(0) {
            return;
        }
        let user_balances = self.user_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        let balance = user_balances.entry(token.clone()).or_insert(U128C::from(0));
        *balance -= amount;

        let user_locked_balances = self.user_locked_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        let locked_balance = user_locked_balances.entry(token.clone()).or_insert(U128C::from(0));
        *locked_balance += amount;
    }

    pub fn internal_transfer_assets_to_unlock(&mut self, user: &AccountId, token: &AccountId, amount: U128C) {
        if amount == U128C::from(0) {
            return;
        }
        let user_balances = self.user_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        let balance = user_balances.entry(token.clone()).or_insert(U128C::from(0));
        *balance += amount.clone();

        let user_locked_balances = self.user_locked_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        let locked_balance = user_locked_balances.entry(token.clone()).or_insert(U128C::from(0));
        *locked_balance -= amount;
    }

    pub fn internal_add_protocol_fee(&mut self, token: &AccountId, fee: Balance, bot_id: String, pair: &Pair) {
        if !self.protocol_fee_map.contains_key(token) {
            self.protocol_fee_map.insert(token.clone(), U128C::from(0));
        }
        let bot_mut = self.bot_map.get_mut(&bot_id).unwrap();
        let user = bot_mut.user.clone();
        // reduce bot's asset
        if *token == pair.base_token {
            bot_mut.total_base_amount -= U128C::from(fee.clone());
        } else {
            bot_mut.total_quote_amount -= U128C::from(fee.clone());
        }
        // reduce user's lock asset
        self.internal_reduce_locked_assets(&user, &token, &(U128C::from(fee.clone())));
        // add into protocol fee map
        self.internal_increase_protocol_fee(token, &(U128C::from(fee.clone())));
    }

    pub fn internal_update_bot_asset(bot: &mut GridBot, pair: &Pair, token_sell: AccountId, amount_sell: Balance, amount_buy: Balance) {
        if pair.base_token == token_sell {
            bot.total_base_amount = bot.total_base_amount.checked_sub(U128C::from(amount_sell)).expect("Base amount underflow");
            bot.total_quote_amount = bot.total_quote_amount.checked_add(U128C::from(amount_buy)).expect("Quote amount overflow");
        } else {
            bot.total_base_amount = bot.total_base_amount.checked_add(U128C::from(amount_buy)).expect("Base amount overflow");
            bot.total_quote_amount = bot.total_quote_amount.checked_sub(U128C::from(amount_sell)).expect("Quote amount underflow");
        }
    }

    //########################################## bot revenue #######################################
    pub fn internal_remove_revenue_from_bot(&mut self, bot: &GridBot) {
        if bot.fill_base_or_quote {
            self.bot_map.get_mut(&(bot.bot_id)).unwrap().total_base_amount -= bot.revenue.clone();
        } else {
            self.bot_map.get_mut(&(bot.bot_id)).unwrap().total_quote_amount -= bot.revenue.clone();
        }
    }

    pub fn internal_harvest_revenue(&mut self, bot: &GridBot, pair: &Pair, user: &AccountId) -> (AccountId, U128C) {
        let revenue_token = if bot.fill_base_or_quote {
            pair.base_token.clone()
        } else {
            pair.quote_token.clone()
        };
        let revenue = bot.revenue.clone();
        // transfer out from bot asset
        self.internal_remove_revenue_from_bot(&bot);
        // transfer to available asset
        self.internal_increase_asset(&user, &revenue_token, &(U128C::from(revenue.clone())));
        // sign to 0
        self.bot_map.get_mut(&(bot.bot_id)).unwrap().revenue = U128C::from(0);
        return (revenue_token, U128C::from(revenue));
    }

    //################################## Withdraw ##################################################
    pub fn internal_withdraw(&mut self, user: &AccountId, token: &AccountId, amount: U128C) {
        // reduce user asset
        self.internal_reduce_asset(user, token, &amount);
        // start transfer
        self.internal_ft_transfer(&user, &token, amount.as_u128());
        emit::withdraw_started(&user, amount.as_u128(), &token);
    }

    pub fn internal_withdraw_protocol_fee(&mut self, user: &AccountId, token: &AccountId, amount: U128C) {
        // reduce protocol
        self.internal_reduce_protocol_fee(token, &(amount.clone()));
        // start transfer
        self.internal_ft_transfer(&user, &token, amount.as_u128());
        emit::withdraw_protocol_fee_started(&user, amount.as_u128(), &token);
    }

    pub fn internal_withdraw_unowned_asset(&mut self, user: &AccountId, token: &AccountId, amount: U128C) {
        self.internal_ft_transfer_unowned_asset(&user, &token, amount.as_u128());
        emit::withdraw_unowned_asset_started(&user, amount.as_u128(), &token);
    }
}
