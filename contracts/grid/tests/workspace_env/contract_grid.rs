use near_sdk::AccountId;
use near_units::parse_near;
use serde_json::json;
use workspaces::{Account, Contract};
use workspaces::result::ExecutionFinalResult;
use grid::{GridBot, GridType, Order, OrderKeyInfo, OrderResult, U256C};
use crate::*;

pub struct GridBotHelper(pub Contract);

impl GridBotHelper {
    pub async fn storage_deposit(&self, account: &Account) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start storage_deposit");
        self.0
            .call("storage_deposit")
            .args_json(json!({
                "account_id": Some(account.id()),
                "registration_only": Option::<bool>::None,
            }))
            .gas(20_000_000_000_000)
            .deposit(parse_near!("1 N"))
            .transact()
            .await
    }

    pub async fn deposit(&self, token_contract: &FtContractHelper, caller: &Account, amount: u128) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start deposit");
        return token_contract.ft_transfer_call(caller, &(AccountId::from_str(self.0.id()).expect("Invalid AccountId")), amount, "".to_string()).await;
    }

    pub async fn register_pair(&self, caller: &Account, base_token: &AccountId, quote_token: &AccountId) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start register_pair");
        caller
            .call(self.0.id(), "register_pair")
            .args_json(json!({
                "base_token": *base_token,
                "quote_token": *quote_token,
            }))
            .gas(300_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }

    pub async fn set_oracle_price(&self, caller: &Account, price: &U256C, pair_id: String) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start set_oracle_price");
        caller
            .call(self.0.id(), "set_oracle_price")
            .args_json(json!({
                "price": *price,
                "pair_id": pair_id,
            }))
            .gas(300_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }

    pub async fn create_bot(&self, caller: &Account, pair_id: String, slippage: u16, grid_type: GridType, grid_rate: u16, grid_offset: U256C, first_base_amount: U256C, first_quote_amount: U256C,
                            last_base_amount: U256C, last_quote_amount: U256C, fill_base_or_quote: bool, grid_sell_count: u16, grid_buy_count: u16,
                            trigger_price: U256C, take_profit_price: U256C, stop_loss_price: U256C, valid_until_time: U256C,
                            entry_price: U256C) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start create_bot");
        caller
            .call(self.0.id(), "create_bot")
            .args_json(json!({
                "pair_id": pair_id,
                "slippage": slippage,
                "grid_type": grid_type,
                "grid_rate": grid_rate,
                "grid_offset": grid_offset,
                "first_base_amount": first_base_amount,
                "first_quote_amount": first_quote_amount,
                "last_base_amount": last_base_amount,
                "last_quote_amount": last_quote_amount,
                "fill_base_or_quote": fill_base_or_quote,
                "grid_sell_count": grid_sell_count,
                "grid_buy_count": grid_buy_count,
                "trigger_price": trigger_price,
                "take_profit_price": take_profit_price,
                "stop_loss_price": stop_loss_price,
                "valid_until_time": valid_until_time,
                "entry_price": entry_price,
            }))
            .gas(300_000_000_000_000)
            .deposit(10_000_000_000_000_000_000_000)
            .transact()
            .await
    }

    pub async fn take_orders(&self, caller: &Account, order: &Order, maker_orders: Vec<OrderKeyInfo>) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start take_orders");
        caller
            .call(self.0.id(), "take_orders")
            .args_json(json!({
                "take_order": order,
                "maker_orders": maker_orders,
            }))
            .gas(300_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }

    pub async fn claim(&self, caller: &Account, bot_id: String) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start claim");
        caller
            .call(self.0.id(), "claim")
            .args_json(json!({
                "bot_id": bot_id,
            }))
            .gas(300_000_000_000_000)
            // .deposit(1)
            .transact()
            .await
    }

    pub async fn close_bot(&self, caller: &Account, bot_id: String) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start close_bot");
        caller
            .call(self.0.id(), "close_bot")
            .args_json(json!({
                "bot_id": bot_id,
            }))
            .gas(300_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }

    pub async fn trigger_bot(&self, caller: &Account, bot_id: String) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start trigger_bot");
        caller
            .call(self.0.id(), "trigger_bot")
            .args_json(json!({
                "bot_id": bot_id,
            }))
            .gas(300_000_000_000_000)
            // .deposit(1)
            .transact()
            .await
    }

    pub async fn withdraw(&self, caller: &Account, token: AccountId) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start withdraw");
        caller
            .call(self.0.id(), "withdraw")
            .args_json(json!({
                "token": token,
            }))
            .gas(300_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }

    // ####################################### Owner
    pub async fn withdraw_protocol_fee(&self, caller: &Account, token: AccountId, to_user: AccountId, amount: U256C) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start withdraw_protocol_fee");
        caller
            .call(self.0.id(), "withdraw_protocol_fee")
            .args_json(json!({
                "token": token,
                "to_user": to_user,
                "amount": amount,
            }))
            .gas(300_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }
    pub async fn withdraw_unowned_asset(&self, caller: &Account, token: AccountId, to_user: AccountId) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start withdraw_unowned_asset");
        caller
            .call(self.0.id(), "withdraw_unowned_asset")
            .args_json(json!({
                "token": token,
                "to_user": to_user,
            }))
            .gas(300_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }
    pub async fn pause(&self, caller: &Account) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start pause");
        caller
            .call(self.0.id(), "pause")
            // .args_json(json!({
            //     "token": token,
            // }))
            .gas(300_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }
    pub async fn start(&self, caller: &Account) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start start");
        caller
            .call(self.0.id(), "start")
            // .args_json(json!({
            //     "token": token,
            // }))
            .gas(300_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }
    pub async fn shutdown(&self, caller: &Account) -> Result<ExecutionFinalResult, workspaces::error::Error> {
        log!("start shutdown");
        caller
            .call(self.0.id(), "shutdown")
            // .args_json(json!({
            //     "token": token,
            // }))
            .gas(300_000_000_000_000)
            .deposit(1)
            .transact()
            .await
    }

}

