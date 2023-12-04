use near_sdk::{AccountId, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::cmp::{PartialEq, Eq};
use crate::utils::{U128C};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub enum GridStatus {
    Running = 0,
    Paused = 1,
    Shutdown = 2,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum GridType {
    EqOffset = 0,
    EqRate = 1,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct GridBot {
    pub active: bool,
    pub user: AccountId,
    pub bot_id: String,
    pub closed: bool,
    pub name: String,
    pub pair_id: String,
    pub grid_type: GridType,
    pub grid_sell_count: u16,
    pub grid_buy_count: u16,
    /// real_grid_rate = grid_rate / 10000
    pub grid_rate: u16,
    pub grid_offset: U128C,
    pub first_base_amount: U128C,
    pub first_quote_amount: U128C,
    pub last_base_amount: U128C,
    pub last_quote_amount: U128C,
    pub fill_base_or_quote: bool,
    /// real_trigger_price = trigger_price / 10^18
    pub trigger_price: U128C,
    /// eg: trigger_price=100, current_price=50, then trigger_price_above_or_below = true
    /// eg: trigger_price=100, current_price=200, then trigger_price_above_or_below = false
    pub trigger_price_above_or_below: bool,
    /// real_take_profit_price = take_profit_price / 10^18
    pub take_profit_price: U128C,
    /// real_stop_loss_price = stop_loss_price / 10^18
    pub stop_loss_price: U128C,
    pub valid_until_time: u64,
    pub total_quote_amount: Balance,
    pub total_base_amount: Balance,
    pub revenue: Balance,
}

impl Clone for GridBot {
    fn clone(&self) -> Self {
        GridBot {
            active: self.active.clone(),
            user: self.user.clone(),
            bot_id: self.bot_id.clone(),
            closed: self.closed.clone(),
            name: self.name.clone(),
            pair_id: self.pair_id.clone(),
            grid_type: self.grid_type.clone(),
            grid_sell_count: self.grid_sell_count.clone(),
            grid_buy_count: self.grid_buy_count.clone(),
            grid_rate: self.grid_rate.clone(),
            grid_offset: self.grid_offset.clone(),
            first_base_amount: self.first_base_amount.clone(),
            first_quote_amount: self.first_quote_amount.clone(),
            last_base_amount: self.last_base_amount.clone(),
            last_quote_amount: self.last_quote_amount.clone(),
            fill_base_or_quote: self.fill_base_or_quote.clone(),
            trigger_price: self.trigger_price.clone(),
            trigger_price_above_or_below: self.trigger_price_above_or_below.clone(),
            take_profit_price: self.take_profit_price.clone(),
            stop_loss_price: self.stop_loss_price.clone(),
            valid_until_time: self.valid_until_time.clone(),
            total_quote_amount: self.total_quote_amount.clone(),
            total_base_amount: self.total_base_amount.clone(),
            revenue: self.revenue.clone(),
        }
    }
}


#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Order {
    /// order_id: botId-0/1-level
    pub order_id: String,
    pub token_sell: AccountId,
    pub token_buy: AccountId,
    pub amount_sell: U128C,
    pub amount_buy: U128C,
    pub fill_buy_or_sell: bool,
    pub filled: U128C,
}

impl Clone for Order {
    fn clone(&self) -> Self {
        Order {
            order_id: self.order_id.clone(),
            token_sell: self.token_sell.clone(),
            token_buy: self.token_buy.clone(),
            amount_sell: self.amount_sell.clone(),
            amount_buy: self.amount_buy.clone(),
            fill_buy_or_sell: self.fill_buy_or_sell,
            filled: self.filled.clone(),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct OrderKeyInfo {
    pub bot_id: String,
    pub forward_or_reverse: bool,
    pub level: usize,
}

impl Clone for OrderKeyInfo {
    fn clone(&self) -> Self {
        OrderKeyInfo {
            bot_id: self.bot_id.clone(),
            forward_or_reverse: self.forward_or_reverse.clone(),
            level: self.level.clone(),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OraclePrice {
    pub valid_timestamp: u64,
    pub pair_id: U128C,
    pub price: U128C,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pair {
    // pub pair_id: U128C,
    pub base_token: AccountId,
    pub quote_token: AccountId,
}

impl Clone for Pair {
    fn clone(&self) -> Self {
        Pair {
            base_token: self.base_token.clone(),
            quote_token: self.quote_token.clone(),
        }
    }
}
