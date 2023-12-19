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

pub async fn create_contract() -> Result<(Worker<Testnet>, Account, Account, GridBotHelper, FtContractHelper, FtContractHelper), workspaces::error::Error> {
    let worker = workspaces::testnet().await?;
    let owner = create_account(&worker).await;
    log!("owner account:".to_string() + &owner.id().to_string());
    let gridbot_contract = setup_contract(&worker, &owner).await?;
    // account
    let maker_account = create_account(&worker).await;
    log!("maker account:".to_string() + &maker_account.id().to_string());
    // deposit
    let eth_token_contract = setup_token_contract(&worker, "ETH", 18).await?;
    let usdc_token_contract = setup_token_contract(&worker, "USDC", 6).await?;
    Ok((worker, owner, maker_account, gridbot_contract, eth_token_contract, usdc_token_contract))
}

// #[tokio::test]
// async fn asset_change() -> Result<(), workspaces::error::Error> {
//     let (worker, owner, maker_account, gridbot_contract, eth_token_contract, usdc_token_contract) = create_contract().await?;
//
//     check_success(eth_token_contract.ft_mint(&maker_account, U128::from(10000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&maker_account, U128::from(100000000000000 as u128).into()).await);
//
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
//     require!(user_usdc_balance == U128::from(100000000000000 as u128));
//     let user_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
//     require!(user_eth_balance == U128::from(10000000000000000000000 as u128));
//     // register pair
//     check_success(gridbot_contract.register_pair(&owner, &(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id())).await);
//
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &maker_account, 10000000000000000000000).await);
//     check_success(gridbot_contract.deposit(&usdc_token_contract, &maker_account, 100000000000000).await);
//
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
//     require!(user_usdc_balance == U128::from(0 as u128));
//     let user_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
//     require!(user_eth_balance == U128::from(0 as u128));
//
//     // query global balance
//     let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
//     require!(global_usdc == U256C::from(100000000000000 as u128));
//     let global_eth = gridbot_contract.query_global_balance(eth_token_contract.get_account_id()).await?.unwrap();
//     require!(global_eth == U256C::from(10000000000000000000000 as u128));
//     // query user balance
//     let user_balance_usdc = gridbot_contract.query_user_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("after deposit user_balance_usdc:{}", user_balance_usdc.to_string());
//     require!(user_balance_usdc == U256C::from(100000000000000 as u128));
//     let user_balance_eth = gridbot_contract.query_user_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), eth_token_contract.get_account_id()).await?.unwrap();
//     println!("after deposit user_balance_eth:{}", user_balance_eth.to_string());
//     require!(user_balance_eth == U256C::from(10000000000000000000000 as u128));
//
//     // set oracle price
//     let current_price = U256C::from(220000);
//     let pair_id = get_pair_key(&(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()));
//     check_success(gridbot_contract.set_oracle_price(&owner, &current_price, pair_id.clone()).await);
//
//     // create bot
//     check_success(gridbot_contract.create_bot(&maker_account, pair_id.clone(), 1000, GridType::EqOffset, 0,
//                                               U256C::from(10000000), U256C::from(100000000), U256C::from(2000000000),
//                                               U256C::from(100000000), U256C::from(3000000000 as u128), true, 10, 15,
//                                               U256C::from(0), U256C::from(0), U256C::from(0), U256C::from(get_time_stamp() * 1000 + 36000000),
//                                               U256C::from(220000)).await);
//     let next_bot_id = format!("GRID:{}", "0".to_string());
//
//     // taker order
//     // taker account
//     let taker_account = create_account(&worker).await;
//     log!("taker account:".to_string() + &taker_account.id().to_string());
//     check_success(eth_token_contract.ft_mint(&taker_account, U128::from(10000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&taker_account, U128::from(100000000000000 as u128).into()).await);
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &taker_account, 10000000000000000000000).await);
//     check_success(gridbot_contract.deposit(&usdc_token_contract, &taker_account, 100000000000000).await);
//     // sell ETH
//     // Buy ETH 100000000, 2000000000, fill base, grid_offset: 10000000, grid_buy_count: 15
//     // buy one: 100000000, 2000000000 + 10000000 * 14=2140000000
//     // buy two: 100000000, 2000000000 + 10000000 * 13=2130000000
//     let take_order = Order {
//         token_sell: eth_token_contract.get_account_id(),
//         token_buy: usdc_token_contract.get_account_id(),
//         amount_sell: U256C::from(100000000 as u128),
//         amount_buy: U256C::from(2140000000 as u128),
//         fill_buy_or_sell: false,
//         filled: Default::default(),
//     };
//     let maker_orders = vec![OrderKeyInfo{
//         bot_id: next_bot_id.clone(),
//         forward_or_reverse: true,
//         level: 14,
//     }];
//     check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
//
//     // query global balance
//     let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("global_usdc:{}", global_usdc.to_string());
//     require!(global_usdc == U256C::from(199997860000000 as u128));
//     let global_eth = gridbot_contract.query_global_balance(eth_token_contract.get_account_id()).await?.unwrap();
//     println!("global_eth:{}", global_eth.to_string());
//     require!(global_eth == U256C::from(20000000000000000000000 as u128));
//     // query taker balance
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker user_usdc:{}", user_usdc_balance.0.to_string());
//     require!(user_usdc_balance == U128::from(2140000000 as u128));
//     let user_eth_balance = eth_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker user_eth:{}", user_eth_balance.0.to_string());
//     require!(user_eth_balance == U128::from(0 as u128));
//
//     // query user balance
//     let user_balance_usdc = gridbot_contract.query_user_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("after taker user_balance_usdc:{}", user_balance_usdc.to_string());
//     require!(user_balance_usdc == U256C::from(99968950000000 as u128));
//     let user_balance_eth = gridbot_contract.query_user_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), eth_token_contract.get_account_id()).await?.unwrap();
//     println!("after taker user_balance_eth:{}", user_balance_eth.to_string());
//     require!(user_balance_eth == U256C::from(9999999999999000000000 as u128));
//     // query user locked balance
//     let user_locked_balance_usdc = gridbot_contract.query_user_locked_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("after taker user_locked_balance_usdc:{}", user_locked_balance_usdc.to_string());
//     require!(user_locked_balance_usdc == U256C::from(28910000000 as u128));
//     let user_locked_balance_eth = gridbot_contract.query_user_locked_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), eth_token_contract.get_account_id()).await?.unwrap();
//     println!("after taker user_locked_balance_eth:{}", user_locked_balance_eth.to_string());
//     require!(user_locked_balance_eth == U256C::from(1100000000 as u128));
//
//     // check grid bot balance
//     let gridbot_usdc_balance = usdc_token_contract.ft_balance_of(gridbot_contract.0.as_account()).await?;
//     require!(gridbot_usdc_balance == U128::from(199997860000000 as u128));
//     let gridbot_eth_balance = eth_token_contract.ft_balance_of(gridbot_contract.0.as_account()).await?;
//     require!(gridbot_eth_balance == U128::from(20000000000000000000000 as u128));
//     Ok(())
// }

