
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{ext_contract, near_bindgen, AccountId, PanicOnDefault, Promise, Balance, Gas, PromiseResult, log};
use near_sdk::collections::LookupMap;
use near_sdk::env;
use near_sdk::json_types::U128;

const GAS_FOR_FT_ON_TRANSFER: Gas = Gas(Gas::ONE_TERA.0 * 10);
const NO_DEPOSIT: Balance = 0;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FtContract {
    balances: LookupMap<AccountId, u128>,
    total_supply: u128,
    name: String,
    symbol: String,
    decimals: u8,
}

#[ext_contract(ext_ft_receiver)]
pub trait FTReceiver {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> Promise;
}

#[near_bindgen]
impl FtContract {
    #[init]
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        Self {
            balances: LookupMap::new(b"b".to_vec()),
            total_supply: 0,
            name,
            symbol,
            decimals,
        }
    }

    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        let balance = self.balances.get(&account_id).unwrap_or(0);
        self.balances.insert(&account_id, &(balance + amount.0));
        self.total_supply += amount.0;
    }

    #[payable]
    pub fn transfer(&mut self, recipient: AccountId, amount: u128) {
        let sender_id = env::predecessor_account_id();
        let sender_balance = self.balances.get(&sender_id).expect("Balance not found");
        assert!(amount <= sender_balance, "Not enough balance");

        let recipient_balance = self.balances.get(&recipient).unwrap_or(0);
        self.balances.insert(&sender_id, &(sender_balance - amount));
        self.balances.insert(&recipient, &(recipient_balance + amount));
    }

    #[payable]
    pub fn ft_transfer_call(&mut self, recipient: AccountId, amount: U128, msg: String) -> Promise {
        // log!("sender_id".to_string() + &env::predecessor_account_id().to_string());
        assert!(self.balances.get(&env::predecessor_account_id()).unwrap_or(0) >= amount.0, "Not enough balance");

        let sender_id = env::predecessor_account_id();
        let sender_balance = self.balances.get(&sender_id).expect("Balance not found");
        let recipient_balance = self.balances.get(&recipient).unwrap_or(0);
        self.balances.insert(&sender_id, &(sender_balance - amount.0));
        self.balances.insert(&recipient, &(recipient_balance + amount.0));

        // Self::ext(env::current_account_id())
        //     .with_static_gas(GAS_FOR_FT_ON_TRANSFER)
        //     .ft_on_transfer(sender_id,
        //                     amount,
        //                     msg)

        // ext_ft_receiver::ft_on_transfer(
        //     sender_id,
        //     amount,
        //     msg
        // ).deposit(NO_DEPOSIT)
        //     .gas(GAS_FOR_FT_ON_TRANSFER)

        ext_ft_receiver::ext(recipient.clone())
            .with_attached_deposit(NO_DEPOSIT)
            .with_static_gas(GAS_FOR_FT_ON_TRANSFER)
            .ft_on_transfer(
                sender_id.clone(),
                amount.clone(),
                msg,
            )
    }
    pub fn get_balance(&self, account_id: AccountId) -> U128 {
        U128::from(self.balances.get(&account_id).unwrap_or(0))
    }

    pub fn get_total_supply(&self) -> U128 {
        U128::from(self.total_supply)
    }
}

