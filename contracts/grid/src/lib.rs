// use near_contract_standards::fungible_token::metadata::{
//     FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
// };

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};
use std::collections::HashMap;
// use std::panic::PanicInfo;

mod utils;
mod constants;
mod errors;
mod entity;
mod grid_bot;
mod orderbook;
mod storage;
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

pub use crate::constants::*;
pub use crate::errors::*;
pub use crate::utils::*;
pub use crate::entity::{GridBot, Order};
use crate::entity::{GridStatus, OraclePrice, Pair};

// near_sdk::setup_alloc!();
// near_sdk::wee_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct GridBotContract {
    pub owner_id: AccountId,
    pub status: GridStatus,
    /// real_protocol_fee = protocol_fee / 1000000
    pub protocol_fee_rate: u128,
    /// bot_map[bot_id] = bot
    pub bot_map: HashMap<String, GridBot>,
    /// order_map[bot_id][0][0] = first forward order; order_map[bot_id][1][0] = first reverse order;
    pub order_map: HashMap<String, Vec<Vec<Order>>>,
    /// start from 0, used from 1
    pub next_bot_id: u128,
    /// oracle_price_map[pair_id] = OraclePrice
    pub oracle_price_map: HashMap<String, OraclePrice>,
    /// pair_map[base_token_addr+":"+quote_token_addr] = Pair
    pub pair_map: HashMap<String, Pair>,
    pub protocol_fee_map: HashMap<AccountId, U128C>,
    pub global_balances_map: HashMap<AccountId, U128C>,
    pub user_balances_map: HashMap<AccountId, HashMap<AccountId, U128C>>,
    pub user_locked_balances_map: HashMap<AccountId, HashMap<AccountId, U128C>>,
}

#[near_bindgen]
impl GridBotContract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        GridBotContract {
            owner_id: owner_id.into(),
            // 1%
            status: GridStatus::Running,
            protocol_fee_rate: DEFAULT_PROTOCOL_FEE,
            bot_map: Default::default(),
            order_map: Default::default(),
            next_bot_id: 0,
            oracle_price_map: Default::default(),
            pair_map: Default::default(),
            protocol_fee_map: Default::default(),
            global_balances_map: Default::default(),
            user_balances_map: Default::default(),
            user_locked_balances_map: Default::default(),
        }
    }
}
