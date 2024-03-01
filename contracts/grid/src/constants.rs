use near_sdk::{Balance, Gas};

/// Attach no deposit.
pub const NO_DEPOSIT: Balance = 0;
pub const ONE_YOCTO: Balance = 1;

pub const GAS_FOR_FT_TRANSFER: Gas = Gas(Gas::ONE_TERA.0 * 10);
pub const GAS_FOR_AFTER_FT_TRANSFER: Gas = Gas(Gas::ONE_TERA.0 * 20);
pub const GAS_FOR_CREATE_BOT_AFTER_NEAR: Gas = Gas(Gas::ONE_TERA.0 * 250);
pub const GAS_FOR_CREATE_BOT_AFTER_ORACLE: Gas = Gas(Gas::ONE_TERA.0 * 200);
pub const GAS_FOR_AFTER_ORACLE: Gas = Gas(Gas::ONE_TERA.0 * 20);

/// slippage denominator
pub const SLIPPAGE_DENOMINATOR: u16 = 10000;

/// grid rate denominator
pub const GRID_RATE_DENOMINATOR: u16 = 10000;

pub const DEFAULT_PROTOCOL_FEE: u128 = 10000;
pub const DEFAULT_TAKER_FEE: u128 = 500;
pub const MAX_PROTOCOL_FEE: u128 = 100000;
/// protocol fee denominator
pub const PROTOCOL_FEE_DENOMINATOR: u128 = 1000000;

pub const BASE_CREATE_STORAGE_FEE: u128 = 10_000_000_000_000_000_000_000; // 0.01Near
pub const PER_GRID_STORAGE_FEE: u128 = 3_600_000_000_000_000_000_000; // 0.0036Near

/// self.order_map[bot_id][FORWARD_ORDERS_INDEX]
pub const FORWARD_ORDERS_INDEX: u64 = 0;
/// self.order_map[bot_id][REVERSE_ORDERS_INDEX]
pub const REVERSE_ORDERS_INDEX: u64 = 1;
/// Forward and Reverse
pub const ORDER_POSITION_SIZE: u64 = 2;

pub const DEFAULT_TOKEN_STORAGE_FEE: u128 = 1_00_000_000_000_000_000_000_000; // 0.1Near

pub const PAIR_TOKEN_LENGTH: usize = 2;

// ms
pub const DEFAULT_ORACLE_VALID_TIME: u64 = 90000;

pub const PRICE_DENOMINATOR: u128 = 1_000_000_000_000_000_000;

pub const MAX_GRID_COUNT: u16 = 300;

/// Price per 1 byte of storage from mainnet genesis config.
pub const STORAGE_PRICE_PER_BYTE: Balance = 10_000_000_000_000_000_000;
