use std::str::FromStr;
use near_sdk::{log};
// use near_account_id::AccountId;
// use workspaces::AccountId;
use workspaces::network::Testnet;
use workspaces::{Account, AccountId, Contract, Worker};
use workspaces::result::ExecutionFinalResult;
use crate::workspace_env::{deploy_grid_bot, deploy_token, FtContractHelper, GridBotHelper};
use std::time::{SystemTime, UNIX_EPOCH};
use workspaces::types::SecretKey;

pub const owner_id_str: &str = "cosmosfirst.testnet";
pub const owner_key_str: &str = "2K19UNqRZKD7USxhD36M3ppXEJ8R392XWpPpNbjpMyP3eLiLm4mmGuZCp7fNs8yeV62DhKCJ8sUvW8m18nCnVrPQ";

pub const maker_id_str: &str = "cosmossecond.testnet";
pub const maker_key_str: &str = "3Uswy3tqTTmG6U8Fz5shDyfFSTN9XJD4366vZ79a3kTtBSdPhjh7QEQDcxMCyybhEovoRA7qXBj4x7utnbL2XgHM";

pub const taker_id_str: &str = "cosmosthird.testnet";
pub const taker_key_str: &str = "34z4p8KwF2bNbSAG8awSdp693PsJxSWHayEBvqyP8qtcDLmfRLUfYdztAKZZXKCJwzePiPM8yf7pBq3NLC99jFmt";

pub const eth_id_str: &str = "dev-20240103091543-78946249092411";
pub const eth_key_str: &str = "3KyUucv7xFhA7xcjvS8owYeTotN2zYPc8AWhcRDkMG9ejac4gQsdVqDRrhh1v22ccuSK1JEFkhL7mzoKSuHKVyBH";

pub const usdc_id_str: &str = "dev-20240103092617-93971264224721";
pub const usdc_key_str: &str = "3KyUucv7xFhA7xcjvS8owYeTotN2zYPc8AWhcRDkMG9ejac4gQsdVqDRrhh1v22ccuSK1JEFkhL7mzoKSuHKVyBH";

pub async fn create_contract() -> Result<(Worker<Testnet>, Account, Account, Account, GridBotHelper, FtContractHelper, FtContractHelper), workspaces::error::Error> {
    let worker = workspaces::testnet().await?;
    // let owner = create_account(&worker).await;
    let owner = recovery_account(owner_id_str, owner_key_str, &worker).await;
    log!("owner account:".to_string() + &owner.id().to_string());

    let gridbot_contract = setup_contract(&worker, &owner).await?;
    // account
    // let maker_account = create_account(&worker).await;
    let maker_account = recovery_account(maker_id_str, maker_key_str, &worker).await;
    log!("maker account:".to_string() + &maker_account.id().to_string());
    // let taker_account = create_account(&worker).await;
    let taker_account = recovery_account(taker_id_str, taker_key_str, &worker).await;
    log!("taker account:".to_string() + &maker_account.id().to_string());
    // deposit
    // let eth_token_contract = setup_token_contract(&worker, "ETH", 18).await?;
    let eth_token_contract = FtContractHelper(recovery_contract(eth_id_str, eth_key_str, &worker).await);
    // let usdc_token_contract = setup_token_contract(&worker, "USDC", 6).await?;
    let usdc_token_contract = FtContractHelper(recovery_contract(usdc_id_str, usdc_key_str, &worker).await);

    // log!("eth_token_contract addr:{}", eth_token_contract.get_account_id());
    // let secret_str = serde_json::to_string(&(eth_token_contract.0.as_account().secret_key())).unwrap();
    // println!("secret_str:{}", secret_str);
    //
    // log!("usdc_token_contract addr:{}", usdc_token_contract.get_account_id());
    // let secret_str = serde_json::to_string(&(usdc_token_contract.0.as_account().secret_key())).unwrap();
    // println!("secret_str:{}", secret_str);

    Ok((worker, owner, maker_account, taker_account, gridbot_contract, eth_token_contract, usdc_token_contract))
}

pub async fn create_account(
    worker: &Worker<Testnet>
) -> Account {
    // let new_account = worker.create_tla(account_id.parse().unwrap()).await?;
    let new_account = worker.dev_create_account().await.unwrap();
    return new_account;
}

pub async fn recovery_account(
    id_str: &str,
    key_str: &str,
    worker: &Worker<Testnet>
) -> Account {
    let key = SecretKey::from_str(key_str).expect("Invalid Key");
    let id = id_str.parse::<workspaces::AccountId>().expect("");
    return Account::from_secret_key(id, key, worker);
}

pub async fn recovery_contract(
    id_str: &str,
    key_str: &str,
    worker: &Worker<Testnet>
) -> Contract {
    let key = SecretKey::from_str(key_str).expect("Invalid Key");
    let id = id_str.parse::<workspaces::AccountId>().expect("");
    return Contract::from_secret_key(id, key, worker);
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
