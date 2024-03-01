use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, PromiseResult, ext_contract, require, Promise, StorageUsage};
// use near_sdk::__private::schemars::schema::SingleOrVec::Vec;
use near_sdk::json_types::{I64, U64};
use uint::hex;
use crate::{GAS_FOR_AFTER_ORACLE, GridBot, Pair, U256C};
use crate::constants::*;
use crate::errors::*;
use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Clone)]
#[repr(transparent)]
pub struct PriceIdentifier(pub [u8; 32]);

impl<'de> near_sdk::serde::Deserialize<'de> for PriceIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: near_sdk::serde::Deserializer<'de>,
    {
        /// A visitor that deserializes a hex string into a 32 byte array.
        struct IdentifierVisitor;

        impl<'de> near_sdk::serde::de::Visitor<'de> for IdentifierVisitor {
            /// Target type for either a hex string or a 32 byte array.
            type Value = [u8; 32];

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a hex string")
            }

            // When given a string, attempt a standard hex decode.
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: near_sdk::serde::de::Error,
            {
                if value.len() != 64 {
                    return Err(E::custom(format!(
                        "expected a 64 character hex string, got {}",
                        value.len()
                    )));
                }
                let mut bytes = [0u8; 32];
                hex::decode_to_slice(value, &mut bytes).map_err(E::custom)?;
                Ok(bytes)
            }
        }

        deserializer
            .deserialize_any(IdentifierVisitor)
            .map(PriceIdentifier)
    }
}

impl near_sdk::serde::Serialize for PriceIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: near_sdk::serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(&self.0))
    }
}

/// A price with a degree of uncertainty, represented as a price +- a confidence interval.
///
/// The confidence interval roughly corresponds to the standard error of a normal distribution.
/// Both the price and confidence are stored in a fixed-point numeric representation,
/// `x * (10^expo)`, where `expo` is the exponent.
//
/// Please refer to the documentation at https://docs.pyth.network/documentation/pythnet-price-feeds/best-practices for how
/// to how this price safely.
#[derive(BorshDeserialize, BorshSerialize, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub struct Price {
    pub price:        I64,
    /// Confidence interval around the price
    pub conf:         U64,
    /// The exponent
    pub expo:         i32,
    /// Unix timestamp of when this price was computed
    pub publish_time: i64,
}

impl Clone for Price {
    fn clone(&self) -> Self {
        Price {
            price: self.price.clone(),
            conf: self.conf.clone(),
            expo: self.expo.clone(),
            publish_time: self.publish_time.clone(),
        }
    }
}

#[ext_contract(ext_pyth)]
pub trait Pyth {
    fn get_price(&self, price_identifier: PriceIdentifier) -> Option<Price>;
}

impl GridBotContract {
    fn private_create_pair_price_request(&self, pair: &Pair) -> (Promise, Vec<AccountId>) {
        let identifiers = vec![pair.base_oracle_id.clone(), pair.quote_oracle_id.clone()];
        let tokens = vec![pair.base_token.clone(), pair.quote_token.clone()];
        let mut promise = ext_pyth::ext(self.oracle.clone()).get_price(identifiers[0].clone());
        for index in 1..identifiers.len() {
            promise = promise.and(ext_pyth::ext(self.oracle.clone()).get_price(identifiers[index].clone()));
        }
        return (promise, tokens);
    }

    fn private_get_price_list(&self, promise_num: usize, tokens: Vec<AccountId>) -> Vec<Price> {
        let mut price_list = vec![];
        (0..promise_num).for_each(|index|{
            let result = env::promise_result(index as u64);
            match result {
                PromiseResult::Failed => {
                    log!(format!("Failure got price, token:{}", tokens[index]));
                }
                PromiseResult::NotReady => {
                    log!(format!("Failure got price, not ready yet, token:{}", tokens[index]));
                }
                PromiseResult::Successful(value) => {
                    if let Ok(message) = near_sdk::serde_json::from_slice::<Option<Price>>(&value) {
                        if message.is_some() {
                            let price = message.unwrap();
                            // log!(format!("Success got price, token:{}, price:{}, publish_time:{}", tokens[index], price.price.0.to_string(), price.publish_time.to_string()));
                            price_list.push(price);
                        } else {
                            log!(format!("Failure got price, price empty, token:{}", tokens[index]));
                        }

                    } else {
                        log!(format!("Failure got price, error deserializing, token:{}", tokens[index]));
                    }
                }
            }
        });
        // log!("publish_time base:{}, quote:{}", price_list[0].publish_time.to_string(), price_list[1].publish_time.to_string());
        // log!("price base:{}, quote:{}", price_list[0].price.0.to_string(), price_list[1].price.0.to_string());
        return price_list;
    }

