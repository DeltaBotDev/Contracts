use near_sdk::{AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::cmp::{PartialEq, Eq};
use crate::utils::{U256C};
use near_sdk::BorshStorageKey;
use near_sdk::collections::Vector;
use near_sdk::json_types::U128;
use crate::oracle::PriceIdentifier;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Eq, Clone)]
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

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GridBot {
    pub name: String,
    pub active: bool,
    pub user: AccountId,
    pub bot_id: String,
    pub closed: bool,
    pub pair_id: String,
    pub grid_type: GridType,
    pub grid_sell_count: u16,
    pub grid_buy_count: u16,
    /// real_grid_rate = grid_rate / 10000
    pub grid_rate: u16,
    pub grid_offset: U256C,
    pub first_base_amount: U256C,
    pub first_quote_amount: U256C,
    pub last_base_amount: U256C,
    pub last_quote_amount: U256C,
    pub fill_base_or_quote: bool,
    /// real_trigger_price = trigger_price / 10^18
    pub trigger_price: U256C,
    /// eg: trigger_price=100, current_price=50, then trigger_price_above_or_below = true
    /// eg: trigger_price=100, current_price=200, then trigger_price_above_or_below = false
    pub trigger_price_above_or_below: bool,
    /// real_take_profit_price = take_profit_price / 10^18
    pub take_profit_price: U256C,
    /// real_stop_loss_price = stop_loss_price / 10^18
    pub stop_loss_price: U256C,
    pub valid_until_time: U256C,
    pub total_quote_amount: U256C,
    pub total_base_amount: U256C,
    pub revenue: U256C,
    pub total_revenue: U256C,
}

impl Clone for GridBot {
    fn clone(&self) -> Self {
        GridBot {
            name: self.name.clone(),
            active: self.active.clone(),
            user: self.user.clone(),
            bot_id: self.bot_id.clone(),
            closed: self.closed.clone(),
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
            total_revenue: self.total_revenue.clone(),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Order {
    pub token_sell: AccountId,
    pub token_buy: AccountId,
    pub amount_sell: U256C,
    pub amount_buy: U256C,
    pub fill_buy_or_sell: bool,
    pub filled: U256C,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OrdersStorage {
    pub forward_orders: Vector<Order>,
    pub reverse_orders: Vector<Order>,
}

impl Default for Order {
    fn default() -> Self {
        Order {
            token_sell: AccountId::new_unchecked("alice".to_string()),
            token_buy: AccountId::new_unchecked("alice".to_string()),
            amount_sell: Default::default(),
            amount_buy: Default::default(),
            fill_buy_or_sell: false,
            filled: Default::default(),
        }
    }
}
impl Clone for Order {
    fn clone(&self) -> Self {
        Order {
            token_sell: self.token_sell.clone(),
            token_buy: self.token_buy.clone(),
            amount_sell: self.amount_sell.clone(),
            amount_buy: self.amount_buy.clone(),
            fill_buy_or_sell: self.fill_buy_or_sell.clone(),
            filled: self.filled.clone(),
        }
    }
}

impl Order {
    pub fn to_request_order(&self) -> RequestOrder {
        RequestOrder {
            token_sell: self.token_sell.clone(),
            token_buy: self.token_buy.clone(),
            amount_sell: U128::from(self.amount_sell.clone().as_u128()),
            amount_buy: U128::from(self.amount_buy.clone().as_u128()),
            fill_buy_or_sell: self.fill_buy_or_sell.clone(),
            filled: U128::from(self.filled.clone().as_u128()),
        }
    }
}
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RequestOrder {
    pub token_sell: AccountId,
    pub token_buy: AccountId,
    pub amount_sell: U128,
    pub amount_buy: U128,
    pub fill_buy_or_sell: bool,
    pub filled: U128,
}
impl RequestOrder {
    pub fn to_order(&self) -> Order {
        Order {
            token_sell: self.token_sell.clone(),
            token_buy: self.token_buy.clone(),
            amount_sell: U256C::from(self.amount_sell.clone().0),
            amount_buy: U256C::from(self.amount_buy.clone().0),
            fill_buy_or_sell: self.fill_buy_or_sell.clone(),
            filled: U256C::from(self.filled.clone().0),
        }
    }
}
impl Clone for RequestOrder {
    fn clone(&self) -> Self {
        RequestOrder {
            token_sell: self.token_sell.clone(),
            token_buy: self.token_buy.clone(),
            amount_sell: self.amount_sell.clone(),
            amount_buy: self.amount_buy.clone(),
            fill_buy_or_sell: self.fill_buy_or_sell.clone(),
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
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct OrderResult {
    pub order: Order,
    pub flag: bool,
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

// #[derive(BorshDeserialize, BorshSerialize)]
// pub struct OraclePrice {
//     pub valid_timestamp: u64,
//     pub price: U256C,
// }

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Pair {
    pub base_token: AccountId,
    pub quote_token: AccountId,
    pub base_oracle_id: PriceIdentifier,
    pub quote_oracle_id: PriceIdentifier,
}

impl Clone for Pair {
    fn clone(&self) -> Self {
        Pair {
            base_token: self.base_token.clone(),
            quote_token: self.quote_token.clone(),
            base_oracle_id: self.base_oracle_id.clone(),
            quote_oracle_id: self.quote_oracle_id.clone(),
        }
    }
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    UserBalanceMainKey,
    UserBalanceSubKey(AccountId),
    UserLockedBalanceMainKey,
    UserLockedBalanceSubKey(AccountId),
    OrdersMainKey,
    OrdersSubKey(u64),
    ReferFeeMainKey,
    ReferFeeSubKey(AccountId),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TakeRequest {
    pub take_order: RequestOrder,
    pub maker_orders: Vec<OrderKeyInfo>,
    pub return_near: Option<bool>,
}
