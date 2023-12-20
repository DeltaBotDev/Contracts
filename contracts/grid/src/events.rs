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

    fn log_event<T: Serialize>(event: &str, data: T) {
        let event = json!({
            "standard": "Unimate",
            "version": "1.0.0",
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
    pub fn withdraw_unowned_asset_started(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "withdraw_unowned_asset_started",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }

    pub fn withdraw_unowned_asset_succeeded(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "withdraw_unowned_asset_succeeded",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }

    pub fn withdraw_unowned_asset_failed(account_id: &AccountId, amount: Balance, token_id: &AccountId) {
        log_event(
            "withdraw_unowned_asset_failed",
            AccountAmountToken {
                account_id: &account_id,
                amount,
                token_id: &token_id,
            },
        );
    }

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
}
