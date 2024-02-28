use std::str::FromStr;
use near_sdk::json_types::U128;
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::{AccountId, log, require, testing_env};
use near_units::parse_near;
use workspaces::network::Testnet;
use workspaces::{Account, Worker};
use workspaces::result::ExecutionFinalResult;
use grid::{GridBotContract, GridType, Order, OrderKeyInfo, RequestOrder, TakeRequest, U256C};
use common::*;
use crate::workspace_env::*;

mod workspace_env;

pub fn get_pair_key(base_token: &AccountId, quote_token: &AccountId) -> String {
    return format!("{}:{}", base_token.clone().to_string(), quote_token.clone().to_string());
}

// #[tokio::test]
// async fn refer() -> Result<(), workspaces::error::Error> {
//     let (worker, owner, maker_account, taker_account, gridbot_contract, eth_token_contract, usdc_token_contract) = create_contract().await?;
//
//     check_success(eth_token_contract.ft_mint(&maker_account, U128::from(10000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&maker_account, U128::from(100000000000000 as u128).into()).await);
//
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
//     // require!(user_usdc_balance == U128::from(100000000000000 as u128));
//     let user_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
//     // require!(user_eth_balance == U128::from(10000000000000000000000 as u128));
//
//     let eth_oracle_id = "27e867f0f4f61076456d1a73b14c7edc1cf5cef4f4d6193a33424288f11bd0f4"; // 333698517
//     let usdc_oracle_id = "1fc18861232290221461220bd4e2acd1dcdfbc89c84092c93c18bdc7756c1588"; // 100000737
//     // register pair
//     check_success(gridbot_contract.register_pair(&owner, &(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()), U256C::from(1000), U256C::from(1000), eth_oracle_id.to_string(), usdc_oracle_id.to_string()).await);
//
//     let refer_fee_rate = vec![500000, 400000, 300000];
//     // let refer_fee_rate = vec![500000, 400000];
//     check_success(gridbot_contract.set_refer_fee_rate(&owner, refer_fee_rate).await);
//
//     // set refer user
//     let maker_id = AccountId::from_str(maker_account.id()).expect("Invalid AccountId");
//     let taker_id = AccountId::from_str(taker_account.id()).expect("Invalid AccountId");
//     let owner_id = AccountId::from_str(owner.id()).expect("Invalid AccountId");
//     // 500000, 400000, 300000
//     // 100000 * 0.5 = 50000
//     // 50000 * 0.4 = 20000
//     // 20000 * 0.3 = 6000
//     // 30000, 14000, 6000
//     // taker: 30000
//     // owner: 14000
//     // maker: 6000
//     check_success(gridbot_contract.add_refer(&owner, maker_id.clone(), taker_id.clone()).await);
//     check_success(gridbot_contract.add_refer(&owner, taker_id.clone(), owner_id.clone()).await);
//     check_success(gridbot_contract.add_refer(&owner, owner_id.clone(), taker_id.clone()).await);
//
//     let result = gridbot_contract.query_invited_users(&taker_id.clone(), U128::from(1), U128::from(2)).await.unwrap();
//     let result_str = serde_json::to_string(&(result)).unwrap();
//     println!("result:{}", result_str);
//
//     // deposit
//     check_success(gridbot_contract.deposit(&eth_token_contract, &maker_account, 10000000000000000000000).await);
//     check_success(gridbot_contract.deposit(&usdc_token_contract, &maker_account, 100000000000000).await);
//
//     let user_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
//     println!("user_usdc_balance:{}", user_usdc_balance.0.to_string());
//     // require!(user_usdc_balance == U128::from(0 as u128));
//     let user_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
//     println!("user_eth_balance:{}", user_eth_balance.0.to_string());
//     // require!(user_eth_balance == U128::from(0 as u128));
//
//     // query global balance
//     let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("global_usdc:{}", global_usdc.0.to_string());
//     require!(global_usdc.0 == 100000000000000 as u128);
//     let global_eth = gridbot_contract.query_global_balance(eth_token_contract.get_account_id()).await?.unwrap();
//     require!(global_eth.0 == 10000000000000000000000 as u128);
//     println!("global_eth:{}", global_eth.0.to_string());
//     // query user balance
//     let user_balance_usdc = gridbot_contract.query_user_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), usdc_token_contract.get_account_id()).await?.unwrap();
//     println!("after deposit user_balance_usdc:{}", user_balance_usdc.0.to_string());
//     require!(user_balance_usdc.0 == 100000000000000 as u128);
//     let user_balance_eth = gridbot_contract.query_user_balance(&(AccountId::from_str(maker_account.id()).expect("Invalid AccountId")), eth_token_contract.get_account_id()).await?.unwrap();
//     println!("after deposit user_balance_eth:{}", user_balance_eth.0.to_string());
//     require!(user_balance_eth.0 == 10000000000000000000000 as u128);
//
//     // set oracle price
//     let current_price = U256C::from(220000);
//     let pair_id = get_pair_key(&(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()));
//     // check_success(gridbot_contract.set_oracle_price(&owner, &current_price, pair_id.clone()).await);
//
//     // create bot, big slippage, because the oracle price
//     check_success(gridbot_contract.create_bot(&maker_account, pair_id.clone(), 9990, GridType::EqOffset, 0,
//                                               U256C::from(10000000), U256C::from(100000000), U256C::from(2000000000),
//                                               U256C::from(100000000), U256C::from(3000000000 as u128), true, 10, 15,
//                                               U256C::from(0), U256C::from(0), U256C::from(0), U256C::from(get_time_stamp() * 1000 + 36000000),
//                                               U256C::from(3500000000000000000 as u128)).await);
//     let next_bot_id = format!("GRID:{}", "1".to_string());
//     // query bot
//     let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
//     let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
//     println!("grid_bot:{}", grid_bot_str);
//
//     // taker order
//     // taker account
//     // let taker_account = create_account(&worker).await;
//     // log!("taker account:".to_string() + &taker_account.id().to_string());
//     check_success(eth_token_contract.ft_mint(&taker_account, U128::from(10000000000000000000000 as u128).into()).await);
//     check_success(usdc_token_contract.ft_mint(&taker_account, U128::from(100000000000000 as u128).into()).await);
//     // // deposit
//     // check_success(gridbot_contract.deposit(&eth_token_contract, &taker_account, 10000000000000000000000).await);
//     // check_success(gridbot_contract.deposit(&usdc_token_contract, &taker_account, 100000000000000).await);
//     // sell ETH
//     // Buy ETH 100000000, 2000000000, fill base, grid_offset: 10000000, grid_buy_count: 15
//     // buy one: 100000000, 2000000000 + 10000000 * 14=2140000000
//     // buy two: 100000000, 2000000000 + 10000000 * 13=2130000000
//     let take_order = RequestOrder {
//         token_sell: eth_token_contract.get_account_id(),
//         token_buy: usdc_token_contract.get_account_id(),
//         amount_sell: U128::from(100000000 as u128),
//         amount_buy: U128::from(2140000000 as u128),
//         fill_buy_or_sell: false,
//         filled: U128::from(0),
//     };
//     let maker_orders = vec![OrderKeyInfo{
//         bot_id: next_bot_id.clone(),
//         forward_or_reverse: true,
//         level: 14,
//     }];
//     // check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
//     let taker_request = TakeRequest {
//         take_order: take_order.clone(),
//         maker_orders,
//     };
//     let taker_request_str = serde_json::to_string(&(taker_request)).unwrap();
//     // query taker balance
//     let taker_usdc_balance_before = usdc_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker taker_usdc_balance_before:{}", taker_usdc_balance_before.0.to_string());
//     // require!(user_usdc_balance == U128::from(100002140000000 as u128));
//     let taker_eth_balance_before = eth_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker taker_eth_balance_before:{}", taker_eth_balance_before.0.to_string());
//     // require!(user_eth_balance == U128::from(9999999999999900000000 as u128));
//     check_success(eth_token_contract.ft_transfer_call(&taker_account, &gridbot_contract.get_account_id(), take_order.amount_sell.clone().0, taker_request_str).await);
//
//     let take_order = RequestOrder {
//         token_sell: usdc_token_contract.get_account_id(),
//         token_buy: eth_token_contract.get_account_id(),
//         amount_sell: U128::from(2150000000 as u128),
//         amount_buy: U128::from(100000000 as u128),
//         fill_buy_or_sell: false,
//         filled: U128::from(0),
//     };
//     // 2150000000 - 2140000000 = 10000000
//     // 10000000 * 0.01= 100000
//     //
//     let maker_orders = vec![OrderKeyInfo{
//         bot_id: next_bot_id.clone(),
//         forward_or_reverse: false,
//         level: 14,
//     }];
//     // check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
//     let taker_request = TakeRequest {
//         take_order: take_order.clone(),
//         maker_orders,
//     };
//     let taker_request_str = serde_json::to_string(&(taker_request)).unwrap();
//     // query taker balance
//     let taker_usdc_balance_before = usdc_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker taker_usdc_balance_before:{}", taker_usdc_balance_before.0.to_string());
//     // require!(user_usdc_balance == U128::from(100002140000000 as u128));
//     let taker_eth_balance_before = eth_token_contract.ft_balance_of(&taker_account).await?;
//     println!("taker taker_eth_balance_before:{}", taker_eth_balance_before.0.to_string());
//     // require!(user_eth_balance == U128::from(9999999999999900000000 as u128));
//     check_success(usdc_token_contract.ft_transfer_call(&taker_account, &gridbot_contract.get_account_id(), take_order.amount_sell.clone().0, taker_request_str).await);
//
//     let taker_usdc_fee = gridbot_contract.query_refer_fee(&taker_id, usdc_token_contract.get_account_id()).await.unwrap();
//     let taker_eth_fee = gridbot_contract.query_refer_fee(&taker_id, eth_token_contract.get_account_id()).await.unwrap();
//
//     let maker_usdc_fee = gridbot_contract.query_refer_fee(&maker_id, usdc_token_contract.get_account_id()).await.unwrap();
//     let maker_eth_fee = gridbot_contract.query_refer_fee(&maker_id, eth_token_contract.get_account_id()).await.unwrap();
//
//     let owner_usdc_fee = gridbot_contract.query_refer_fee(&owner_id, usdc_token_contract.get_account_id()).await.unwrap();
//     let owner_eth_fee = gridbot_contract.query_refer_fee(&owner_id, eth_token_contract.get_account_id()).await.unwrap();
//
//     let protocol_fee = gridbot_contract.query_protocol_fee(usdc_token_contract.get_account_id()).await.unwrap();
//     println!("protocol_fee:{}", protocol_fee.0.to_string());
//     require!(protocol_fee.0 == 1120000 as u128);
//
//     println!("taker taker_usdc_fee:{}", taker_usdc_fee.0.to_string());
//     println!("taker taker_eth_fee:{}", taker_eth_fee.0.to_string());
//
//     println!("taker maker_usdc_fee:{}", maker_usdc_fee.0.to_string());
//     println!("taker maker_eth_fee:{}", maker_eth_fee.0.to_string());
//
//     println!("taker owner_usdc_fee:{}", owner_usdc_fee.0.to_string());
//     println!("taker owner_eth_fee:{}", owner_eth_fee.0.to_string());
//
//     Ok(())
// }
