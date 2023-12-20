use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::storage_management::StorageBalance;
use near_sdk::{AccountId, Balance, env, is_promise_success, log, Promise, PromiseError, PromiseOrValue, require};
use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{ext_contract, near_bindgen};
use crate::events::emit;

#[ext_contract(ext_fungible_token)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn storage_deposit(&mut self, account_id: Option<AccountId>, registration_only: Option<bool>) -> StorageBalance;
}

#[near_bindgen]
impl FungibleTokenReceiver for GridBotContract {
    /// Callback on receiving tokens by this contract.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_in = env::predecessor_account_id();
        if msg.is_empty() {
            self.internal_deposit(&sender_id, &token_in, amount);
            return PromiseOrValue::Value(U128::from(0));
        } else {
            let left = self.internal_parse_take_request(&sender_id, &token_in, amount, msg);
            return PromiseOrValue::Value(left);
        }
    }
}

impl GridBotContract {
    pub fn internal_storage_deposit(&mut self, account_id: &AccountId, token_id: &AccountId, amount: Balance) -> Promise {
        ext_fungible_token::ext(token_id.clone())
            .with_attached_deposit(amount)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .storage_deposit(
                Some(account_id.clone()),
                Some(false),
            ).then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_AFTER_FT_TRANSFER)
                .after_storage_deposit(
                    account_id.clone(),
                    token_id.clone(),
                    amount.into(),
                )
        )
    }

    pub fn internal_ft_transfer(&mut self, account_id: &AccountId, token_id: &AccountId, amount: Balance) -> Promise {
        ext_fungible_token::ext(token_id.clone())
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(
                account_id.clone(),
                amount.into(),
                None,
            ).then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_AFTER_FT_TRANSFER)
                .after_ft_transfer(
                    account_id.clone(),
                    token_id.clone(),
                    amount.into(),
                )
        )
    }
    pub fn internal_ft_transfer_protocol_fee(&mut self, account_id: &AccountId, token_id: &AccountId, amount: Balance) -> Promise {
        ext_fungible_token::ext(token_id.clone())
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(
                account_id.clone(),
                amount.into(),
                None,
            ).then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_AFTER_FT_TRANSFER)
                .after_ft_transfer_protocol_fee(
                    account_id.clone(),
                    token_id.clone(),
                    amount.into(),
                )
        )
    }
    pub fn internal_ft_transfer_unowned_asset(&mut self, account_id: &AccountId, token_id: &AccountId, amount: Balance) -> Promise {
        ext_fungible_token::ext(token_id.clone())
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(
                account_id.clone(),
                amount.into(),
                None,
            ).then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_AFTER_FT_TRANSFER)
                .after_ft_transfer_unowned_asset(
                    account_id.clone(),
                    token_id.clone(),
                    amount.into(),
                )
        )
    }
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn after_storage_deposit(&mut self, account_id: AccountId, token_id: AccountId, amount: U128)
                             -> bool;
    fn after_ft_transfer(&mut self, account_id: AccountId, token_id: AccountId, amount: U128)
                         -> bool;
    fn after_ft_transfer_protocol_fee(&mut self, account_id: AccountId, token_id: AccountId, amount: U128)
                         -> bool;
    fn after_ft_transfer_unowned_asset(&mut self, account_id: AccountId, token_id: AccountId, amount: U128)
                         -> bool;
    fn after_ft_balance_of_for_withdraw_unowned_asset(&mut self, token_id: AccountId, to_user: AccountId, #[callback_result] last_result: Result<U128, PromiseError>);
}

#[near_bindgen]
impl ExtSelf for GridBotContract {

    #[private]
    fn after_storage_deposit(
        &mut self,
        account_id: AccountId,
        token_id: AccountId,
        amount: U128,
    ) -> bool {
        let promise_success = is_promise_success();
        if !promise_success.clone() {
            emit::storage_deposit_failed(&account_id, amount.clone().0, &token_id);
        } else {
            emit::storage_deposit_succeeded(&account_id, amount.clone().0, &token_id);
        }
        promise_success
    }

    #[private]
    fn after_ft_transfer(
        &mut self,
        account_id: AccountId,
        token_id: AccountId,
        amount: U128,
    ) -> bool {
        let promise_success = is_promise_success();
        if !promise_success.clone() {
            emit::withdraw_failed(&account_id, amount.clone().0, &token_id);
            self.internal_increase_withdraw_failed_asset(&account_id, &token_id, &(U256C::from(amount.clone().0)));
        } else {
            emit::withdraw_succeeded(&account_id, amount.clone().0, &token_id);
            // reduce from global asset
            self.internal_reduce_global_asset(&token_id, &(U256C::from(amount.clone().0)))
        }
        promise_success
    }

    #[private]
    fn after_ft_transfer_protocol_fee(
        &mut self,
        account_id: AccountId,
        token_id: AccountId,
        amount: U128,
    ) -> bool {
        let promise_success = is_promise_success();
        if !promise_success.clone() {
            emit::withdraw_protocol_fee_failed(&account_id, amount.clone().0, &token_id);
            self.internal_increase_protocol_fee(&token_id, &(U256C::from(amount.clone().0)));
        } else {
            emit::withdraw_protocol_fee_succeeded(&account_id, amount.clone().0, &token_id);
            // reduce from global asset
            self.internal_reduce_global_asset(&token_id, &(U256C::from(amount.clone().0)))
        }
        promise_success
    }

    #[private]
    fn after_ft_transfer_unowned_asset(
        &mut self,
        account_id: AccountId,
        token_id: AccountId,
        amount: U128,
    ) -> bool {
        let promise_success = is_promise_success();
        if !promise_success.clone() {
            emit::withdraw_unowned_asset_failed(&account_id, amount.0, &token_id);
        } else {
            emit::withdraw_unowned_asset_succeeded(&account_id, amount.clone().0, &token_id);
        }
        promise_success
    }

    #[private]
    fn after_ft_balance_of_for_withdraw_unowned_asset(&mut self, token_id: AccountId, to_user: AccountId, #[callback_result] last_result: Result<U128, PromiseError>) {
        if let Ok(balance) = last_result {
            let recorded_balance = self.internal_get_global_balance(&token_id);
            require!(balance.0 >= recorded_balance.as_u128(), INVALID_BALANCE);
            let can_withdraw_amount = balance.0 - recorded_balance.as_u128();
            self.internal_withdraw_unowned_asset(&to_user, &token_id, U256C::from(can_withdraw_amount));
        } else {
            log!(WITHDRAW_UNOWNED_ASSET_ERROR);
        }
    }
}
