use std::str::FromStr;
use near_sdk::json_types::U128;
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::{AccountId, log, require, testing_env};
use near_units::parse_near;
use workspaces::network::Testnet;
use workspaces::{Account, Worker};
use workspaces::result::ExecutionFinalResult;
use grid::{GridBotContract, GridType, Order, OrderKeyInfo, U256C};
use common::*;
use crate::workspace_env::*;

mod workspace_env;

// #[tokio::test]
// async fn upgrade() -> Result<(), workspaces::error::Error> {
//     let (worker, owner, maker_account, gridbot_contract, eth_token_contract, usdc_token_contract) = create_contract().await?;
//
//     check_success(eth_token_contract.ft_mint(&maker_account, U128::from(10000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&maker_account, U128::from(100000000000000 as u128).into()).await);
//
//     // register pair
//     check_success(gridbot_contract.register_pair(&owner, &(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()), U256C::from(10000), U256C::from(10000)).await);
//
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &maker_account, 10000).await);
//
//     check_success(gridbot_contract.set_min_deposit(&owner, eth_token_contract.get_account_id(), U256C::from(100)).await);
//
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &maker_account, 100).await);
//
//     // migrate
//     owner
//         .call(gridbot_contract.0.id(), "upgrade")
//         .args_json(std::fs::read(GRID_WASM).unwrap())
//         .max_gas()
//         .deposit(1)
//         .transact()
//         .await?.is_success();
//     Ok(())
// }
