use crate::*;
use crate::utils::u128_dec_format;

pub mod emit {
    use near_sdk::log;
    use near_sdk::serde::Serialize;
    use super::*;
    use near_sdk::serde_json::json;

    #[derive(Serialize)]
    #[serde(crate = "near_sdk::serde")]
    struct AccountAmountToken<'a> {
        pub account_id: &'a AccountId,
        #[serde(with = "u128_dec_format")]
        pub amount: Balance,
        pub token_id: &'a AccountId,
    }

    #[derive(Serialize)]
    #[serde(crate = "near_sdk::serde")]
    struct CreateBot<'a> {
        pub account_id: &'a AccountId,
        pub bot_id: String,
        pub base_price: String,
        pub quote_price: String,
        pub base_expo: String,
        pub quote_expo: String,
    }

    #[derive(Serialize)]
    #[serde(crate = "near_sdk::serde")]
    struct CloseBot<'a> {
        pub account_id: &'a AccountId,
        pub bot_id: String,
    }

    #[derive(Serialize)]
    #[serde(crate = "near_sdk::serde")]
    struct CloseBotPrice {
        pub base_price: String,
        pub quote_price: String,
        pub base_expo: String,
        pub quote_expo: String,
    }

    #[derive(Serialize)]
    #[serde(crate = "near_sdk::serde")]
    struct OrderUpdate<'a> {
        pub bot_id: String,
        pub forward_or_reverse: bool,
        pub level: usize,
        pub token_sell: &'a AccountId,
        pub token_buy: &'a AccountId,
        #[serde(with = "u128_dec_format")]
        pub amount_sell: Balance,
        #[serde(with = "u128_dec_format")]
        pub amount_buy: Balance,
        pub fill_buy_or_sell: bool,
        #[serde(with = "u128_dec_format")]
        pub filled: Balance,
    }

    #[derive(Serialize)]
    #[serde(crate = "near_sdk::serde")]
    struct TakeOrder<'a> {
        pub taker: &'a AccountId,
        pub maker: &'a AccountId,
        pub maker_bot_id: String,
        pub maker_forward_or_reverse: bool,
        #[serde(with = "u128_dec_format")]
        pub maker_level: Balance,
        #[serde(with = "u128_dec_format")]
        pub took_sell: Balance,
        #[serde(with = "u128_dec_format")]
        pub took_buy: Balance,
        #[serde(with = "u128_dec_format")]
        pub taker_fee: Balance,
        #[serde(with = "u128_dec_format")]
        pub maker_fee: Balance,
        #[serde(with = "u128_dec_format")]
        pub current_revenue: Balance,
        #[serde(with = "u128_dec_format")]
        pub maker_left_revenue: Balance,
        #[serde(with = "u128_dec_format")]
        pub maker_total_revenue: Balance,
    }

    #[derive(Serialize)]
    #[serde(crate = "near_sdk::serde")]
    struct Claim<'a> {
        pub claim_user: &'a AccountId,
        pub bot_id: String,
        pub user: &'a AccountId,
        pub revenue_token: &'a AccountId,
        #[serde(with = "u128_dec_format")]
        pub revenue: Balance,
    }

    #[derive(Serialize)]
    #[serde(crate = "near_sdk::serde")]
    struct TriggerBot {
        pub bot_id: String,
        pub base_price: String,
        pub quote_price: String,
        pub base_expo: String,
        pub quote_expo: String,
    }

    fn log_event<T: Serialize>(event: &str, data: T) {
        let event = json!({
            "standard": "DeltaBot",
            "version": "0.0.1",
            "event": event,
            "data": [data]
        });

        log!("EVENT_JSON:{}", event.to_string());
    }

    pub fn withdraw_started(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "withdraw_started",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }

    pub fn withdraw_succeeded(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "withdraw_succeeded",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }

    pub fn withdraw_failed(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "withdraw_failed",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }
    pub fn withdraw_protocol_fee_started(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "withdraw_protocol_fee_started",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }

    pub fn withdraw_protocol_fee_succeeded(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "withdraw_protocol_fee_succeeded",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }

    pub fn withdraw_protocol_fee_failed(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "withdraw_protocol_fee_failed",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }
    // pub fn withdraw_unowned_asset_started(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
    //     log_event(
    //         "withdraw_unowned_asset_started",
    //         AccountAmountToken {
    //             account_id: &account_id,
    //             amount,
    //             token_id: &token_id,
    //         },
    //     );
    // }
    //
    // pub fn withdraw_unowned_asset_succeeded(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
    //     log_event(
    //         "withdraw_unowned_asset_succeeded",
    //         AccountAmountToken {
    //             account_id: &account_id,
    //             amount,
    //             token_id: &token_id,
    //         },
    //     );
    // }
    //
    // pub fn withdraw_unowned_asset_failed(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
    //     log_event(
    //         "withdraw_unowned_asset_failed",
    //         AccountAmountToken {
    //             account_id: &account_id,
    //             amount,
    //             token_id: &token_id,
    //         },
    //     );
    // }

    pub fn storage_deposit_succeeded(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "storage_deposit_succeeded",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }

    pub fn storage_deposit_failed(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "storage_deposit_failed",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }

    pub fn deposit_success(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "deposit",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }

    pub fn deposit_return_success(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "deposit_return",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }

    pub fn take_order(taker: &AccountId, maker: &AccountId, maker_bot_id: String, maker_forward_or_reverse: bool, maker_level: usize, took_sell: &U256C, took_buy: &U256C, maker_fee: &U256C, taker_fee: &U256C, current_revenue: &U256C, maker_left_revenue: &U256C, maker_total_revenue: &U256C) {
        log_event(
            "take_order",
            TakeOrder {
                taker,
                maker,
                maker_bot_id,
                maker_forward_or_reverse,
                maker_level: maker_level as u128,
                took_sell: took_sell.as_u128(),
                took_buy: took_buy.as_u128(),
                maker_fee: maker_fee.as_u128(),
                taker_fee: taker_fee.as_u128(),
                current_revenue: current_revenue.as_u128(),
                maker_left_revenue: maker_left_revenue.as_u128(),
                maker_total_revenue: maker_total_revenue.as_u128(),
            },
        );
    }

    pub fn create_bot(account_id: &AccountId, bot_id: String, base_price: String, quote_price: String, base_expo: String, quote_expo: String) {
        log_event(
            "create_bot",
            CreateBot {
                account_id: &account_id,
                bot_id,
                base_price,
                quote_price,
                base_expo,
                quote_expo,
            },
        );
    }

    pub fn close_bot(account_id: &AccountId, bot_id: String) {
        log_event(
            "close_bot",
            CloseBot {
                account_id: &account_id,
                bot_id,
            },
        );
    }

    pub fn close_bot_price(base_price: String, quote_price: String, base_expo: String, quote_expo: String) {
        log_event(
            "close_bot_price",
            CloseBotPrice {
                base_price,
                quote_price,
                base_expo,
                quote_expo,
            },
        );
    }

    pub fn claim(claim_user: &AccountId, user: &AccountId, bot_id: String, revenue_token: &AccountId, revenue: U256C) {
        log_event(
            "claim",
            Claim {
                claim_user,
                bot_id,
                user,
                revenue_token,
                revenue: revenue.as_u128(),
            },
        );
    }

    pub fn trigger_bot(bot_id: String, base_price: String, quote_price: String, base_expo: String, quote_expo: String) {
        log_event(
            "trigger_bot",
            TriggerBot {
                bot_id,
                base_price,
                quote_price,
                base_expo,
                quote_expo,
            },
        );
    }

    pub fn order_update(bot_id: String, forward_or_reverse: bool, level: usize, order: &Order) {
        log_event(
            "order_update",
            OrderUpdate {
                bot_id,
                forward_or_reverse,
                level,
                token_sell: &order.token_sell,
                token_buy: &order.token_buy,
                amount_sell: order.amount_sell.as_u128(),
                amount_buy: order.amount_buy.as_u128(),
                fill_buy_or_sell: order.fill_buy_or_sell.clone(),
                filled: order.filled.as_u128(),
            },
        );
    }

}
