use crate::*;
use near_sdk::env;
use crate::{GridBotContract, SLIPPAGE_DENOMINATOR, U256C};
use crate::entity::GridType::EqOffset;

impl GridBotContract {
    pub fn internal_get_next_bot_id(&self) -> u128 {
        return self.next_bot_id;
    }

    pub fn internal_get_and_use_next_bot_id(&mut self) -> u128 {
        let next_id = self.next_bot_id;

        assert_ne!(self.next_bot_id.checked_add(1), None, "VALID_NEXT_BOT_ID");

        self.next_bot_id += 1;

        return next_id;
    }

    pub fn internal_get_and_use_next_pair_id(&mut self) -> u128 {
        let next_id = self.next_pair_id;

        assert_ne!(self.next_pair_id.checked_add(1), None, "VALID_NEXT_PAIR_ID");

        self.next_pair_id += 1;

        return next_id;
    }

    pub fn internal_check_oracle_price(&self, entry_price: U256C, pair_id: U128C, slippage: u16) -> bool {
        if !self.oracle_price_map.contains_key(&pair_id) {
            return false;
        }
        let price_info = self.oracle_price_map.get(&pair_id).unwrap();
        if price_info.valid_timestamp < env::block_timestamp() {
            // oracle price expired
            return false
        }

        let recorded_price = price_info.price;
        if entry_price >= recorded_price {
            return (entry_price - recorded_price) / entry_price * SLIPPAGE_DENOMINATOR <= U256C::from(slippage);
        } else {
            return (recorded_price - entry_price) / entry_price * SLIPPAGE_DENOMINATOR <= U256C::from(slippage);
        }
    }
}