// #[tokio::test]
// async fn create_bot() -> Result<(), workspaces::error::Error> {
//     let (worker, owner, maker_account, gridbot_contract, eth_token_contract, usdc_token_contract) = create_contract().await?;
//
//     check_success(eth_token_contract.ft_mint(&maker_account, U128::from(10000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&maker_account, U128::from(100000000000000 as u128).into()).await);
//
//     // register pair
//     check_success(gridbot_contract.register_pair(&owner, &(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()), U256C::from(1000), U256C::from(1000)).await);
//
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &maker_account, 10000000000000000000000).await);
//     check_success(gridbot_contract.deposit(&usdc_token_contract, &maker_account, 100000000000000).await);
//
//     // set oracle price
//     let current_price = U256C::from(220000);
//     let pair_id = get_pair_key(&(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()));
//     check_success(gridbot_contract.set_oracle_price(&owner, &current_price, pair_id.clone()).await);
//
//     // create bot
//     check_success(gridbot_contract.create_bot(&maker_account, pair_id.clone(), 1000, GridType::EqOffset, 0,
//                                               U256C::from(10000000), U256C::from(100000000), U256C::from(2000000000),
//                                               U256C::from(100000000), U256C::from(3000000000 as u128), true, 10, 15,
//                                               U256C::from(0), U256C::from(0), U256C::from(0), U256C::from(get_time_stamp() * 1000 + 36000000),
//                                               U256C::from(220000)).await);
//     let next_bot_id = format!("GRID:{}", "0".to_string());
//     // query storage fee
//     let storage_fee = gridbot_contract.query_storage_fee().await.unwrap();
//     println!("storage_fee:{}", storage_fee.to_string());
//     require!(storage_fee == U256C::from(10000000000000000000000 as u128));
//
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot:{}", grid_bot_str);
//     require!(grid_bot.total_base_amount == U256C::from(1000000000 as u128));
//     require!(grid_bot.total_quote_amount == U256C::from(31050000000 as u128));
//
//     // query order
//     let order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 0).await?.unwrap();
//     let order_string = serde_json::to_string(&(order_result.order)).unwrap();
//     println!("order:{}", order_string);
//
//     // query orders
//     let bot_ids = vec![next_bot_id.clone(), next_bot_id.clone(), next_bot_id.clone()];
//     let forward_or_reverses = vec![true, true, true];
//     let levels = vec![0, 1, 24];
//     let orders = gridbot_contract.query_orders(bot_ids, forward_or_reverses, levels).await?.unwrap();
//     let orders_string = serde_json::to_string(&orders).unwrap();
//     println!("orders:{}", orders_string);
//
//     // taker order
//     // taker account
//     let taker_account = create_account(&worker).await;
//     log!("taker account:".to_string() + &taker_account.id().to_string());
//     check_success(eth_token_contract.ft_mint(&taker_account, U128::from(10000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&taker_account, U128::from(100000000000000 as u128).into()).await);
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &taker_account, 10000000000000000000000).await);
//     check_success(gridbot_contract.deposit(&usdc_token_contract, &taker_account, 100000000000000).await);
//     // sell ETH
//     // Buy ETH 100000000, 2000000000, fill base, grid_offset: 10000000, grid_buy_count: 15
//     // buy one: 100000000, 2000000000 + 10000000 * 14=2140000000
//     // buy two: 100000000, 2000000000 + 10000000 * 13=2130000000
//     let take_order = Order {
//         token_sell: eth_token_contract.get_account_id(),
//         token_buy: usdc_token_contract.get_account_id(),
//         amount_sell: U256C::from(100000000 as u128),
//         amount_buy: U256C::from(2140000000 as u128),
//         fill_buy_or_sell: false,
//         filled: Default::default(),
//     };
//     let maker_orders = vec![OrderKeyInfo{
//         bot_id: next_bot_id.clone(),
//         forward_or_reverse: true,
//         level: 14,
//     }];
//     let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("global_usdc{}", global_usdc.to_string());
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
//     println!("maker user_usdc{}", user_usdc_balance.0.to_string());
//     let user_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
//     println!("maker user_eth{}", user_eth_balance.0.to_string());
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker user_usdc{}", user_usdc_balance.0.to_string());
//     let user_eth_balance = eth_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker user_eth{}", user_eth_balance.0.to_string());
//     check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 14).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("first forward level 14 forward order:{}", order_string);
//     // filled must be 100000000
//     require!(forward_order_result.order.filled == forward_order_result.order.amount_buy);
//
//     let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 14).await?.unwrap();
//     let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
//     println!("first forward level 14 reverse order:{}", order_string);
//     // fixed base
//     require!(reverse_order_result.order.amount_sell == forward_order_result.order.amount_buy);
//     // reversed order sell must be 100000000, buy must be 2140000000 + 10000000
//     require!(reverse_order_result.order.amount_buy == forward_order_result.order.amount_sell + U256C::from(10000000 as u128));
//
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot after first forward:{}", grid_bot_str);
//     require!(grid_bot.total_base_amount == U256C::from(1100000000 as u128));
//     require!(grid_bot.total_quote_amount == U256C::from(28910000000 as u128));
//
//
//     let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("after first forward global_usdc{}", global_usdc.to_string());
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
//     println!("after first forward maker user_usdc{}", user_usdc_balance.0.to_string());
//     let user_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
//     println!("after first forward maker user_eth{}", user_eth_balance.0.to_string());
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&taker_account).await?;
//     println!("after first forward taker user_usdc{}", user_usdc_balance.0.to_string());
//     let user_eth_balance = eth_token_contract.ft_balance_of(&taker_account).await?;
//     println!("after first forward taker user_eth{}", user_eth_balance.0.to_string());
//
//     // buy ETH, take the reverse order
//     let take_order = Order {
//         token_sell: usdc_token_contract.get_account_id(),
//         token_buy: eth_token_contract.get_account_id(),
//         amount_sell: U256C::from(2150000000 as u128),
//         amount_buy: U256C::from(100000000 as u128),
//         fill_buy_or_sell: false,
//         filled: Default::default(),
//     };
//     let maker_orders = vec![OrderKeyInfo{
//         bot_id: next_bot_id.clone(),
//         forward_or_reverse: false,
//         level: 14,
//     }];
//     let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("global_usdc{}", global_usdc.to_string());
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
//     println!("maker user_usdc{}", user_usdc_balance.0.to_string());
//     let user_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
//     println!("maker user_eth{}", user_eth_balance.0.to_string());
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker user_usdc{}", user_usdc_balance.0.to_string());
//     let user_eth_balance = eth_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker user_eth{}", user_eth_balance.0.to_string());
//     check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 14).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("second reverse level 14 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(4280000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(200000000 as u128));
//
//     let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 14).await?.unwrap();
//     let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
//     println!("second reverse level 14 reverse order:{}", order_string);
//     require!(reverse_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(reverse_order_result.order.amount_sell == U256C::from(100000000 as u128));
//     require!(reverse_order_result.order.amount_buy == U256C::from(2150000000 as u128));
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot after second reverse:{}", grid_bot_str);
//     require!(grid_bot.revenue == U256C::from(9900000 as u128));
//     require!(grid_bot.total_base_amount == U256C::from(1000000000 as u128));
//     require!(grid_bot.total_quote_amount == U256C::from(31059900000 as u128));
//     // query protocol fee
//     let protocol_fee_usdc = gridbot_contract.query_protocol_fee(usdc_token_contract.get_account_id()).await?.unwrap();
//     require!(protocol_fee_usdc == U256C::from(100000 as u128));
//
//     let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("after second reverse:global_usdc{}", global_usdc.to_string());
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
//     println!("after second reverse:maker user_usdc{}", user_usdc_balance.0.to_string());
//     let user_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
//     println!("after second reverse:maker user_eth{}", user_eth_balance.0.to_string());
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&taker_account).await?;
//     println!("after second reverse:taker user_usdc{}", user_usdc_balance.0.to_string());
//     let user_eth_balance = eth_token_contract.ft_balance_of(&taker_account).await?;
//     println!("after second reverse:taker user_eth{}", user_eth_balance.0.to_string());
//
//     // Partial filled
//     let take_order = Order {
//         token_sell: eth_token_contract.get_account_id(),
//         token_buy: usdc_token_contract.get_account_id(),
//         amount_sell: U256C::from(50000000 as u128),
//         amount_buy: U256C::from(1070000000 as u128),
//         fill_buy_or_sell: false,
//         filled: Default::default(),
//     };
//     let maker_orders = vec![OrderKeyInfo{
//         bot_id: next_bot_id.clone(),
//         forward_or_reverse: true,
//         level: 14,
//     }];
//     let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("global_usdc{}", global_usdc.to_string());
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
//     println!("maker user_usdc{}", user_usdc_balance.0.to_string());
//     let user_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
//     println!("maker user_eth{}", user_eth_balance.0.to_string());
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker user_usdc{}", user_usdc_balance.0.to_string());
//     let user_eth_balance = eth_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker user_eth{}", user_eth_balance.0.to_string());
//     check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 14).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("Third Partial forward filled level 14 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(150000000 as u128));
//
//     let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 14).await?.unwrap();
//     let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
//     println!("Third Partial forward filled level 14 reverse order:{}", order_string);
//     require!(reverse_order_result.order.amount_sell == U256C::from(150000000 as u128));
//     require!(reverse_order_result.order.amount_buy == U256C::from(3225000000 as u128));
//
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot after third forward:{}", grid_bot_str);
//     require!(grid_bot.total_base_amount == U256C::from(1050000000 as u128));
//     require!(grid_bot.total_quote_amount == U256C::from(29989900000 as u128));
//
//     let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("after third forward:global_usdc{}", global_usdc.to_string());
//     require!(global_usdc == U256C::from(199996790000000 as u128));
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
//     println!("after third forward:maker user_usdc{}", user_usdc_balance.0.to_string());
//     require!(user_usdc_balance == U128::from(0 as u128));
//     let user_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
//     println!("after third forward:maker user_eth{}", user_eth_balance.0.to_string());
//     require!(user_eth_balance == U128::from(0 as u128));
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&taker_account).await?;
//     println!("after third forward:taker user_usdc{}", user_usdc_balance.0.to_string());
//     require!(user_usdc_balance == U128::from(3210000000 as u128));
//     let user_eth_balance = eth_token_contract.ft_balance_of(&taker_account).await?;
//     println!("after third forward:taker user_eth{}", user_eth_balance.0.to_string());
//     require!(user_eth_balance == U128::from(100000000 as u128));
//
//     // user claim revenue, any user can claim, but revenue just send to bot's owner
//     // check_success(gridbot_contract.claim(&maker_account, next_bot_id.clone()).await);
//     check_success(gridbot_contract.claim(&taker_account, next_bot_id.clone()).await);
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     require!(grid_bot.revenue == U256C::from(0));
//
//     let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("after claim:global_usdc{}", global_usdc.to_string());
//     require!(global_usdc == U256C::from(199996780100000 as u128));
//     let global_eth = gridbot_contract.query_global_balance(eth_token_contract.get_account_id()).await?.unwrap();
//     println!("after claim:global_eth{}", global_eth.to_string());
//     require!(global_eth == U256C::from(19999999999999900000000 as u128));
//     // check grid bot balance
//     let gridbot_usdc_balance = usdc_token_contract.ft_balance_of(gridbot_contract.0.as_account()).await?;
//     println!("after claim:gridbot_usdc_balance{}", gridbot_usdc_balance.0.to_string());
//     require!(gridbot_usdc_balance == U128::from(199996780100000 as u128));
//     let gridbot_eth_balance = eth_token_contract.ft_balance_of(gridbot_contract.0.as_account()).await?;
//     println!("after claim:gridbot_eth_balance{}", gridbot_eth_balance.0.to_string());
//     require!(gridbot_eth_balance == U128::from(19999999999999900000000 as u128));
//
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
//     println!("after claim:maker user_usdc:{}", user_usdc_balance.0.to_string());
//     require!(user_usdc_balance == U128::from(9900000 as u128));
//     let user_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
//     println!("after claim:maker user_eth:{}", user_eth_balance.0.to_string());
//     require!(user_eth_balance == U128::from(0 as u128));
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&taker_account).await?;
//     println!("after claim:taker user_usdc:{}", user_usdc_balance.0.to_string());
//     require!(user_usdc_balance == U128::from(3210000000 as u128));
//     let user_eth_balance = eth_token_contract.ft_balance_of(&taker_account).await?;
//     println!("after claim:taker user_eth:{}", user_eth_balance.0.to_string());
//     require!(user_eth_balance == U128::from(100000000 as u128));
//
//     // withdraw_protocol_fee
//     let owner_usdc_balance = usdc_token_contract.ft_balance_of(&owner).await?;
//     println!("before withdraw protocol fee:maker user_usdc:{}", owner_usdc_balance.0.to_string());
//     require!(owner_usdc_balance == U128::from(0 as u128));
//     let owner_eth_balance = eth_token_contract.ft_balance_of(&owner).await?;
//     println!("before withdraw protocol fee:maker user_eth:{}", owner_eth_balance.0.to_string());
//     require!(owner_eth_balance == U128::from(0 as u128));
//     check_success(gridbot_contract.withdraw_protocol_fee(&owner, usdc_token_contract.get_account_id(), (AccountId::from_str(owner.id()).expect("Invalid AccountId")), U256C::from(100000)).await);
//     let owner_usdc_balance = usdc_token_contract.ft_balance_of(&owner).await?;
//     println!("after withdraw protocol fee:maker user_usdc:{}", owner_usdc_balance.0.to_string());
//     require!(owner_usdc_balance == U128::from(100000 as u128));
//     let owner_eth_balance = eth_token_contract.ft_balance_of(&owner).await?;
//     println!("after withdraw protocol fee:maker user_eth:{}", owner_eth_balance.0.to_string());
//     require!(owner_eth_balance == U128::from(0 as u128));
//
//     Ok(())
// }

