use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_contract_standards::fungible_token::events;
use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{AccountId, Balance, env, is_promise_success, log, Promise, PromiseOrValue};
use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{ext_contract, near_bindgen};
use crate::events::emit;

#[near_bindgen]
impl FungibleTokenReceiver for GridBotContract {
    /// Callback on receiving tokens by this contract.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert!(msg.is_empty(), "VALID_TRANSFER_DATA");
        let token_in = env::predecessor_account_id();
        assert!(self.token_map.contains_key(&(token_in.clone())), "VALID_TOKEN");
        let amount: u128 = amount.into();
        log!("Deposit token:{}, amount:{}", token_in.clone(), amount.clone());
        // add amount to user
        self.internal_increase_asset(&sender_id, &token_in, amount.clone());
        // add amount to global
        self.internal_increase_global_asset(&token_in, &(U128C::from(amount)));
        return PromiseOrValue::Value(U128::from(0));
    }
}

impl GridBotContract {
    pub fn internal_ft_transfer(&mut self, account_id: &AccountId, token_id: &AccountId, amount: Balance) -> Promise {
        ext_fungible_token::ft_transfer(
            account_id.clone(),
            amount.into(),
            None,
            token_id.clone(),
            // TODO ONE_YOCTO
            ONE_YOCTO,
            GAS_FOR_FT_TRANSFER,
        )
            .then(ext_self::after_ft_transfer(
                account_id.clone(),
                token_id.clone(),
                amount.into(),
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_FOR_AFTER_FT_TRANSFER,
            ))
    }
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn after_ft_transfer(&mut self, account_id: AccountId, token_id: AccountId, amount: U128)
                         -> bool;
}

trait ExtSelf {
    fn after_ft_transfer(&mut self, account_id: AccountId, token_id: AccountId, amount: U128)
                         -> bool;
}

#[near_bindgen]
impl ExtSelf for GridBotContract {
    #[private]
    fn after_ft_transfer(
        &mut self,
        account_id: AccountId,
        token_id: AccountId,
        amount: U128,
    ) -> bool {
        // TODO confirm transfer failed mean !promise_success
        let promise_success = is_promise_success();
        if !promise_success {
            emit::withdraw_failed(&account_id, amount.0, &token_id);
        } else {
            // reduce asset
            self.internal_reduce_asset(&account_id, &token_id, amount.0);
            emit::withdraw_succeeded(&account_id, amount.clone().0, &token_id);
        }
        promise_success
    }
}
