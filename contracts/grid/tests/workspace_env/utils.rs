use near_sdk::{AccountId, log};
use workspaces::network::Testnet;
use workspaces::{Account, Worker};
use workspaces::result::ExecutionFinalResult;
use crate::workspace_env::{deploy_grid_bot, deploy_token, FtContractHelper, GridBotHelper};
use std::time::{SystemTime, UNIX_EPOCH};


pub async fn create_account(
    worker: &Worker<Testnet>
) -> Account {
    // let new_account = worker.create_tla(account_id.parse().unwrap()).await?;
    let new_account = worker.dev_create_account().await.unwrap();
    return new_account;
}

pub async fn setup_contract(worker: &Worker<Testnet>, owner: &Account) -> Result<GridBotHelper, workspaces::error::Error> {
    // let worker = workspaces::testnet().await?;
    // let worker = workspaces::sandbox().await?;
    let contract = deploy_grid_bot(&worker, owner).await?;
    Ok(contract)
}

pub async fn setup_token_contract(worker: &Worker<Testnet>, symbol: &str, decimal: u8) -> Result<FtContractHelper, workspaces::error::Error> {
    // let worker = workspaces::sandbox().await?;
    let contract = deploy_token(&worker, symbol, decimal).await?;
    Ok(contract)
}

pub fn check_success(result: Result<ExecutionFinalResult, workspaces::error::Error> ) {
    match result {
        Ok(execution_result) => {
            // println!("success:{:?}", execution_result);
            if execution_result.is_failure() {
                println!("error:{:?}", execution_result);
            }
            assert!(execution_result.is_success());
        },
        Err(error) => {
            println!("directly error:{:?}", error);
        }
    }
}

pub fn get_pair_key(base_token: &AccountId, quote_token: &AccountId) -> String {
    return format!("{}:{}", base_token.clone().to_string(), quote_token.clone().to_string());
}

pub fn get_time_stamp() -> u64 {
    let start = SystemTime::now();
    let timestamp;
    match start.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            timestamp = duration.as_secs();
            println!("Current timestamp: {}", timestamp);
        }
        Err(e) => {
            timestamp = 0;
            println!("An error occurred: {}", e);
        }
    }
    return timestamp;
}
