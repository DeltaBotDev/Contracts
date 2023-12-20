use near_contract_standards::storage_management::StorageBalance;
use near_sdk::AccountId;
use near_sdk::json_types::U128;
use serde_json::json;
use workspaces::{Account, Contract};
use workspaces::result::ExecutionFinalResult;
use crate::*;

pub struct FtContractHelper(pub Contract);

impl FtContractHelper {
    pub fn get_account_id(&self) -> AccountId {
        return AccountId::from_str(self.0.id()).expect("Invalid AccountId");
    }

    pub async fn ft_mint(
        &self,
        receiver: &Account,
        amount: u128,
    ) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        self.0
            .call("mint")
            .args_json(json!({
                "account_id": receiver.id(),
                "amount": U128::from(amount),
            }))
            .gas(20_000_000_000_000)
            .deposit(0)
            .transact()
            .await
    }

    pub async fn ft_transfer(
        &self,
        sender: &Account,
        receiver: &Account,
        amount: u128,
    ) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        sender
            .call(self.0.id(), "ft_transfer")
            .args_json(json!({
                "receiver_id": receiver.id(),
                "amount": U128::from(amount),
                "memo": Option::<String>::None,
            }))
            .gas(20_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }

    pub async fn ft_transfer_call(
        &self,
        sender: &Account,
        receiver_id: &AccountId,
        amount: u128,
        msg: String,
    ) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start ft_transfer_call");
        sender
            .call(self.0.id(), "ft_transfer_call")
            .args_json(json!({
                "receiver_id": receiver_id,
                "amount": U128::from(amount),
                "msg": msg.clone(),
            }))
            .gas(300_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }

    pub async fn ft_balance_of(
        &self,
        user: &Account,
    ) -> Result<U128, workspaces::error::Error> {
        self.0
            .call("ft_balance_of")
            .args_json(json!({
                "account_id": user.id()
            }))
            .view()
            .await?
            .json::<U128>()
    }

    pub async fn ft_storage_deposit(
        &self,
        account_id: &AccountId,
    ) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        self.0
            .call("storage_deposit")
            .args_json(json!({
                "account_id": Some(account_id),
                "registration_only": Option::<bool>::None,
            }))
            .gas(20_000_000_000_000)
            .deposit(near_sdk::env::storage_byte_cost() * 125)
            .transact()
            .await
    }

    pub async fn ft_storage_unregister(
        &self,
        account: &Account,
    ) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        account
            .call(self.0.id(), "storage_unregister")
            .args_json(json!({
                "force": Some(true),
            }))
            .gas(20_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }

    pub async fn get_storage_balance_of(
        &self,
        user: &Account,
    ) -> Result<Option<StorageBalance>, workspaces::error::Error> {
        self.0
            .call("storage_balance_of")
            .args_json(json!({
                "account_id": user.id()
            }))
            .view()
            .await?
            .json::<Option<StorageBalance>>()
    }
}