// // take multi orders
// #[tokio::test]
// async fn take_multi_orders() -> Result<(), workspaces::error::Error> {
//     let (worker, owner, maker_account, gridbot_contract, eth_token_contract, usdc_token_contract) = create_contract().await?;
//
//     check_success(eth_token_contract.ft_mint(&maker_account, U128::from(10000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&maker_account, U128::from(100000000000000 as u128).into()).await);
//
//     // register pair
//     check_success(gridbot_contract.register_pair(&owner, &(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id())).await);
//
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &maker_account, 10000000000000000000000).await);
//     check_success(gridbot_contract.deposit(&usdc_token_contract, &maker_account, 100000000000000).await);
//
//     // set oracle price
//     let current_price = U256C::from(220000);
//     let pair_id = get_pair_key(&(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()));
//     check_success(gridbot_contract.set_oracle_price(&owner, &current_price, pair_id.clone()).await);
//
//     // create bot
//     check_success(gridbot_contract.create_bot(&maker_account, pair_id.clone(), 1000, GridType::EqOffset, 0,
//                                               U256C::from(10000000), U256C::from(100000000), U256C::from(2000000000),
//                                               U256C::from(100000000), U256C::from(20000000000 as u128), true, 500, 500,
//                                               U256C::from(0), U256C::from(0), U256C::from(0), U256C::from(get_time_stamp() * 1000 + 36000000),
//                                               U256C::from(220000)).await);
//     let next_bot_id = format!("GRID:{}", "0".to_string());
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot:{}", grid_bot_str);
//     require!(grid_bot.total_base_amount == U256C::from(50000000000 as u128));
//     require!(grid_bot.total_quote_amount == U256C::from(2247500000000 as u128));
//
//     // taker order
//     // taker account
//     let taker_account = create_account(&worker).await;
//     log!("taker account:".to_string() + &taker_account.id().to_string());
//     check_success(eth_token_contract.ft_mint(&taker_account, U128::from(10000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&taker_account, U128::from(100000000000000 as u128).into()).await);
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &taker_account, 10000000000000000000000).await);
//     check_success(gridbot_contract.deposit(&usdc_token_contract, &taker_account, 100000000000000).await);
//     // sell ETH
//     // Buy ETH 100000000, 2000000000, fill base, grid_offset: 10000000, grid_buy_count: 15
//     // buy one: 100000000, 2000000000 + 10000000 * 499=6990000000
//     // buy two: 100000000, 2000000000 + 10000000 * 498=6980000000
//     // buy three: 100000000, 2000000000 + 10000000 * 497=6970000000
//     // buy four: 100000000, 2000000000 + 10000000 * 496=6960000000
//     let take_order = Order {
//         order_id: "".to_string(),
//         token_sell: eth_token_contract.get_account_id(),
//         token_buy: usdc_token_contract.get_account_id(),
//         amount_sell: U256C::from(350000000 as u128),
//         amount_buy: U256C::from(24360000000 as u128), // 24360000000 = 6960000000*3.5
//         fill_buy_or_sell: false,
//         filled: Default::default(),
//     };
//     let maker_orders = vec![
//         OrderKeyInfo { bot_id: next_bot_id.clone(), forward_or_reverse: true, level: 499},
//         OrderKeyInfo { bot_id: next_bot_id.clone(), forward_or_reverse: true, level: 498},
//         OrderKeyInfo { bot_id: next_bot_id.clone(), forward_or_reverse: true, level: 497},
//         OrderKeyInfo { bot_id: next_bot_id.clone(), forward_or_reverse: true, level: 496},
//     ];
//     check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 499).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 499 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(6990000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(100000000 as u128));
//
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 498).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 498 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(6980000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(100000000 as u128));
//
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 497).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 497 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(6970000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(100000000 as u128));
//
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 496).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 496 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(50000000 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(6960000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(100000000 as u128));
//
//     let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 499).await?.unwrap();
//     let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
//     println!("level 499 reverse order:{}", order_string);
//     require!(reverse_order_result.order.filled == U256C::from(0 as u128));
//     require!(reverse_order_result.order.amount_sell == U256C::from(100000000 as u128));
//     require!(reverse_order_result.order.amount_buy == U256C::from(7000000000 as u128));
//
//     let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 496).await?.unwrap();
//     let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
//     println!("first forward level 496 reverse order:{}", order_string);
//     require!(reverse_order_result.order.filled == U256C::from(0 as u128));
//     require!(reverse_order_result.order.amount_sell == U256C::from(50000000 as u128));
//     require!(reverse_order_result.order.amount_buy == U256C::from(3485000000 as u128));
//
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot:{}", grid_bot_str);
//     require!(grid_bot.total_base_amount == U256C::from(50350000000 as u128));
//     require!(grid_bot.total_quote_amount == U256C::from(2223080000000 as u128));
//
//     // taker asset
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker user_usdc:{}", user_usdc_balance.0.to_string());
//     require!(user_usdc_balance == U128::from(24420000000 as u128));
//     let user_eth_balance = eth_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker user_eth:{}", user_eth_balance.0.to_string());
//     require!(user_eth_balance == U128::from(0 as u128));
//
//     // user locked asset
//     let user_locked_balance_usdc = gridbot_contract.query_user_locked_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("user_locked_balance_usdc:{}", user_locked_balance_usdc.to_string());
//     require!(user_locked_balance_usdc == U256C::from(2223080000000 as u128));
//     let user_locked_balance_eth = gridbot_contract.query_user_locked_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), eth_token_contract.get_account_id()).await?.unwrap();
//     println!("user_locked_balance_eth:{}", user_locked_balance_eth.to_string());
//     require!(user_locked_balance_eth == U256C::from(50350000000 as u128));
//
//     // global asset
//     let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("global_usdc:{}", global_usdc.to_string());
//     require!(global_usdc == U256C::from(199975580000000 as u128));
//     let global_eth = gridbot_contract.query_global_balance(eth_token_contract.get_account_id()).await?.unwrap();
//     println!("global_eth:{}", global_eth.to_string());
//     require!(global_eth == U256C::from(20000000000000000000000 as u128));
//
//     // contract balance
//     let gridbot_usdc_balance = usdc_token_contract.ft_balance_of(gridbot_contract.0.as_account()).await?;
//     println!("gridbot_usdc_balance{}", gridbot_usdc_balance.0.to_string());
//     require!(gridbot_usdc_balance == U128::from(199975580000000 as u128));
//     let gridbot_eth_balance = eth_token_contract.ft_balance_of(gridbot_contract.0.as_account()).await?;
//     println!("gridbot_eth_balance{}", gridbot_eth_balance.0.to_string());
//     require!(gridbot_eth_balance == U128::from(20000000000000000000000 as u128));
//
//     Ok(())
// }

// // rate grid
// #[tokio::test]
// async fn rate_grid() -> Result<(), workspaces::error::Error> {
//     let (worker, owner, maker_account, gridbot_contract, eth_token_contract, usdc_token_contract) = create_contract().await?;
//
//     check_success(eth_token_contract.ft_mint(&maker_account, U128::from(10000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&maker_account, U128::from(100000000000000 as u128).into()).await);
//
//     // register pair
//     check_success(gridbot_contract.register_pair(&owner, &(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id())).await);
//
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &maker_account, 10000000000000000000000).await);
//     check_success(gridbot_contract.deposit(&usdc_token_contract, &maker_account, 100000000000000).await);
//
//     // set oracle price
//     let current_price = U256C::from(220000);
//     let pair_id = get_pair_key(&(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()));
//     check_success(gridbot_contract.set_oracle_price(&owner, &current_price, pair_id.clone()).await);
//
//     // create bot
//     check_success(gridbot_contract.create_bot(&maker_account, pair_id.clone(), 1000, GridType::EqRate, 1000,
//                                               U256C::from(10000000), U256C::from(100000000), U256C::from(2000000000),
//                                               U256C::from(100000000), U256C::from(5000000000 as u128), true, 3, 3,
//                                               U256C::from(0), U256C::from(0), U256C::from(0), U256C::from(get_time_stamp() * 1000 + 36000000),
//                                               U256C::from(220000)).await);
//     let next_bot_id = format!("GRID:{}", "0".to_string());
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot:{}", grid_bot_str);
//     // require!(grid_bot.total_base_amount == U256C::from(50000000000 as u128));
//     // require!(grid_bot.total_quote_amount == U256C::from(2247500000000 as u128));
//
//     // taker order
//     // taker account
//     let taker_account = create_account(&worker).await;
//     log!("taker account:".to_string() + &taker_account.id().to_string());
//     check_success(eth_token_contract.ft_mint(&taker_account, U128::from(10000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&taker_account, U128::from(100000000000000 as u128).into()).await);
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &taker_account, 10000000000000000000000).await);
//     check_success(gridbot_contract.deposit(&usdc_token_contract, &taker_account, 100000000000000).await);
//     // sell ETH
//     // buy one: 100000000, 2000000000 * 1.1^2 = 2420000000
//     // buy two: 100000000, 2000000000 * 1.1 = 2200000000
//     // buy three: 100000000, 2000000000 = 2000000000
//     let take_order = Order {
//         order_id: "".to_string(),
//         token_sell: eth_token_contract.get_account_id(),
//         token_buy: usdc_token_contract.get_account_id(),
//         amount_sell: U256C::from(200000000 as u128),
//         amount_buy: U256C::from(4400000000 as u128),
//         fill_buy_or_sell: false,
//         filled: Default::default(),
//     };
//     let maker_orders = vec![
//         OrderKeyInfo{ bot_id: next_bot_id.clone(), forward_or_reverse: true, level: 2, },
//         OrderKeyInfo{ bot_id: next_bot_id.clone(), forward_or_reverse: true, level: 1, }
//     ];
//     check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
//
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot:{}", grid_bot_str);
//
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 2).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 2 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(2420000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(100000000 as u128));
//
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 1).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 1 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(2200000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(100000000 as u128));
//
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 3).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 3 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(0 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(4132231404 as u128));
//
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 4).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 4 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(0 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(4545454545 as u128));
//
//
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 5).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 5 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(0 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(5000000000 as u128));
//
//     let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 2).await?.unwrap();
//     let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
//     println!("level 2 reverse order:{}", order_string);
//     require!(reverse_order_result.order.filled == U256C::from(0 as u128));
//     require!(reverse_order_result.order.amount_sell == U256C::from(100000000 as u128));
//     require!(reverse_order_result.order.amount_buy == U256C::from(2662000000 as u128));
//
//     // query global balance
//     let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("global_usdc:{}", global_usdc.to_string());
//     require!(global_usdc == U256C::from(199995380000000 as u128));
//     let global_eth = gridbot_contract.query_global_balance(eth_token_contract.get_account_id()).await?.unwrap();
//     println!("global_eth:{}", global_eth.to_string());
//     require!(global_eth == U256C::from(20000000000000000000000 as u128));
//     // query taker balance
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker user_usdc:{}", user_usdc_balance.0.to_string());
//     require!(user_usdc_balance == U128::from(4620000000 as u128));
//     let user_eth_balance = eth_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker user_eth:{}", user_eth_balance.0.to_string());
//     require!(user_eth_balance == U128::from(0 as u128));
//
//     // query user balance
//     let user_balance_usdc = gridbot_contract.query_user_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("after taker user_balance_usdc:{}", user_balance_usdc.to_string());
//     require!(user_balance_usdc == U256C::from(99993380000000 as u128));
//     let user_balance_eth = gridbot_contract.query_user_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), eth_token_contract.get_account_id()).await?.unwrap();
//     println!("after taker user_balance_eth:{}", user_balance_eth.to_string());
//     require!(user_balance_eth == U256C::from(9999999999999700000000 as u128));
//     // query user locked balance
//     let user_locked_balance_usdc = gridbot_contract.query_user_locked_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("after taker user_locked_balance_usdc:{}", user_locked_balance_usdc.to_string());
//     require!(user_locked_balance_usdc == U256C::from(2000000000 as u128));
//     let user_locked_balance_eth = gridbot_contract.query_user_locked_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), eth_token_contract.get_account_id()).await?.unwrap();
//     println!("after taker user_locked_balance_eth:{}", user_locked_balance_eth.to_string());
//     require!(user_locked_balance_eth == U256C::from(500000000 as u128));
//
//     // take reverse bot
//     // buy ETH
//     // sell one: 100000000, 2000000000 * 1.1^3 = 2662000000
//     // sell two: 100000000, 2000000000 * 1.1^2 = 2420000000
//     // sell three: 1, 5000000000/1.1/1.1 = 4050000000
//     let take_order = Order {
//         order_id: "".to_string(),
//         token_sell: usdc_token_contract.get_account_id(),
//         token_buy: eth_token_contract.get_account_id(),
//         amount_sell: U256C::from(12406694214 as u128),
//         amount_buy: U256C::from(300000000 as u128),
//         fill_buy_or_sell: true,
//         filled: Default::default(),
//     };
//     let maker_orders = vec![
//         OrderKeyInfo{ bot_id: next_bot_id.clone(), forward_or_reverse: true, level: 3, },
//         OrderKeyInfo{ bot_id: next_bot_id.clone(), forward_or_reverse: false, level: 2, },
//         OrderKeyInfo{ bot_id: next_bot_id.clone(), forward_or_reverse: false, level: 1, }
//     ];
//     check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
//
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot:{}", grid_bot_str);
//     require!(grid_bot.total_base_amount == U256C::from(200000000 as u128));
//     require!(grid_bot.total_quote_amount == U256C::from(11209611404 as u128));
//     require!(grid_bot.revenue == U256C::from(457380000 as u128));
//
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 2).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 2 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(4840000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(200000000 as u128));
//
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 1).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 1 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(4400000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(200000000 as u128));
//
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 3).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 3 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(4132231404 as u128));
//
//     let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 2).await?.unwrap();
//     let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
//     println!("level 2 reverse order:{}", order_string);
//     require!(reverse_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(reverse_order_result.order.amount_sell == U256C::from(100000000 as u128));
//     require!(reverse_order_result.order.amount_buy == U256C::from(2662000000 as u128));
//
//     let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 1).await?.unwrap();
//     let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
//     println!("level 1 reverse order:{}", order_string);
//     require!(reverse_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(reverse_order_result.order.amount_sell == U256C::from(100000000 as u128));
//     require!(reverse_order_result.order.amount_buy == U256C::from(2420000000 as u128));
//
//     let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 3).await?.unwrap();
//     let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
//     println!("level 3 reverse order:{}", order_string);
//     require!(reverse_order_result.order.filled == U256C::from(0 as u128));
//     require!(reverse_order_result.order.amount_sell == U256C::from(3756574003 as u128));
//     require!(reverse_order_result.order.amount_buy == U256C::from(100000000 as u128));
//
//     // take reverse bot
//     // sell ETH
//     // sell one: 100000000, 3756574003
//     let take_order = Order {
//         order_id: "".to_string(),
//         token_sell: eth_token_contract.get_account_id(),
//         token_buy: usdc_token_contract.get_account_id(),
//         amount_sell: U256C::from(100000000 as u128),
//         amount_buy: U256C::from(3756574003 as u128),
//         fill_buy_or_sell: true,
//         filled: Default::default(),
//     };
//     let maker_orders = vec![
//         OrderKeyInfo{ bot_id: next_bot_id.clone(), forward_or_reverse: false, level: 3, },
//     ];
//     check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
//
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot:{}", grid_bot_str);
//     // require!(grid_bot.total_base_amount == U256C::from(300000000 as u128));
//     // require!(grid_bot.total_quote_amount == U256C::from(7449280827 as u128));
//     // require!(grid_bot.revenue == U256C::from(829280826 as u128));
//
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 3).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("level 3 forward order:{}", order_string);
//     // require!(forward_order_result.order.filled == U256C::from(100000000 as u128));
//     // require!(forward_order_result.order.amount_sell == U256C::from(100000000 as u128));
//     // require!(forward_order_result.order.amount_buy == U256C::from(4132231404 as u128));
//     //
//     // query order
//     let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 3).await?.unwrap();
//     let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
//     println!("level 3 reverse order:{}", order_string);
//     // require!(reverse_order_result.order.filled == U256C::from(0 as u128));
//     // require!(reverse_order_result.order.amount_sell == U256C::from(3756574003 as u128));
//     // require!(reverse_order_result.order.amount_buy == U256C::from(100000000 as u128));
//
//
//     Ok(())
// }

#[tokio::test]
async fn min_deposit() -> Result<(), workspaces::error::Error> {
    let (worker, owner, maker_account, gridbot_contract, eth_token_contract, usdc_token_contract) = create_contract().await?;

    check_success(eth_token_contract.ft_mint(&maker_account, U128::from(10000000000000000000000 as u128).into()).await);
    check_success(usdc_token_contract.ft_mint(&maker_account, U128::from(100000000000000 as u128).into()).await);

    // register pair
    check_success(gridbot_contract.register_pair(&owner, &(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()), U256C::from(10000), U256C::from(10000)).await);

    // deposit
    check_success(gridbot_contract.deposit(&eth_token_contract, &maker_account, 10000).await);

    check_success(gridbot_contract.set_min_deposit(&owner, eth_token_contract.get_account_id(), U256C::from(100)).await);

    // deposit
    check_success(gridbot_contract.deposit(&eth_token_contract, &maker_account, 100).await);
    Ok(())
}

// withdraw

// withdraw_unowned_asset

// pause
// shutdown
// start
