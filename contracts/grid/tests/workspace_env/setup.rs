use std::fmt::Error;
use std::ptr::null;
use near_sdk::log;
use near_units::parse_near;
use serde_json::{json, to_string};
use workspaces::{Account, Worker};
use workspaces::network::{Sandbox, Testnet};
use crate::*;

pub const GRID_WASM: &str = "res/grid.wasm";
pub const TOKEN_WASM: &str = "res/token.wasm";

pub async fn deploy_grid_bot(
    worker: &Worker<Testnet>,
    owner: &Account,
    // worker: &Worker<Sandbox>,
) -> Result<GridBotHelper, workspaces::error::Error> {
    println!("deploy_grid_bot");
    let contract = worker.dev_deploy(&std::fs::read(GRID_WASM).unwrap()).await?;
    let owner_id = owner.id().clone();
    println!("contract deployed: {:?}", contract.id().clone());

    let gridbot = contract
        .call("new")
        .args_json(serde_json::json!({ "owner_id": owner_id }))
        .max_gas()
        .transact()
        .await?;
        // .map(|response| println!("success: {:?}", response))
        // .map_err(|e| println!("error: {:?}", e));

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
