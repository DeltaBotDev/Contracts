use crate::*;
use near_sdk::{env};
use crate::{GridBotContract, SLIPPAGE_DENOMINATOR};
use crate::big_decimal::BigDecimal;
use crate::oracle::{Price};

impl GridBotContract {

    pub fn internal_check_oracle_price(&self, entry_price: U256C, base_price: Price, quote_price: Price, slippage: u16) -> bool {
        if base_price.publish_time as u64 * 1000 + self.oracle_valid_time.clone() < env::block_timestamp_ms() {
            return false;
        }
        if quote_price.publish_time as u64 * 1000 + self.oracle_valid_time.clone() < env::block_timestamp_ms() {
            return false;
        }
        let oracle_pair_price = (BigDecimal::from(base_price.price.0 as u64) / BigDecimal::from(quote_price.price.0 as u64) * BigDecimal::from(PRICE_DENOMINATOR)).round_down_u128();

        if entry_price.as_u128() >= oracle_pair_price {
            return (entry_price.as_u128() - oracle_pair_price) * SLIPPAGE_DENOMINATOR as u128 / entry_price.as_u128() <= slippage as u128;
        } else {
            return (oracle_pair_price - entry_price.as_u128()) * SLIPPAGE_DENOMINATOR as u128 / entry_price.as_u128() <= slippage  as u128;
        }
    }

    pub fn internal_check_bot_amount(&mut self, grid_sell_count: u16, grid_buy_count: u16, first_base_amount_256: U256C, first_quote_amount_256: U256C,
                                     last_base_amount_256: U256C, last_quote_amount_256: U256C, pair: &Pair, base_amount_sell: U256C, quote_amount_buy: U256C) -> (bool, String) {
        if grid_sell_count > 0 && grid_buy_count > 0 {
            // require!(last_quote_amount_256 * first_base_amount_256 > first_quote_amount_256 * last_base_amount_256 , INVALID_FIRST_OR_LAST_AMOUNT);
            if last_quote_amount_256 * first_base_amount_256 <= first_quote_amount_256 * last_base_amount_256 {
                return (false, INVALID_FIRST_OR_LAST_AMOUNT.to_string());
            }
        }
        if grid_sell_count > 0 {
            // require!(first_base_amount_256.as_u128() > 0 && first_quote_amount_256.as_u128() > 0, INVALID_FIRST_OR_LAST_AMOUNT);
            // if first_base_amount_256.as_u128() == 0 || first_quote_amount_256.as_u128() == 0 {
            if last_base_amount_256.as_u128() == 0 || last_quote_amount_256.as_u128() == 0 {
                return (false, INVALID_FIRST_OR_LAST_AMOUNT.to_string());
            }
            // require!(base_amount_sell.as_u128() / grid_sell_count as u128 >= self.deposit_limit_map.get(&pair.base_token).unwrap().as_u128(), BASE_TO_SMALL);
            if (base_amount_sell.as_u128() / grid_sell_count as u128) < self.deposit_limit_map.get(&pair.base_token).unwrap().as_u128() {
                return (false, BASE_TOO_SMALL.to_string());
            }
        }
        if grid_buy_count > 0 {
            // require!(last_base_amount_256.as_u128() > 0 && last_quote_amount_256.as_u128() > 0, INVALID_FIRST_OR_LAST_AMOUNT);
            // if last_base_amount_256.as_u128() == 0 || last_quote_amount_256.as_u128() == 0 {
            if first_base_amount_256.as_u128() == 0 || first_quote_amount_256.as_u128() == 0 {
                return (false, INVALID_FIRST_OR_LAST_AMOUNT.to_string());
            }
            // require!(quote_amount_buy.as_u128() / grid_buy_count as u128 >= self.deposit_limit_map.get(&pair.quote_token).unwrap().as_u128(), QUOTE_TO_SMALL);
            if (quote_amount_buy.as_u128() / grid_buy_count as u128) < self.deposit_limit_map.get(&pair.quote_token).unwrap().as_u128() {
                return (false, QUOTE_TOO_SMALL.to_string());
            }
        }
        return (true, "".to_string());
    }

    pub fn internal_check_bot_close_permission(&self, base_price: Price, quote_price: Price, bot: &GridBot) -> bool {
        if base_price.publish_time as u64 * 1000 + self.oracle_valid_time.clone() < env::block_timestamp_ms() {
            return false;
        }
        if quote_price.publish_time as u64 * 1000 + self.oracle_valid_time.clone() < env::block_timestamp_ms() {
            return false;
        }
        // base_price = usd amount / base amount
        // quote_price = usd amount / quote amount
        // oracle_pair_price = quote amount / base amount = base_price / quote_price
        let oracle_pair_price = (BigDecimal::from(base_price.price.0 as u64) / BigDecimal::from(quote_price.price.0 as u64) * BigDecimal::from(PRICE_DENOMINATOR)).round_down_u128();
        if oracle_pair_price >= bot.take_profit_price.as_u128() {
            return true;
        }
        if oracle_pair_price <= bot.stop_loss_price.as_u128() {
            return true;
        }
        return false;
    }
}
