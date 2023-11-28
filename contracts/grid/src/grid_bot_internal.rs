use crate::*;
use near_sdk::env;
use crate::{GridBotContract, SLIPPAGE_DENOMINATOR, U256C};
use crate::entity::GridType;
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

    // pub fn internal_get_and_use_next_pair_id(&mut self) -> u128 {
    //     let next_id = self.next_pair_id;
    //
    //     assert_ne!(self.next_pair_id.checked_add(1), None, "VALID_NEXT_PAIR_ID");
    //
    //     self.next_pair_id += 1;
    //
    //     return next_id;
    // }

    pub fn internal_check_oracle_price(&self, entry_price: U256C, pair_id: String, slippage: u16) -> bool {
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

    pub fn internal_get_first_forward_order(grid_bot: GridBot, pair: Pair, level: usize) -> Order {
        let mut order = Order{
            order_id: level.to_string(),
            token_sell: pair.base_token.clone(),
            token_buy: pair.quote_token.clone(),
            amount_sell: U128C::from(0),
            amount_buy: U128C::from(0),
            fill_buy_or_sell: false,
            filled: U128C::from(0),
        };
        let grid_rate_denominator_128 = U128C::from(GRID_RATE_DENOMINATOR);
        if grid_bot.grid_buy_count > (level.clone() as u16) {
            // buy grid
            order.token_sell = pair.quote_token.clone();
            order.token_buy = pair.base_token.clone();
            order.fill_buy_or_sell = grid_bot.fill_base_or_quote.clone();
            if grid_bot.fill_base_or_quote {
                // fixed base
                order.amount_buy = grid_bot.first_base_amount.clone();
                order.amount_sell = if grid_bot.grid_type == EqOffset {
                    // arithmetic grid
                    grid_bot.first_quote_amount.clone() + grid_bot.grid_offset * U128C::from(level.clone() as u16)
                } else {
                    // proportional grid
                    grid_bot.first_quote_amount.clone() * (grid_rate_denominator_128 + U128C::from(grid_bot.grid_rate)).pow(U128C::from(level.clone() as u16)) / grid_rate_denominator_128.pow(U128C::from(level.clone() as u16))
                };
            } else {
                // fixed quote
                order.amount_sell = grid_bot.first_quote_amount.clone();
                order.amount_buy = if grid_bot.grid_type == EqOffset {
                    // arithmetic grid
                    grid_bot.first_base_amount.clone() - grid_bot.grid_offset * U128C::from(level.clone() as u16)
                } else {
                    // proportional grid
                    grid_bot.first_base_amount.clone() * (grid_rate_denominator_128 - U128C::from(grid_bot.grid_rate)).pow(U128C::from(level.clone() as u16)) / grid_rate_denominator_128.pow(U128C::from(level.clone() as u16))
                };
            }
        } else {
            // sell grid
            order.token_sell = pair.base_token.clone();
            order.token_buy = pair.quote_token.clone();
            order.fill_buy_or_sell = !grid_bot.fill_base_or_quote.clone();
            let coefficient = U128C::from(grid_bot.grid_buy_count.clone() + grid_bot.grid_sell_count.clone() - 1 - level.clone() as u16);
            if grid_bot.fill_base_or_quote {
                // fixed base
                order.amount_sell = grid_bot.last_base_amount.clone();
                order.amount_buy = if grid_bot.grid_type == EqOffset {
                    grid_bot.last_quote_amount.clone() - grid_bot.grid_offset * coefficient.clone()
                } else {
                    grid_bot.last_quote_amount.clone() * (grid_rate_denominator_128 - U128C::from(grid_bot.grid_rate)).pow(coefficient.clone()) / grid_rate_denominator_128.pow(coefficient.clone())
                };
            } else {
                // fixed quote
                order.amount_buy = grid_bot.last_quote_amount.clone();
                order.amount_sell = if grid_bot.grid_type == EqOffset {
                    grid_bot.last_base_amount.clone() + grid_bot.grid_offset * coefficient.clone()
                } else {
                    grid_bot.last_base_amount.clone() * (grid_rate_denominator_128 + U128C::from(grid_bot.grid_rate)).pow(coefficient.clone()) / grid_rate_denominator_128.pow(coefficient.clone())
                };
            }
        }
        return order;
    }
}
