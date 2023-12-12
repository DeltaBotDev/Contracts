use std::str::FromStr;
use near_sdk::json_types::U128;
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::{AccountId, log, testing_env};
use near_units::parse_near;
use workspaces::network::Testnet;
use workspaces::{Account, Worker};
use workspaces::result::ExecutionFinalResult;
use grid::GridBotContract;
use common::*;
use crate::workspace_env::*;

mod workspace_env;

// #[tokio::test]
// async fn not_register_pair() -> Result<(), workspaces::error::Error> {
//     let worker = workspaces::testnet().await?;
//     let owner = create_account(&worker).await;
//     let gridbot_contract = setup_contract(&worker, &owner).await?;
//     // account
//     let test_account = create_account(&worker).await;
//     // deposit
//     let eth_token_contract = setup_token_contract(&worker, "ETH", 18).await?;
//     let result = eth_token_contract.ft_mint(&test_account, U128::from(10000000000000000000000 as u128).into()).await;
//     check_success(result);
//
//     // deposit
//     let result = gridbot_contract.deposit(&eth_token_contract, &test_account, 10000000000000000000000).await;
//     check_success(result);
//     Ok(())
// }

// #[tokio::test]
// async fn not_set_oracle() -> Result<(), workspaces::error::Error> {
//     let worker = workspaces::testnet().await?;
//     let owner = create_account(&worker).await;
//     let gridbot_contract = setup_contract(&worker, &owner).await?;
//     // account
//     let test_account = create_account(&worker).await;
//     // deposit
//     let eth_token_contract = setup_token_contract(&worker, "ETH", 18).await?;
//     let usdc_token_contract = setup_token_contract(&worker, "USDC", 6).await?;
//     let result = eth_token_contract.ft_mint(&test_account, U128::from(10000000000000000000000 as u128).into()).await;
//     check_success(result);
//
//     // register pair
//     gridbot_contract.register_pair(&owner, &(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()));
//
//     // deposit
//     let result = gridbot_contract.deposit(&eth_token_contract, &test_account, 10000000000000000000000).await;
//     check_success(result);
//     Ok(())
// }
