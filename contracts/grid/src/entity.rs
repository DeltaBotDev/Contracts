use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::cmp::{PartialEq, Eq};
use crate::utils::{U256C, U128C};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub enum GridStatus {
    Running = 0,
    Paused = 1,
    Shutdown = 2,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum GridType {
    EqOffset = 0,
    EqRate = 1,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct GridBot {
    pub user: AccountId,
    pub bot_id: String,
    pub closed: bool,
    pub name: String,
    pub pair_id: U128C,
    pub grid_type: GridType,
    pub grid_count: u16,
    /// real_grid_rate = grid_rate / 10000
    pub grid_rate: u16,
    pub grid_offset: U256C,
    pub first_base_amount: U256C,
    pub first_quote_amount: U256C,
    pub last_base_amount: U256C,
    pub last_quote_amount: U256C,
    pub fill_base_or_quote: u8,
    /// real_trigger_price = trigger_price / 10^18
    pub trigger_price: U256C,
    /// real_take_profit_price = take_profit_price / 10^18
    pub take_profit_price: U256C,
    /// real_stop_loss_price = stop_loss_price / 10^18
    pub stop_loss_price: U256C,
    pub valid_until_time: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Order {
    pub token_sell: U256C,
    pub token_buy: U256C,
    pub amount_sell: U256C,
    pub amount_buy: U256C,
    pub fill_buy_or_sell: u8,
    pub filled: U256C,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OraclePrice {
    pub valid_timestamp: u64,
    pub pair_id: U128C,
    pub price: U256C,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pair {
    pub pair_id: U128C,
    pub base_token: AccountId,
    pub quote_token: AccountId,
}
