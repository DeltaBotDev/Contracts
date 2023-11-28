use crate::U128C;

/// Attach no deposit.
pub const NO_DEPOSIT: u128 = 0;

/// slippage denominator
pub const SLIPPAGE_DENOMINATOR: u16 = 10000;

/// grid rate denominator
pub const GRID_RATE_DENOMINATOR: u16 = 10000;

/// self.order_map[bot_id][FORWARD_ORDERS_INDEX]
pub const FORWARD_ORDERS_INDEX: usize = 0;
/// self.order_map[bot_id][REVERSE_ORDERS_INDEX]
pub const REVERSE_ORDERS_INDEX: usize = 1;
/// Forward and Reverse
pub const ORDER_POSITION_SIZE: u16 = 2;

