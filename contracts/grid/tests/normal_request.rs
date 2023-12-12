use std::str::FromStr;
use near_sdk::json_types::U128;
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::{AccountId, log, require, testing_env};
use near_units::parse_near;
use workspaces::network::Testnet;
use workspaces::{Account, Worker};
use workspaces::result::ExecutionFinalResult;
use grid::{GridBotContract, GridType, Order, OrderKeyInfo, U128C};
use common::*;
use crate::workspace_env::*;

mod workspace_env;


#[tokio::test]
async fn create_bot() -> Result<(), workspaces::error::Error> {
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
    check_success(eth_token_contract.ft_mint(&maker_account, U128::from(10000000000000000000000 as u128).into()).await);
    check_success(usdc_token_contract.ft_mint(&maker_account, U128::from(100000000000000 as u128).into()).await);

    // register pair
    check_success(gridbot_contract.register_pair(&owner, &(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id())).await);

    // deposit
    check_success(gridbot_contract.deposit(&eth_token_contract, &maker_account, 10000000000000000000000).await);
    check_success(gridbot_contract.deposit(&usdc_token_contract, &maker_account, 100000000000000).await);

    // set oracle price
    let current_price = U128C::from(220000);
    let pair_id = get_pair_key(&(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()));
    check_success(gridbot_contract.set_oracle_price(&owner, &current_price, pair_id.clone()).await);

    // create bot
    check_success(gridbot_contract.create_bot(&maker_account, "test1".to_string(), pair_id.clone(), 1000, GridType::EqOffset, 0,
                                              U128C::from(10000000), U128C::from(100000000), U128C::from(2000000000),
                                              U128C::from(100000000), U128C::from(3000000000 as u128), true, 10, 15,
                                              U128C::from(0), U128C::from(0), U128C::from(0), U128C::from(get_time_stamp() * 1000 + 36000000),
                                              U128C::from(220000)).await);
    // query order
    let next_bot_id = format!("GRID:{}", "0".to_string());
    let order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 0).await?.unwrap();
    let order_string = serde_json::to_string(&(order_result.order)).unwrap();
    println!("order:{}", order_string);

    // query orders
    let bot_ids = vec![next_bot_id.clone(), next_bot_id.clone(), next_bot_id.clone()];
    let forward_or_reverses = vec![true, true, true];
    let levels = vec![0, 1, 24];
    let orders = gridbot_contract.query_orders(bot_ids, forward_or_reverses, levels).await?.unwrap();
    let orders_string = serde_json::to_string(&orders).unwrap();
    println!("orders:{}", orders_string);

    // taker order
    // taker account
    let taker_account = create_account(&worker).await;
    log!("taker account:".to_string() + &taker_account.id().to_string());
    check_success(eth_token_contract.ft_mint(&taker_account, U128::from(10000000000000000000000 as u128).into()).await);
    check_success(usdc_token_contract.ft_mint(&taker_account, U128::from(100000000000000 as u128).into()).await);
    // deposit
    check_success(gridbot_contract.deposit(&eth_token_contract, &taker_account, 10000000000000000000000).await);
    check_success(gridbot_contract.deposit(&usdc_token_contract, &taker_account, 100000000000000).await);
    // sell ETH
    // Buy ETH 100000000, 2000000000, fill base, grid_offset: 10000000, grid_buy_count: 15
    // buy one: 100000000, 2000000000 + 10000000 * 14=2140000000
    // buy two: 100000000, 2000000000 + 10000000 * 13=2130000000
    let take_order = Order {
        order_id: "".to_string(),
        token_sell: eth_token_contract.get_account_id(),
        token_buy: usdc_token_contract.get_account_id(),
        amount_sell: U128C::from(100000000 as u128),
        amount_buy: U128C::from(2140000000 as u128),
        fill_buy_or_sell: false,
        filled: Default::default(),
    };
    let maker_orders = vec![OrderKeyInfo{
        bot_id: next_bot_id.clone(),
        forward_or_reverse: true,
        level: 14,
    }];
    check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
    // query order
    let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 14).await?.unwrap();
    let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
    println!("first forward level 14 forward order:{}", order_string);
    // filled must be 100000000
    require!(forward_order_result.order.filled == forward_order_result.order.amount_buy);

    let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 14).await?.unwrap();
    let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
    println!("first forward level 14 reverse order:{}", order_string);
    // fixed base
    require!(reverse_order_result.order.amount_sell == forward_order_result.order.amount_buy);
    // reversed order sell must be 100000000, buy must be 2140000000 + 10000000
    require!(reverse_order_result.order.amount_buy == forward_order_result.order.amount_sell + U128C::from(10000000 as u128));

    // buy ETH, take the reverse order
    let take_order = Order {
        order_id: "".to_string(),
        token_sell: usdc_token_contract.get_account_id(),
        token_buy: eth_token_contract.get_account_id(),
        amount_sell: U128C::from(2150000000 as u128),
        amount_buy: U128C::from(100000000 as u128),
        fill_buy_or_sell: false,
        filled: Default::default(),
    };
    let maker_orders = vec![OrderKeyInfo{
        bot_id: next_bot_id.clone(),
        forward_or_reverse: false,
        level: 14,
    }];
    check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
    // query order
    let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 14).await?.unwrap();
    let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
    println!("second reverse level 14 forward order:{}", order_string);
    // // filled must be 100000000
    // require!(forward_order_result.order.filled == forward_order_result.order.amount_buy);

    let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 14).await?.unwrap();
    let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
    println!("second reverse level 14 reverse order:{}", order_string);
    // // fixed base
    // require!(reverse_order_result.order.amount_sell == forward_order_result.order.amount_buy);
    // // reversed order sell must be 100000000, buy must be 2140000000 + 10000000
    // require!(reverse_order_result.order.amount_buy == forward_order_result.order.amount_sell + U128C::from(10000000));

    // Partial filled
    let take_order = Order {
        order_id: "".to_string(),
        token_sell: eth_token_contract.get_account_id(),
        token_buy: usdc_token_contract.get_account_id(),
        amount_sell: U128C::from(50000000 as u128),
        amount_buy: U128C::from(1070000000 as u128),
        fill_buy_or_sell: false,
        filled: Default::default(),
    };
    let maker_orders = vec![OrderKeyInfo{
        bot_id: next_bot_id.clone(),
        forward_or_reverse: true,
        level: 14,
    }];
    check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
    // query order
    let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 14).await?.unwrap();
    let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
    println!("Third Partial forward filled level 14 forward order:{}", order_string);
    // // filled must be 100000000
    // require!(forward_order_result.order.filled == forward_order_result.order.amount_buy);

    let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 14).await?.unwrap();
    let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
    println!("Third Partial forward filled level 14 reverse order:{}", order_string);

    Ok(())
}
