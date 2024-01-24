use near_sdk::{AccountId, Balance, require};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use crate::{GridBot, GridBotContract, StorageKey, TakeRequest, U256C};
use crate::entity::Pair;
use crate::events::emit;
use crate::errors::*;

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

    // pub fn internal_reduce_withdraw_failed_asset(&mut self, user: &AccountId, token: &AccountId, amount: &U256C) {
    //     let mut user_balances = self.user_withdraw_failed_map.get(user).unwrap_or_else(|| {
    //         let mut map = LookupMap::new(StorageKey::WithdrawFailedSubKey(user.clone()));
    //         map.insert(token, &U256C::from(0));
    //         map
    //     });
    //
    //     let balance = user_balances.get(token).unwrap_or(U256C::from(0));
    //     user_balances.insert(token, &(balance - amount));
    //
    //     self.user_withdraw_failed_map.insert(user, &user_balances);
    // }

    // pub fn internal_increase_withdraw_failed_asset(&mut self, user: &AccountId, token: &AccountId, amount: &U256C) {
    //     let mut user_balances = self.user_withdraw_failed_map.get(user).unwrap_or_else(|| {
    //         let mut map = LookupMap::new(StorageKey::WithdrawFailedSubKey(user.clone()));
    //         map.insert(token, &U256C::from(0));
    //         map
    //     });
    //
    //     let balance = user_balances.get(token).unwrap_or(U256C::from(0));
    //     user_balances.insert(token, &(balance + amount));
    //
    //     self.user_withdraw_failed_map.insert(user, &user_balances);
    // }

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

        self.internal_reduce_locked_assets(user, token, &amount);
    }

    pub fn internal_add_protocol_fee_from_revenue(&mut self, bot: &mut GridBot, token: &AccountId, fee: U256C, pair: &Pair) {
        // if !self.protocol_fee_map.contains_key(token) {
        //     self.protocol_fee_map.insert(&token, &U256C::from(0));
        // }
        // let bot_mut = self.bot_map.get_mut(&bot_id).unwrap();
        // let user = bot_mut.user.clone();
        let user = bot.user.clone();
        // reduce bot's asset
        if *token == pair.base_token {
            // bot_mut.total_base_amount -= U256C::from(fee.clone());
            bot.total_base_amount -= fee.clone();
        } else {
            // bot_mut.total_quote_amount -= U256C::from(fee.clone());
            bot.total_quote_amount -= fee.clone();
        }
        // reduce user's lock asset
        self.internal_reduce_locked_assets(&user, &token, &(fee.clone()));
        // add into protocol fee map
        self.internal_increase_protocol_fee(token, &(fee.clone()));
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

    pub fn internal_deposit(&mut self, sender_id: &AccountId, token_in: &AccountId, amount: U128) -> bool {
        require!(self.global_balances_map.contains_key(token_in), INVALID_TOKEN);
        // require min deposit
        // require!(amount.clone().0 >= self.deposit_limit_map.get(token_in).unwrap().as_u128(), LESS_DEPOSIT_AMOUNT);
        if amount.clone().0 < self.deposit_limit_map.get(token_in).unwrap().as_u128() {
            self.internal_token_refund(sender_id, token_in, LESS_DEPOSIT_AMOUNT);
            emit::deposit_failed(sender_id, amount.clone().0, token_in);
            return false;
        }
        // log!("Deposit user:{}, token:{}, amount:{}", sender_id.clone(), token_in.clone(), amount.clone().0);
        // add amount to user
        self.internal_increase_asset(sender_id, token_in, &(U256C::from(amount.clone().0)));
        // add amount to global
        self.internal_increase_global_asset(token_in, &(U256C::from(amount.clone().0)));
        // event
        emit::deposit_success(sender_id, amount.clone().0, token_in);
        return true;
    }

    pub fn internal_parse_take_request(&mut self, sender_id: &AccountId, token_in: &AccountId, amount: U128, msg: String) -> U128 {
        let take_request = serde_json::from_str::<TakeRequest>(&msg).expect(INVALID_TAKE_PARAM);
        // deposit first
        if !self.internal_deposit(sender_id, token_in, amount) {
            return amount;
        }
        // require
        require!(token_in.clone() == take_request.take_order.token_sell, INVALID_TOKEN);
        require!(amount.clone().0 >= take_request.take_order.amount_sell.0, INVALID_ORDER_AMOUNT);
        // take
        let taker_order = take_request.take_order.to_order();
        let (took_sell, took_buy) = self.internal_take_orders(sender_id, &taker_order, take_request.maker_orders);
        // reduce left
        let left = amount.0 - took_sell.as_u128();
        if left.clone() > 0 {
            // add amount to user
            self.internal_reduce_asset(sender_id, token_in, &(U256C::from(left.clone())));
            // add amount to global
            self.internal_reduce_global_asset(token_in, &(U256C::from(left.clone())));
        }
        // event
        emit::deposit_return_success(sender_id, left.clone(), token_in);
        // withdraw for taker
        self.internal_withdraw(sender_id, &(taker_order.token_buy), took_buy);
        return U128::from(left);
    }

    //################################## Withdraw ##################################################
    pub fn internal_withdraw_all(&mut self, user: &AccountId, token: &AccountId) {
        let balance = self.internal_get_user_balance(&user, &token);
        self.internal_withdraw(&user, &token, balance);
    }

    pub fn internal_withdraw(&mut self, user: &AccountId, token: &AccountId, amount: U256C) {
        if amount.as_u128() == 0 {
            return;
        }
        // reduce user asset
        self.internal_reduce_asset(user, token, &amount);
        if token.clone() == self.wnear {
            // wrap to near
            self.withdraw_near(&user, amount.as_u128());
        } else {
            // start transfer
            self.internal_ft_transfer(&user, &token, amount.as_u128());
        }
        emit::withdraw_started(&user, amount.as_u128(), &token);
    }

    pub fn internal_withdraw_protocol_fee(&mut self, user: &AccountId, token: &AccountId, amount: U256C) {
        if amount.as_u128() == 0 {
            return;
        }
        // reduce protocol
        self.internal_reduce_protocol_fee(token, &(amount.clone()));
        // start transfer
        self.internal_ft_transfer_protocol_fee(&user, &token, amount.as_u128());
        emit::withdraw_protocol_fee_started(&user, amount.as_u128(), &token);
    }

    pub fn internal_create_bot_refund(&mut self, user: &AccountId, pair: &Pair, reason: &str) {
        self.internal_withdraw_all(user, &pair.base_token);
        self.internal_withdraw_all(user, &pair.quote_token);
        emit::create_bot_error(user, reason);
    }

    pub fn internal_token_refund(&mut self, user: &AccountId, token: &AccountId, reason: &str) {
        self.internal_withdraw_all(user, token);
        emit::create_bot_error(user, reason);
    }

    // pub fn internal_withdraw_unowned_asset(&mut self, user: &AccountId, token: &AccountId, amount: U256C) {
    //     self.internal_ft_transfer_unowned_asset(&user, &token, amount.as_u128());
    //     emit::withdraw_unowned_asset_started(&user, amount.as_u128(), &token);
    // }
}
