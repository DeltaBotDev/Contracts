use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::json_types::U128;

mod utils;
mod constants;
mod errors;
mod entity;
mod grid_bot;
mod orderbook;
mod grid_bot_internal;
mod token;
mod orderbook_internal;
mod grid_bot_views;
mod orderbook_views;
mod big_decimal;
mod events;
mod grid_bot_private;
mod grid_bot_get_set;
mod grid_bot_asset;
mod owner;
mod oracle;
mod wnear;

pub use crate::constants::*;
pub use crate::errors::*;
pub use crate::utils::*;
pub use crate::entity::*;
pub use crate::oracle::*;

// near_sdk::setup_alloc!();
// near_sdk::wee_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct GridBotContract {
    pub owner_id: AccountId,
    pub oracle: AccountId,
    pub oracle_valid_time: u64,
    pub status: GridStatus,
    /// real_protocol_fee = protocol_fee / 1000000
    pub protocol_fee_rate: u128,
    pub taker_fee_rate: u128,
    /// bot_map[bot_id] = bot
    /// bot_id = GRID:index
    pub bot_map: LookupMap<String, GridBot>,
    /// order_map[bot_id][0][0] = first forward order; order_map[bot_id][1][0] = first reverse order;
    pub order_map: LookupMap<String, OrdersStorage>,
    /// start from 0, used from 1
    pub next_bot_id: u128,
    // /// oracle_price_map[pair_id] = OraclePrice
    // pub oracle_price_map: LookupMap<String, OraclePrice>,
    /// pair_map[base_token_addr+":"+quote_token_addr] = Pair
    pub pair_map: LookupMap<String, Pair>,
    pub protocol_fee_map: LookupMap<AccountId, U256C>,
    // pub storage_fee: u128,
    pub global_balances_map: LookupMap<AccountId, U256C>,
    pub deposit_limit_map: LookupMap<AccountId, U256C>,
    pub user_balances_map: LookupMap<AccountId, LookupMap<AccountId, U256C>>,
    pub user_locked_balances_map: LookupMap<AccountId, LookupMap<AccountId, U256C>>,
    // pub user_withdraw_failed_map: LookupMap<AccountId, LookupMap<AccountId, U256C>>,
    pub market_user_map: LookupMap<AccountId, bool>,
    pub wnear: AccountId,
    /// post refer info and other things
    pub operator_id: AccountId,
    /// refer_recommender_user_map[recommender] = users
    pub refer_recommender_user_map: LookupMap<AccountId, Vector<AccountId>>,
    /// refer_user_recommender_map[user] = user's recommender
    pub refer_user_recommender_map: LookupMap<AccountId, AccountId>,
    /// refer_fee_map[user][token] = balance
    pub refer_fee_map: LookupMap<AccountId, LookupMap<AccountId, U128>>,
    /// refer_fee_rate[0] = first level, refer_fee_rate[1] = second level
    pub refer_fee_rate: Vec<u32>,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldGridBotContract {
    pub owner_id: AccountId,
    pub oracle: AccountId,
    pub oracle_valid_time: u64,
    pub status: GridStatus,
    /// real_protocol_fee = protocol_fee / 1000000
    pub protocol_fee_rate: u128,
    pub taker_fee_rate: u128,
    /// bot_map[bot_id] = bot
    /// bot_id = GRID:index
    pub bot_map: LookupMap<String, GridBot>,
    /// order_map[bot_id][0][0] = first forward order; order_map[bot_id][1][0] = first reverse order;
    pub order_map: LookupMap<String, OrdersStorage>,
    /// start from 0, used from 1
    pub next_bot_id: u128,
    // /// oracle_price_map[pair_id] = OraclePrice
    // pub oracle_price_map: LookupMap<String, OraclePrice>,
    /// pair_map[base_token_addr+":"+quote_token_addr] = Pair
    pub pair_map: LookupMap<String, Pair>,
    pub protocol_fee_map: LookupMap<AccountId, U256C>,
    // pub storage_fee: u128,
    pub global_balances_map: LookupMap<AccountId, U256C>,
    pub deposit_limit_map: LookupMap<AccountId, U256C>,
    pub user_balances_map: LookupMap<AccountId, LookupMap<AccountId, U256C>>,
    pub user_locked_balances_map: LookupMap<AccountId, LookupMap<AccountId, U256C>>,
    // pub user_withdraw_failed_map: LookupMap<AccountId, LookupMap<AccountId, U256C>>,
    pub market_user_map: LookupMap<AccountId, bool>,
    pub wnear: AccountId,
}

#[near_bindgen]
impl GridBotContract {
    #[init]
    pub fn new(owner_id: AccountId, oracle: AccountId, wnear: AccountId) -> Self {
        assert!(!env::state_exists());
        GridBotContract {
            owner_id: owner_id.clone(),
            oracle,
            oracle_valid_time: DEFAULT_ORACLE_VALID_TIME,
            status: GridStatus::Running,
            // 1%
            protocol_fee_rate: DEFAULT_PROTOCOL_FEE,
            taker_fee_rate: DEFAULT_TAKER_FEE,
            bot_map: LookupMap::new(b"bots".to_vec()),
            // order_map: LookupMap::new(b"orders".to_vec()),
            order_map: LookupMap::new(b"orders_storage".to_vec()),
            next_bot_id: 1,
            // oracle_price_map: LookupMap::new(b"oracle".to_vec()),
            pair_map: LookupMap::new(b"pairs".to_vec()),
            protocol_fee_map: LookupMap::new(b"protocol".to_vec()),
            // storage_fee: 0,
            global_balances_map: LookupMap::new(b"global".to_vec()),
            deposit_limit_map: LookupMap::new(b"deposit_limit".to_vec()),
            user_balances_map: LookupMap::new(StorageKey::UserBalanceMainKey),
            user_locked_balances_map: LookupMap::new(StorageKey::UserLockedBalanceMainKey),
            // user_withdraw_failed_map: LookupMap::new(StorageKey::WithdrawFailedMainKey),
            market_user_map: LookupMap::new(b"market_users".to_vec()),
            wnear,
            operator_id: owner_id,
            refer_recommender_user_map: LookupMap::new(b"rec_users".to_vec()),
            refer_user_recommender_map: LookupMap::new(b"user_rec".to_vec()),
            refer_fee_map: LookupMap::new(StorageKey::ReferFeeMainKey),
            refer_fee_rate: vec![],
        }
    }
}
