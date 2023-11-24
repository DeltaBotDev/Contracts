use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::{AccountId, Balance, env, log, PromiseOrValue};
use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{near_bindgen};

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
        log!("Deposit token:{}, amount:{}", token_in.clone(), amount);
        // initial user
        if !self.user_balances_map.contains_key(&(sender_id.clone())) {
            self.user_balances_map.insert(sender_id.clone(), Default::default());
        }
        // initial user balance
        let user_balances = self.user_balances_map.get_mut(&(sender_id.clone())).unwrap();
        if !user_balances.contains_key(&(token_in.clone())) {
            user_balances.insert(token_in.clone(), U128C::from(0));
        }
        // add user balance
        *(user_balances.get_mut(&(token_in.clone())).unwrap()) = user_balances.get(&(token_in.clone())).unwrap() + U128C::from(amount);
        return PromiseOrValue::Value(U128::from(0));
    }
}