impl GridBotHelper {
    pub async fn query_order(&self, bot_id: String, forward_or_reverse: bool, level: usize) -> Result<Option<OrderResult>, workspaces::error::Error> {
        log!("start query_order");
        self.0
            .call("query_order")
            .args_json(json!({
                "bot_id": bot_id,
                "forward_or_reverse": forward_or_reverse,
                "level": level,
            }))
            .view()
            .await?
            .json::<Option<OrderResult>>()
    }

    pub async fn query_orders(&self, bot_ids: Vec<String>, forward_or_reverses: Vec<bool>, levels: Vec<usize>) -> Result<Option<Vec<Order>>, workspaces::error::Error> {
        log!("start query_orders");
        self.0
            .call("query_orders")
            .args_json(json!({
                "bot_ids": bot_ids,
                "forward_or_reverses": forward_or_reverses,
                "levels": levels,
            }))
            .view()
            .await?
            .json::<Option<Vec<Order>>>()
    }

    pub async fn query_bot(&self, bot_id: String) -> Result<Option<GridBot>, workspaces::error::Error> {
        log!("start query_bot");
        self.0
            .call("query_bot")
            .args_json(json!({
                "bot_id": bot_id,
            }))
            .view()
            .await?
            .json::<Option<GridBot>>()
    }

    pub async fn query_bots(&self, bot_ids: Vec<String>) -> Result<Option<Vec<GridBot>>, workspaces::error::Error> {
        log!("start query_bots");
        self.0
            .call("query_bots")
            .args_json(json!({
                "bot_ids": bot_ids,
            }))
            .view()
            .await?
            .json::<Option<Vec<GridBot>>>()
    }

    pub async fn query_protocol_fee(&self, token: AccountId) -> Result<Option<U256C>, workspaces::error::Error> {
        log!("start query_protocol_fee");
        self.0
            .call("query_protocol_fee")
            .args_json(json!({
                "token": token,
            }))
            .view()
            .await?
            .json::<Option<U256C>>()
    }

    pub async fn query_global_balance(&self, token: AccountId) -> Result<Option<U256C>, workspaces::error::Error> {
        log!("start query_global_balance");
        self.0
            .call("query_global_balance")
            .args_json(json!({
                "token": token,
            }))
            .view()
            .await?
            .json::<Option<U256C>>()
    }

    pub async fn query_user_balance(&self, user: &AccountId, token: AccountId) -> Result<Option<U256C>, workspaces::error::Error> {
        log!("start query_user_balance");
        self.0
            .call("query_user_balance")
            .args_json(json!({
                "user": user,
                "token": token,
            }))
            .view()
            .await?
            .json::<Option<U256C>>()
    }

    pub async fn query_user_locked_balance(&self, user: &AccountId,token: AccountId) -> Result<Option<U256C>, workspaces::error::Error> {
        log!("start query_user_locked_balance");
        self.0
            .call("query_user_locked_balance")
            .args_json(json!({
                "user": user,
                "token": token,
            }))
            .view()
            .await?
            .json::<Option<U256C>>()
    }

    pub async fn query_storage_fee(&self) -> Result<U256C, workspaces::error::Error> {
        log!("start query_storage_fee");
        self.0
            .call("query_storage_fee")
            // .args_json(json!({
            // }))
            .view()
            .await?
            .json::<U256C>()
    }
}
