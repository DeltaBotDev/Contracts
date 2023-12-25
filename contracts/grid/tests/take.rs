use std::str::FromStr;
use near_sdk::json_types::U128;
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::{AccountId, log, require, testing_env};
use near_units::parse_near;
use workspaces::network::Testnet;
use workspaces::{Account, Worker};
use workspaces::result::ExecutionFinalResult;
use grid::{GridBotContract, GridType, Order, OrderKeyInfo, TakeRequest, U256C};
use common::*;
use crate::workspace_env::*;

mod workspace_env;

// #[tokio::test]
// async fn take() -> Result<(), workspaces::error::Error> {
//     let (worker, owner, maker_account, gridbot_contract, eth_token_contract, usdc_token_contract) = create_contract().await?;
//
//     let maker_account_id = AccountId::from_str(maker_account.id()).expect("Invalid AccountId");
//     check_success(eth_token_contract.ft_storage_deposit(&maker_account_id).await);
//     check_success(usdc_token_contract.ft_storage_deposit(&maker_account_id).await);
//
//     check_success(eth_token_contract.ft_mint(&maker_account, U128::from(10000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&maker_account, U128::from(100000000000000 as u128).into()).await);
//
//     let eth_oracle_id = "27e867f0f4f61076456d1a73b14c7edc1cf5cef4f4d6193a33424288f11bd0f4"; // 333698517
//     let usdc_oracle_id = "1fc18861232290221461220bd4e2acd1dcdfbc89c84092c93c18bdc7756c1588"; // 100000737
//     // pair price = 3.34
//
//
//     // register pair
//     check_success(gridbot_contract.register_pair(&owner, &(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()), U256C::from(1000), U256C::from(1000), eth_oracle_id.to_string(), usdc_oracle_id.to_string()).await);
//
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &maker_account, 10000000000000000000000).await);
//     check_success(gridbot_contract.deposit(&usdc_token_contract, &maker_account, 100000000000000).await);
//
//     // set oracle price
//     let current_price = U256C::from(220000);
//     let pair_id = get_pair_key(&(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()));
//     // check_success(gridbot_contract.set_oracle_price(&owner, &current_price, pair_id.clone()).await);
//
//     // create bot
//     // 10000000
//     // 100000000
//     // 350000000
//     // price base:338146497, quote:100007076
//     // more oracle_pair_price: 966064489848116917
//     // 3500000000000000000
//     //        966064489848116917
//     // 338146497 / 100007076 * 1000000000000000000
//     check_success(gridbot_contract.create_bot(&maker_account, pair_id.clone(), 1000, GridType::EqOffset, 0,
//                                               U256C::from(10000000), U256C::from(100000000), U256C::from(300000000),
//                                               U256C::from(100000000), U256C::from(600000000 as u128), true, 3, 3,
//                                               U256C::from(0), U256C::from(0), U256C::from(0), U256C::from(get_time_stamp() * 1000 + 36000000),
//                                               U256C::from(3500000000000000000 as u128)).await);
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
//     require!(grid_bot.total_base_amount == U256C::from(300000000 as u128));
//     require!(grid_bot.total_quote_amount == U256C::from(930000000 as u128));
//
//     // query order
//     let order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 0).await?.unwrap();
//     let order_string = serde_json::to_string(&(order_result.order)).unwrap();
//     println!("order:{}", order_string);
//
//     // query orders
//     let bot_ids = vec![next_bot_id.clone(), next_bot_id.clone(), next_bot_id.clone(), next_bot_id.clone(), next_bot_id.clone(), next_bot_id.clone()];
//     let forward_or_reverses = vec![true, true, true, true, true, true];
//     let levels = vec![0, 1, 2, 3, 4, 5];
//     let orders = gridbot_contract.query_orders(bot_ids, forward_or_reverses, levels).await?.unwrap();
//     let orders_string = serde_json::to_string(&orders).unwrap();
//     println!("orders:{}", orders_string);
//
//     // taker order
//     // taker account
//     let taker_account = create_account(&worker).await;
//
//     let taker_account_id = AccountId::from_str(taker_account.id()).expect("Invalid AccountId");
//     check_success(eth_token_contract.ft_storage_deposit(&taker_account_id).await);
//     check_success(usdc_token_contract.ft_storage_deposit(&taker_account_id).await);
//     log!("taker account:".to_string() + &taker_account.id().to_string());
//     check_success(eth_token_contract.ft_mint(&taker_account, U128::from(20000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&taker_account, U128::from(200000000000000 as u128).into()).await);
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &taker_account, 10000000000000000000000).await);
//     check_success(gridbot_contract.deposit(&usdc_token_contract, &taker_account, 100000000000000).await);
//     // sell ETH
//     // Buy ETH 100000000, 300000000, fill base, grid_offset: 10000000, grid_buy_count: 3
//     // buy one: 100000000, 300000000 + 10000000 * 2=320000000
//     // buy two: 100000000, 300000000 + 10000000=310000000
//     // buy three: 100000000, 300000000
//     let take_order = Order {
//         token_sell: eth_token_contract.get_account_id(),
//         token_buy: usdc_token_contract.get_account_id(),
//         amount_sell: U256C::from(100000000 as u128),
//         amount_buy: U256C::from(320000000 as u128),
//         fill_buy_or_sell: false,
//         filled: Default::default(),
//     };
//     let maker_orders = vec![OrderKeyInfo {
//         bot_id: next_bot_id.clone(),
//         forward_or_reverse: true,
//         level: 2,
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
//     // check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
//     let taker_request = TakeRequest {
//         take_order: take_order.clone(),
//         maker_orders,
//     };
//     let taker_request_str = serde_json::to_string(&(taker_request)).unwrap();
//     check_success(eth_token_contract.ft_transfer_call(&taker_account, &gridbot_contract.get_account_id(), take_order.amount_sell.clone().as_u128(), taker_request_str).await);
//
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 2).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("first forward level 2 forward order:{}", order_string);
//     // filled must be 100000000
//     require!(forward_order_result.order.filled == forward_order_result.order.amount_buy);
//
//     let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 2).await?.unwrap();
//     let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
//     println!("first forward level 12 reverse order:{}", order_string);
//     // // fixed base
//     require!(reverse_order_result.order.amount_sell == forward_order_result.order.amount_buy);
//     // reversed order sell must be 100000000, buy must be 2140000000 + 10000000
//     require!(reverse_order_result.order.amount_buy == forward_order_result.order.amount_sell + U256C::from(10000000 as u128));
//
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot after first forward:{}", grid_bot_str);
//     require!(grid_bot.total_base_amount == U256C::from(400000000 as u128));
//     // 930000000 - 320000000
//     require!(grid_bot.total_quote_amount == U256C::from(610000000 as u128));
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
//         amount_sell: U256C::from(330000000 as u128),
//         amount_buy: U256C::from(100000000 as u128),
//         fill_buy_or_sell: false,
//         filled: Default::default(),
//     };
//     let maker_orders = vec![OrderKeyInfo {
//         bot_id: next_bot_id.clone(),
//         forward_or_reverse: false,
//         level: 2,
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
//     // check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
//     let taker_request = TakeRequest {
//         take_order: take_order.clone(),
//         maker_orders,
//     };
//     let taker_request_str = serde_json::to_string(&(taker_request)).unwrap();
//     check_success(usdc_token_contract.ft_transfer_call(&taker_account, &gridbot_contract.get_account_id(), take_order.amount_sell.clone().as_u128() + 2150000000, taker_request_str).await);
//
//     // query order
//     let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 2).await?.unwrap();
//     let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
//     println!("second reverse level 2 forward order:{}", order_string);
//     require!(forward_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(forward_order_result.order.amount_sell == U256C::from(640000000 as u128));
//     require!(forward_order_result.order.amount_buy == U256C::from(200000000 as u128));
//
//     let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 2).await?.unwrap();
//     let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
//     println!("second reverse level 2 reverse order:{}", order_string);
//     require!(reverse_order_result.order.filled == U256C::from(100000000 as u128));
//     require!(reverse_order_result.order.amount_sell == U256C::from(100000000 as u128));
//     require!(reverse_order_result.order.amount_buy == U256C::from(330000000 as u128));
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot after second reverse:{}", grid_bot_str);
//     require!(grid_bot.revenue == U256C::from(9900000 as u128));
//     require!(grid_bot.total_base_amount == U256C::from(300000000 as u128));
//     require!(grid_bot.total_quote_amount == U256C::from(939900000 as u128));
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
//     Ok(())
// }
