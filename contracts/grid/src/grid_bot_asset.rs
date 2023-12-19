use near_sdk::{AccountId, Balance};
use near_sdk::collections::LookupMap;
use crate::{GridBot, GridBotContract, StorageKey, U256C};
use crate::entity::Pair;
use crate::events::emit;

impl GridBotContract {
    // ############################### Increase or Reduce Asset ####################################
    pub fn internal_reduce_asset(&mut self, user: &AccountId, token: &AccountId, amount: &U256C) {
        let mut user_balances = self.user_balances_map.get(user).unwrap_or_else(|| {
            let mut map = LookupMap::new(StorageKey::UserBalanceSubKey(user.clone()));
            map.insert(token, &U256C::from(0));
            map
        });

        let balance = user_balances.get(token).unwrap_or(U256C::from(0));
        user_balances.insert(token, &(balance - amount));

        self.user_balances_map.insert(user, &user_balances);
    }

    pub fn internal_increase_asset(&mut self, user: &AccountId, token: &AccountId, amount: &U256C) {
        let mut user_balances = self.user_balances_map.get(user).unwrap_or_else(|| {
            let mut map = LookupMap::new(StorageKey::UserBalanceSubKey(user.clone()));
            map.insert(token, &U256C::from(0));
            map
        });

        let balance = user_balances.get(token).unwrap_or(U256C::from(0));
        user_balances.insert(token, &(balance + amount));

        self.user_balances_map.insert(user, &user_balances);
    }

    pub fn internal_reduce_withdraw_failed_asset(&mut self, user: &AccountId, token: &AccountId, amount: &U256C) {
        let mut user_balances = self.user_withdraw_failed_map.get(user).unwrap_or_else(|| {
            let mut map = LookupMap::new(StorageKey::WithdrawFailedSubKey(user.clone()));
            map.insert(token, &U256C::from(0));
            map
        });

        let balance = user_balances.get(token).unwrap_or(U256C::from(0));
        user_balances.insert(token, &(balance - amount));

        self.user_withdraw_failed_map.insert(user, &user_balances);
    }

    pub fn internal_increase_withdraw_failed_asset(&mut self, user: &AccountId, token: &AccountId, amount: &U256C) {
        let mut user_balances = self.user_withdraw_failed_map.get(user).unwrap_or_else(|| {
            let mut map = LookupMap::new(StorageKey::WithdrawFailedSubKey(user.clone()));
            map.insert(token, &U256C::from(0));
            map
        });

        let balance = user_balances.get(token).unwrap_or(U256C::from(0));
        user_balances.insert(token, &(balance + amount));

        self.user_withdraw_failed_map.insert(user, &user_balances);
    }

    pub fn internal_increase_global_asset(&mut self, token: &AccountId, amount: &U256C) {
        let balance = self.global_balances_map.get(token).unwrap();
        let new_balance = balance + amount;
        self.global_balances_map.insert(token, &new_balance);
    }

    pub fn internal_reduce_global_asset(&mut self, token: &AccountId, amount: &U256C) {
        let balance = self.global_balances_map.get(token).unwrap();
        let new_balance = balance - amount;
        self.global_balances_map.insert(token, &new_balance);
    }

    pub fn internal_increase_protocol_fee(&mut self, token: &AccountId, amount: &U256C) {
        let balance = self.protocol_fee_map.get(token).unwrap();
        let new_balance = balance + amount;
        self.protocol_fee_map.insert(token, &new_balance);
    }

    pub fn internal_reduce_protocol_fee(&mut self, token: &AccountId, amount: &U256C) {
        let balance = self.protocol_fee_map.get(token).unwrap();
        let new_balance = balance - amount;
        self.protocol_fee_map.insert(token, &new_balance);
    }

    pub fn internal_increase_locked_assets(&mut self, user: &AccountId, token: &AccountId, amount: &U256C) {
        if *amount == U256C::from(0) {
            return;
        }
        // let user_locked_balances = self.user_locked_balances_map.get_mut(&user).unwrap();
        // let locked_balance = user_locked_balances.get_mut(&token).unwrap();
        // *locked_balance += *amount;

        let mut user_locked_balances = self.user_locked_balances_map.get(user).unwrap_or_else(|| {
            let mut map = LookupMap::new(StorageKey::UserLockedBalanceSubKey(user.clone()));
            map.insert(token, &U256C::from(0));
            map
        });

        let balance = user_locked_balances.get(token).unwrap_or(U256C::from(0));
        user_locked_balances.insert(token, &(balance + amount));

        self.user_locked_balances_map.insert(user, &user_locked_balances);
    }

    pub fn internal_reduce_locked_assets(&mut self, user: &AccountId, token: &AccountId, amount: &U256C) {
        if *amount == U256C::from(0) {
            return;
        }
        // let user_locked_balances = self.user_locked_balances_map.get_mut(&user).unwrap();
        // let locked_balance = user_locked_balances.get_mut(&token).unwrap();
        // *locked_balance -= *amount;

        let mut user_locked_balances = self.user_locked_balances_map.get(user).unwrap_or_else(|| {
            let mut map = LookupMap::new(StorageKey::UserLockedBalanceSubKey(user.clone()));
            map.insert(token, &U256C::from(0));
            map
        });

        let balance = user_locked_balances.get(token).unwrap_or(U256C::from(0));
        user_locked_balances.insert(token, &(balance - amount));

        self.user_locked_balances_map.insert(user, &user_locked_balances);
    }

