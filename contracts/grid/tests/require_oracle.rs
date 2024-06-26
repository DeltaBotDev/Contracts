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


#[tokio::test]
async fn require_oracle() -> Result<(), workspaces::error::Error> {
    let (worker, owner, maker_account, taker_account, gridbot_contract, eth_token_contract, usdc_token_contract) = create_contract().await?;

    check_success(eth_token_contract.ft_mint(&maker_account, U128::from(10000000000000000000000 as u128).into()).await);
    check_success(usdc_token_contract.ft_mint(&maker_account, U128::from(100000000000000 as u128).into()).await);

    // check_success(usdc_token_contract.ft_storage_deposit(&maker_account).await);

    let eth_oracle_id = "27e867f0f4f61076456d1a73b14c7edc1cf5cef4f4d6193a33424288f11bd0f4"; // 333698517
    let usdc_oracle_id = "1fc18861232290221461220bd4e2acd1dcdfbc89c84092c93c18bdc7756c1588"; // 100000737
    // register pair
    check_success(gridbot_contract.register_pair(&owner, &(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()), U256C::from(1000), U256C::from(1000), false, eth_oracle_id.to_string(), usdc_oracle_id.to_string()).await);
    // check_success(gridbot_contract.register_pair(&owner, &(AccountId::from_str("wrap.testnet").expect("")), &(usdc_token_contract.get_account_id()), U256C::from(1000), U256C::from(1000), false, eth_oracle_id.to_string(), usdc_oracle_id.to_string()).await);

    // deposit
    check_success(gridbot_contract.deposit(&eth_token_contract, &maker_account, 10000000000000000000000).await);
    check_success(gridbot_contract.deposit(&usdc_token_contract, &maker_account, 100000000000000).await);

    // set oracle price
    let current_price = U256C::from(220000);
    let pair_id = get_pair_key(&(eth_token_contract.get_account_id()), &(usdc_token_contract.get_account_id()));
    // check_success(gridbot_contract.set_oracle_price(&owner, &current_price, pair_id.clone()).await);

    // create bot
    check_success(gridbot_contract.create_bot(&maker_account, pair_id.clone(), 9999, GridType::EqOffset, 0,
                                              U256C::from(10000000), U256C::from(100000000), U256C::from(2000000000),
                                              U256C::from(100000000), U256C::from(3000000000 as u128), true, 10, 15,
                                              U256C::from(0), U256C::from(0), U256C::from(0), U256C::from(get_time_stamp() * 1000 + 3600000000),
                                              U256C::from(7000000000000000000 as u128)).await);
    let next_bot_id = format!("GRID:{}", "1".to_string());
    // // query storage fee
    // let storage_fee = gridbot_contract.query_storage_fee().await.unwrap();
    // println!("storage_fee:{}", storage_fee.to_string());
    // require!(storage_fee == U256C::from(10000000000000000000000 as u128));

    // query bot
    let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
    let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
    println!("grid_bot:{}", grid_bot_str);
    require!(grid_bot.total_base_amount == U256C::from(1000000000 as u128));
    require!(grid_bot.total_quote_amount == U256C::from(31050000000 as u128));

    // query order
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
    // let taker_account = create_account(&worker).await;
    log!("taker account:".to_string() + &taker_account.id().to_string());
    check_success(eth_token_contract.ft_mint(&taker_account, U128::from(20000000000000000000000 as u128).into()).await);
    check_success(usdc_token_contract.ft_mint(&taker_account, U128::from(200000000000000 as u128).into()).await);
    // deposit
    // check_success(gridbot_contract.deposit(&eth_token_contract, &taker_account, 10000000000000000000000).await);
    // check_success(gridbot_contract.deposit(&usdc_token_contract, &taker_account, 100000000000000).await);
    // sell ETH
    // Buy ETH 100000000, 2000000000, fill base, grid_offset: 10000000, grid_buy_count: 15
    // buy one: 100000000, 2000000000 + 10000000 * 14=2140000000
    // buy two: 100000000, 2000000000 + 10000000 * 13=2130000000
    let take_order = RequestOrder {
        token_sell: eth_token_contract.get_account_id(),
        token_buy: usdc_token_contract.get_account_id(),
        amount_sell: U128::from(100000000 as u128),
        amount_buy: U128::from(2140000000 as u128),
        fill_buy_or_sell: false,
        filled: U128::from(0),
    };
    let maker_orders = vec![OrderKeyInfo{
        bot_id: next_bot_id.clone(),
        forward_or_reverse: true,
        level: 14,
    }];
    let global_usdc_before = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
    println!("global_usdc_before:{}", global_usdc_before.0.to_string());
    let maker_usdc_balance_before = usdc_token_contract.ft_balance_of(&maker_account).await?;
    println!("maker_usdc_balance_before:{}", maker_usdc_balance_before.0.to_string());
    let maker_eth_balance_before = eth_token_contract.ft_balance_of(&maker_account).await?;
    println!("maker maker_eth_balance_before{}", maker_eth_balance_before.0.to_string());
    let taker_usdc_balance_before = usdc_token_contract.ft_balance_of(&taker_account).await?;
    println!("taker taker_usdc_balance_before{}", taker_usdc_balance_before.0.to_string());
    let taker_eth_balance_before = eth_token_contract.ft_balance_of(&taker_account).await?;
    println!("taker taker_eth_balance_before{}", taker_eth_balance_before.0.to_string());
    // check_success(gridbot_contract.take_orders(&taker_account, &take_order, maker_orders).await);
    let taker_request = TakeRequest {
        take_order: take_order.clone(),
        maker_orders,
        return_near: Some(true),
    };
    let taker_request_str = serde_json::to_string(&(taker_request)).unwrap();
    check_success(eth_token_contract.ft_transfer_call(&taker_account, &gridbot_contract.get_account_id(), take_order.amount_sell.clone().0, taker_request_str).await);

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
    require!(reverse_order_result.order.amount_buy == forward_order_result.order.amount_sell + U256C::from(10000000 as u128));

    // query bot
    let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
    let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
    println!("grid_bot after first forward:{}", grid_bot_str);
    require!(grid_bot.total_base_amount == U256C::from(1100000000 as u128));
    require!(grid_bot.total_quote_amount == U256C::from(28910000000 as u128));


    let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
    // 100000000000000 - 99997861070000=2138930000
    // 2140000000 * (1000000 - 500)/1000000=2138930000
    println!("after first forward global_usdc:{}", global_usdc.0.to_string());
    require!(global_usdc_before.0 - global_usdc.0 == (2138930000 as u128));
    let maker_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
    println!("after first forward maker maker_usdc_balance:{}", maker_usdc_balance.0.to_string());
    require!(maker_usdc_balance_before.0 == maker_usdc_balance.0);
    let maker_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
    println!("after first forward maker maker_eth_balance:{}", maker_eth_balance.0.to_string());
    require!(maker_eth_balance_before.0 == maker_eth_balance.0);
    let taker_usdc_balance = usdc_token_contract.ft_balance_of(&taker_account).await?;
    // taker_usdc_balance_before - taker_usdc_balance = 2140000000
    println!("after first forward taker taker_usdc_balance:{}", taker_usdc_balance.0.to_string());
    require!(taker_usdc_balance.0 - taker_usdc_balance_before.0 == (2138930000 as u128));
    let taker_eth_balance = eth_token_contract.ft_balance_of(&taker_account).await?;
    println!("after first forward taker taker_eth_balance:{}", taker_eth_balance.0.to_string());
    require!(taker_eth_balance_before.0 - taker_eth_balance.0 == (100000000 as u128));

    // buy ETH, take the reverse order
    let take_order = RequestOrder {
        token_sell: usdc_token_contract.get_account_id(),
        token_buy: eth_token_contract.get_account_id(),
        amount_sell: U128::from(2150000000 as u128),
        amount_buy: U128::from(100000000 as u128),
        fill_buy_or_sell: false,
        filled: U128::from(0),
    };
    let maker_orders = vec![OrderKeyInfo{
        bot_id: next_bot_id.clone(),
        forward_or_reverse: false,
        level: 14,
    }];
    let taker_request = TakeRequest {
        take_order: take_order.clone(),
        maker_orders,
        return_near: Some(true),
    };
    let taker_request_str = serde_json::to_string(&(taker_request)).unwrap();
    check_success(usdc_token_contract.ft_transfer_call(&taker_account, &gridbot_contract.get_account_id(), take_order.amount_sell.clone().0, taker_request_str).await);

    // query order
    let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 14).await?.unwrap();
    let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
    println!("second reverse level 14 forward order:{}", order_string);
    require!(forward_order_result.order.filled == U256C::from(100000000 as u128));
    require!(forward_order_result.order.amount_sell == U256C::from(4280000000 as u128));
    require!(forward_order_result.order.amount_buy == U256C::from(200000000 as u128));

    let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 14).await?.unwrap();
    let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
    println!("second reverse level 14 reverse order:{}", order_string);
    require!(reverse_order_result.order.filled == U256C::from(100000000 as u128));
    require!(reverse_order_result.order.amount_sell == U256C::from(100000000 as u128));
    require!(reverse_order_result.order.amount_buy == U256C::from(2150000000 as u128));
    // query bot
    let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
    let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
    println!("grid_bot after second reverse:{}", grid_bot_str);
    require!(grid_bot.revenue == U256C::from(9900000 as u128));
    require!(grid_bot.total_base_amount == U256C::from(1000000000 as u128));
    require!(grid_bot.total_quote_amount == U256C::from(31059900000 as u128));
    // query protocol fee
    let protocol_fee_usdc = gridbot_contract.query_protocol_fee(usdc_token_contract.get_account_id()).await?;
    println!("protocol_fee_usdc:{}", protocol_fee_usdc.0.to_string());
    // require!(protocol_fee_usdc == U256C::from(100000 as u128));

    let global_usdc_second = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
    // 100000011070000 - 99997861070000 = 2150000000
    println!("after second reverse:global_usdc_second:{}", global_usdc_second.0.to_string());
    require!(global_usdc_second.0 - global_usdc.0 == (2150000000 as u128));
    let maker_usdc_balance_second = usdc_token_contract.ft_balance_of(&maker_account).await?;
    println!("after second reverse:maker maker_usdc_balance_second:{}", maker_usdc_balance_second.0.to_string());
    let maker_eth_balance_second = eth_token_contract.ft_balance_of(&maker_account).await?;
    println!("after second reverse:maker maker_eth_balance_second:{}", maker_eth_balance_second.0.to_string());
    let taker_usdc_balance_second = usdc_token_contract.ft_balance_of(&taker_account).await?;
    println!("after second reverse:taker taker_usdc_balance_second{}", taker_usdc_balance_second.0.to_string());
    // 700035869905000-700038019905000=-2150000000
    require!(taker_usdc_balance.0 - taker_usdc_balance_second.0 == (2150000000 as u128));
    let taker_eth_balance_second = eth_token_contract.ft_balance_of(&taker_account).await?;
    println!("after second reverse:taker taker_eth_balance_second{}", taker_eth_balance_second.0.to_string());
    // 69999999999999249900000 - 69999999999999149950000=99950000=100000000*(1000000-500)/1000000=99950000
    require!(taker_eth_balance_second.0 - taker_eth_balance.0 == (99950000 as u128));

    // Partial filled
    let take_order = RequestOrder {
        token_sell: eth_token_contract.get_account_id(),
        token_buy: usdc_token_contract.get_account_id(),
        amount_sell: U128::from(50000000 as u128),
        amount_buy: U128::from(1070000000 as u128),
        fill_buy_or_sell: false,
        filled: U128::from(0),
    };
    let maker_orders = vec![OrderKeyInfo{
        bot_id: next_bot_id.clone(),
        forward_or_reverse: true,
        level: 14,
    }];
    let taker_request = TakeRequest {
        take_order: take_order.clone(),
        maker_orders,
        return_near: Some(true),
    };
    let taker_request_str = serde_json::to_string(&(taker_request)).unwrap();
    check_success(eth_token_contract.ft_transfer_call(&taker_account, &gridbot_contract.get_account_id(), take_order.amount_sell.clone().0, taker_request_str).await);

    // query order
    let forward_order_result = gridbot_contract.query_order(next_bot_id.clone(), true, 14).await?.unwrap();
    let order_string = serde_json::to_string(&(forward_order_result.order)).unwrap();
    println!("Third Partial forward filled level 14 forward order:{}", order_string);
    require!(forward_order_result.order.filled == U256C::from(150000000 as u128));

    let reverse_order_result = gridbot_contract.query_order(next_bot_id.clone(), false, 14).await?.unwrap();
    let order_string = serde_json::to_string(&(reverse_order_result.order)).unwrap();
    println!("Third Partial forward filled level 14 reverse order:{}", order_string);
    require!(reverse_order_result.order.amount_sell == U256C::from(150000000 as u128));
    require!(reverse_order_result.order.amount_buy == U256C::from(3225000000 as u128));

    // query bot
    let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
    let grid_bot_str = serde_json::to_string(&(grid_bot)).unwrap();
    println!("grid_bot after third forward:{}", grid_bot_str);
    require!(grid_bot.total_base_amount == U256C::from(1050000000 as u128));
    require!(grid_bot.total_quote_amount == U256C::from(29989900000 as u128));

    let global_usdc_third = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
    println!("after third forward:global_usdc_third{}", global_usdc_third.0.to_string());
    // 100000011070000 - 99998941605000=1069465000=1070000000*(1000000-500)/1000000=1069465000
    require!(global_usdc_second.0 - global_usdc_third.0 == (1069465000 as u128));
    let maker_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
    println!("after third forward:maker maker_usdc_balance{}", maker_usdc_balance.0.to_string());
    let maker_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
    println!("after third forward:maker maker_eth_balance{}", maker_eth_balance.0.to_string());
    let taker_usdc_balance_third = usdc_token_contract.ft_balance_of(&taker_account).await?;
    println!("after third forward:taker taker_usdc_balance_third:{}", taker_usdc_balance_third.0.to_string());
    // 700036939370000-700035869905000=1069465000
    require!(taker_usdc_balance_third.0 - taker_usdc_balance_second.0 == (1069465000 as u128));
    let taker_eth_balance_third = eth_token_contract.ft_balance_of(&taker_account).await?;
    println!("after third forward:taker taker_eth_balance_third:{}", taker_eth_balance_third.0.to_string());
    // 69999999999999249900000
    // 69999999999999199900000
    // 50000000
    require!(taker_eth_balance_second.0 - taker_eth_balance_third.0 == (50000000 as u128));

    // user claim revenue, any user can claim, but revenue just send to bot's owner
    // check_success(gridbot_contract.claim(&maker_account, next_bot_id.clone()).await);
    check_success(gridbot_contract.claim(&taker_account, next_bot_id.clone()).await);
    // query bot
    let grid_bot = gridbot_contract.query_bot(next_bot_id.clone()).await?.unwrap();
    require!(grid_bot.revenue == U256C::from(0));

    let global_usdc = gridbot_contract.query_global_balance(usdc_token_contract.get_account_id()).await?.unwrap();
    println!("after claim:global_usdc{}", global_usdc.0.to_string());
    // 99998941605000-99998931705000=9900000
    // 99998931705000 -99998941605000
    require!(global_usdc_third.0 - global_usdc.0 == (9900000 as u128));
    let global_eth = gridbot_contract.query_global_balance(eth_token_contract.get_account_id()).await?.unwrap();
    println!("after claim:global_eth{}", global_eth.0.to_string());
    // 100000000 - 100000000 + 50000(taker fee) + 50000000
    // require!(global_eth.0 == (10000000000000050050000 as u128));
    // check grid bot balance
    let gridbot_usdc_balance = usdc_token_contract.ft_balance_of(gridbot_contract.0.as_account()).await?;
    println!("after claim:gridbot_usdc_balance{}", gridbot_usdc_balance.0.to_string());
    require!(gridbot_usdc_balance.0 == global_usdc.0);
    let gridbot_eth_balance = eth_token_contract.ft_balance_of(gridbot_contract.0.as_account()).await?;
    println!("after claim:gridbot_eth_balance{}", gridbot_eth_balance.0.to_string());
    require!(gridbot_eth_balance.0 == global_eth.0);

    let user_usdc_balance = usdc_token_contract.ft_balance_of(&maker_account).await?;
    println!("after claim:maker user_usdc:{}", user_usdc_balance.0.to_string());
    // 100000009900000 + 9900000 = 100000019800000
    require!(user_usdc_balance.0 - maker_usdc_balance.0 == (9900000 as u128));
    let user_eth_balance = eth_token_contract.ft_balance_of(&maker_account).await?;
    println!("after claim:maker user_eth:{}", user_eth_balance.0.to_string());

    // withdraw_protocol_fee
    let owner_usdc_balance_before = usdc_token_contract.ft_balance_of(&owner).await?;
    println!("before withdraw protocol fee:maker owner_usdc_balance_before:{}", owner_usdc_balance_before.0.to_string());
    let owner_eth_balance_before = eth_token_contract.ft_balance_of(&owner).await?;
    println!("before withdraw protocol fee:maker owner_eth_balance_before:{}", owner_eth_balance_before.0.to_string());
    // storage deposit
    let owner_id = AccountId::from_str(owner.id()).expect("Invalid AccountId");
    check_success(usdc_token_contract.ft_storage_deposit(&owner_id).await);

    check_success(gridbot_contract.withdraw_protocol_fee(&owner, usdc_token_contract.get_account_id(), (AccountId::from_str(owner.id()).expect("Invalid AccountId")), U128::from(100000)).await);
    let owner_usdc_balance = usdc_token_contract.ft_balance_of(&owner).await?;
    println!("after withdraw protocol fee:maker user_usdc:{}", owner_usdc_balance.0.to_string());
    require!(owner_usdc_balance.0 - owner_usdc_balance_before.0 == (100000 as u128));
    let owner_eth_balance = eth_token_contract.ft_balance_of(&owner).await?;
    println!("after withdraw protocol fee:maker user_eth:{}", owner_eth_balance.0.to_string());
    require!(owner_eth_balance_before.0 == owner_eth_balance.0);

    Ok(())
}