    pub fn get_price_for_create_bot(
        &mut self,
        pair: &Pair,
        user: &AccountId,
        slippage: u16,
        entry_price: &U256C,
        grid_bot: &mut GridBot,
        storage_fee: u128,
        storage_used: StorageUsage
    ) {
        let (promise, tokens) = self.private_create_pair_price_request(pair);
        promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_CREATE_BOT_AFTER_ORACLE)
                .get_price_for_create_bot_callback(tokens.len(), tokens, user, slippage, entry_price, pair, grid_bot, storage_fee, storage_used),
        );
    }

    pub fn get_price_for_close_bot(
        &mut self,
        user: &AccountId,
        pair: &Pair,
        grid_bot: &mut GridBot,
    ) {
        let (promise, tokens) = self.private_create_pair_price_request(pair);
        promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_AFTER_ORACLE)
                .get_price_for_close_bot_callback(tokens.len(), tokens, user, pair, grid_bot),
        );
    }

    pub fn get_price_for_trigger_bot(
        &mut self,
        pair: &Pair,
        grid_bot: &mut GridBot,
    ) {
        let (promise, tokens) = self.private_create_pair_price_request(pair);
        promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_AFTER_ORACLE)
                .get_price_for_trigger_bot_callback(tokens.len(), tokens, grid_bot),
        );
    }
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn get_price_for_create_bot_callback(&mut self, promise_num: usize, tokens: Vec<AccountId>, user: &AccountId,
                                         slippage: u16, entry_price: &U256C, pair: &Pair, grid_bot: &mut GridBot,
                                         storage_fee: u128, storage_used: StorageUsage) -> bool;
    fn get_price_for_close_bot_callback(&mut self, promise_num: usize, tokens: Vec<AccountId>, user: &AccountId, pair: &Pair, grid_bot: &mut GridBot);
    fn get_price_for_trigger_bot_callback(&mut self, promise_num: usize, tokens: Vec<AccountId>, grid_bot: &mut GridBot);
}

#[near_bindgen]
impl ExtSelf for GridBotContract {
    #[private]
    fn get_price_for_create_bot_callback(&mut self,
                                         promise_num: usize, tokens: Vec<AccountId>, user: &AccountId,
                                         slippage: u16, entry_price: &U256C, pair: &Pair, grid_bot: &mut GridBot,
                                         storage_fee: u128, storage_used: StorageUsage
    ) -> bool {
        let price_list = self.private_get_price_list(promise_num, tokens);
        if price_list.len() != PAIR_TOKEN_LENGTH {
            self.internal_create_bot_refund_with_near(user, pair, storage_fee, INVALID_PAIR_PRICE_LENGTH);
            return false;
        }
        return self.internal_create_bot(price_list[0].clone(), price_list[1].clone(), user, slippage, entry_price, pair, storage_fee, storage_used, grid_bot);
    }

    #[private]
    fn get_price_for_close_bot_callback(&mut self, promise_num: usize, tokens: Vec<AccountId>, user:&AccountId, pair: &Pair, grid_bot: &mut GridBot) {
        let price_list = self.private_get_price_list(promise_num, tokens);
        require!(price_list.len() == PAIR_TOKEN_LENGTH, INVALID_PAIR_PRICE_LENGTH);
        self.internal_auto_close_bot(price_list[0].clone(), price_list[1].clone(), user, &grid_bot.bot_id.clone(), grid_bot, pair);
    }

    #[private]
    fn get_price_for_trigger_bot_callback(&mut self, promise_num: usize, tokens: Vec<AccountId>, grid_bot: &mut GridBot) {
        let price_list = self.private_get_price_list(promise_num, tokens);
        require!(price_list.len() == PAIR_TOKEN_LENGTH, INVALID_PAIR_PRICE_LENGTH);
        self.internal_trigger_bot(price_list[0].clone(), price_list[1].clone(), &grid_bot.bot_id.clone(), grid_bot);
    }
}