    //################################### Asset Transfer ###########################################
    pub fn internal_transfer_assets_to_lock(&mut self, user: AccountId, token: AccountId, amount: U256C) {
        self.internal_reduce_asset(&user, &token, &amount);

        self.internal_increase_locked_assets(&user, &token, &amount)
    }

    pub fn internal_transfer_assets_to_unlock(&mut self, user: &AccountId, token: &AccountId, amount: U256C) {
        if amount == U256C::from(0) {
            return;
        }
        // let user_balances = self.user_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        // let balance = user_balances.entry(token.clone()).or_insert(U256C::from(0));
        // *balance += amount.clone();
        self.internal_increase_asset(user, token, &amount);

        // let user_locked_balances = self.user_locked_balances_map.entry(user.clone()).or_insert_with(HashMap::new);
        // let locked_balance = user_locked_balances.entry(token.clone()).or_insert(U256C::from(0));
        // *locked_balance -= amount;

        self.internal_reduce_asset(user, token, &amount);
    }

    pub fn internal_add_protocol_fee(&mut self, bot: &mut GridBot, token: &AccountId, fee: U256C, pair: &Pair) {
        if !self.protocol_fee_map.contains_key(token) {
            self.protocol_fee_map.insert(&token, &U256C::from(0));
        }
        // let bot_mut = self.bot_map.get_mut(&bot_id).unwrap();
        // let user = bot_mut.user.clone();
        let user = bot.user.clone();
        // reduce bot's asset
        if *token == pair.base_token {
            // bot_mut.total_base_amount -= U256C::from(fee.clone());
            bot.total_base_amount -= U256C::from(fee.clone());
        } else {
            // bot_mut.total_quote_amount -= U256C::from(fee.clone());
            bot.total_quote_amount -= U256C::from(fee.clone());
        }
        // reduce user's lock asset
        self.internal_reduce_locked_assets(&user, &token, &(U256C::from(fee.clone())));
        // add into protocol fee map
        self.internal_increase_protocol_fee(token, &(U256C::from(fee.clone())));
    }

    pub fn internal_update_bot_asset(bot: &mut GridBot, pair: &Pair, token_sell: AccountId, amount_sell: Balance, amount_buy: Balance) {
        if pair.base_token == token_sell {
            bot.total_base_amount = bot.total_base_amount.checked_sub(U256C::from(amount_sell)).expect("Base amount underflow");
            bot.total_quote_amount = bot.total_quote_amount.checked_add(U256C::from(amount_buy)).expect("Quote amount overflow");
        } else {
            bot.total_base_amount = bot.total_base_amount.checked_add(U256C::from(amount_buy)).expect("Base amount overflow");
            bot.total_quote_amount = bot.total_quote_amount.checked_sub(U256C::from(amount_sell)).expect("Quote amount underflow");
        }
    }

    //########################################## bot revenue #######################################
    pub fn internal_remove_revenue_from_bot(&mut self, bot: &mut GridBot) {
        if bot.fill_base_or_quote {
            // self.bot_map.get_mut(&(bot.bot_id)).unwrap().total_quote_amount -= bot.revenue.clone();
            bot.total_quote_amount -= bot.revenue.clone();
        } else {
            // self.bot_map.get_mut(&(bot.bot_id)).unwrap().total_base_amount -= bot.revenue.clone();
            bot.total_base_amount -= bot.revenue.clone();
        }
    }

    pub fn internal_harvest_revenue(&mut self, bot: &mut GridBot, pair: &Pair) -> (AccountId, U256C) {
        let revenue_token = if bot.fill_base_or_quote {
            pair.quote_token.clone()
        } else {
            pair.base_token.clone()
        };
        let revenue = bot.revenue.clone();
        // transfer out from bot asset
        self.internal_remove_revenue_from_bot(bot);
        // transfer to available asset
        self.internal_increase_asset(&(bot.user.clone()), &revenue_token, &(U256C::from(revenue.clone())));
        // sign to 0
        // self.bot_map.get_mut(&(bot.bot_id)).unwrap().revenue = U256C::from(0);
        bot.revenue = U256C::from(0);
        return (revenue_token, U256C::from(revenue));
    }

    //################################## Withdraw ##################################################
    pub fn internal_withdraw(&mut self, user: &AccountId, token: &AccountId, amount: U256C) {
        // reduce user asset
        self.internal_reduce_asset(user, token, &amount);
        // start transfer
        self.internal_ft_transfer(&user, &token, amount.as_u128());
        emit::withdraw_started(&user, amount.as_u128(), &token);
    }

    pub fn internal_withdraw_protocol_fee(&mut self, user: &AccountId, token: &AccountId, amount: U256C) {
        // reduce protocol
        self.internal_reduce_protocol_fee(token, &(amount.clone()));
        // start transfer
        self.internal_ft_transfer_protocol_fee(&user, &token, amount.as_u128());
        emit::withdraw_protocol_fee_started(&user, amount.as_u128(), &token);
    }

    pub fn internal_withdraw_unowned_asset(&mut self, user: &AccountId, token: &AccountId, amount: U256C) {
        self.internal_ft_transfer_unowned_asset(&user, &token, amount.as_u128());
        emit::withdraw_unowned_asset_started(&user, amount.as_u128(), &token);
    }
}
