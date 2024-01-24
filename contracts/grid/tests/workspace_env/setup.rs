use std::fmt::Error;
use std::ptr::null;
use near_sdk::log;
use near_units::parse_near;
use serde_json::{json, to_string};
use workspaces::{Account, Worker};
use workspaces::network::{Sandbox, Testnet};
use crate::*;
use near_sdk::{AccountId};

pub const GRID_WASM: &str = "res/grid.wasm";
pub const TOKEN_WASM: &str = "res/token.wasm";

pub async fn deploy_grid_bot(
    worker: &Worker<Testnet>,
    owner: &Account,
    // worker: &Worker<Sandbox>,
) -> Result<GridBotHelper, workspaces::error::Error> {
    println!("deploy_grid_bot");
    // let contract = worker.dev_deploy(&std::fs::read(GRID_WASM).unwrap()).await?;
    // let (id, sk) = worker.dev_generate().await;
    let accunt = create_account(&worker).await;
    let money_account = recovery_account(maker_id_str, maker_key_str, &worker).await;
    money_account.transfer_near(&accunt.id(), 2000000000000000000000000 as u128).await;
    let contract = worker.create_tla_and_deploy(accunt.id().clone(), accunt.secret_key().clone(), &std::fs::read(GRID_WASM).unwrap()).await?.result;

    let owner_id = owner.id().clone();
    let oracle_id = AccountId::new_unchecked("pyth-oracle.testnet".to_string());
    let wnear = AccountId::new_unchecked("wrap.testnet".to_string());
    println!("contract deployed: {:?}", contract.id().clone());



    println!("contract deployed before new");
    // owner_id: AccountId, oracle: AccountId, wnear: AccountId
    let gridbot = contract
        .call("new")
        .args_json(serde_json::json!({ "owner_id": owner_id, "oracle": oracle_id, "wnear": wnear }))
        .max_gas()
        .transact()
        .await?;
        // .map(|response| println!("success: {:?}", response))
        // .map_err(|e| println!("error: {:?}", e));

    println!("contract deployed after new");
    Ok(GridBotHelper(contract))
}

pub async fn deploy_token(
    worker: &Worker<Testnet>,
    symbol: &str,
    decimal: u8,
) -> Result<FtContractHelper, workspaces::error::Error> {
    let contract = worker.dev_deploy(&std::fs::read(TOKEN_WASM).unwrap()).await?;
    let owner_id = contract.id().clone();
    println!("contract deployed: {:?}", owner_id.clone());

    let gridbot = contract
        .call("new")
        .args_json(serde_json::json!({ "name": symbol.clone(), "symbol": symbol.clone(), "decimals": decimal }))
        .max_gas()
        .transact()
        .await?;
        // .map(|response| println!("success: {:?}", response))
        // .map_err(|e| println!("error: {:?}", e));

    Ok(FtContractHelper(contract))
}
