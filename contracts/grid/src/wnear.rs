use near_sdk::{AccountId, env, is_promise_success, StorageUsage};
use near_sdk::json_types::U128;
use crate::{GridBot, Pair, U256C};
use crate::events::emit;
use crate::*;
use near_sdk::{ext_contract, near_bindgen};

#[ext_contract(ext_wnear)]
pub trait WNEARNear {
    fn near_deposit(&mut self);
    fn near_withdraw(&mut self, amount: U128);
}

impl GridBotContract {
    // note: withdraw_near just can be used by user withdraw, because, if error, will add user's balance
    pub fn withdraw_near(&mut self, user: &AccountId, amount: u128) {
        ext_wnear::ext(self.wnear.clone())
            .with_attached_deposit(ONE_YOCTO)
            .near_withdraw(U128::from(amount))
            .then(
                Self::ext(env::current_account_id())
                    .after_withdraw_near(
                        user,
                        amount,
                    )
            );
    }

    pub fn deposit_near_to_get_wnear_for_create_bot(&mut self, pair: &Pair, user: &AccountId, slippage: u16, entry_price: &U256C,
                                     grid_bot: &mut GridBot, amount: u128, recommender: Option<AccountId>, storage_fee: u128) {
        ext_wnear::ext(self.wnear.clone())
            .with_attached_deposit(amount)
            // .with_static_gas(GAS_FOR_CREATE_BOT_AFTER_NEAR)
            .near_deposit()
            .then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_CREATE_BOT_AFTER_NEAR)
                .after_wrap_near_for_create_bot(
                    pair,
                    user,
                    slippage,
                    entry_price,
                    grid_bot,
                    amount,
                    recommender,
                    storage_fee
                )
        );
    }
}


#[ext_contract(ext_self)]
trait ExtSelf {
    fn after_wrap_near_for_create_bot(&mut self, pair: &Pair, user: &AccountId, slippage: u16, entry_price: &U256C,
                       grid_bot: &mut GridBot, amount: u128, recommender: Option<AccountId>, storage_fee: u128) -> bool;
    fn after_withdraw_near(&mut self, user: &AccountId, amount: u128) -> bool;
}

#[near_bindgen]
impl ExtSelf for GridBotContract {
    #[private]
    // just used for create bot
    fn after_wrap_near_for_create_bot(&mut self, pair: &Pair, user: &AccountId, slippage: u16, entry_price: &U256C, grid_bot: &mut GridBot, amount: u128, recommender: Option<AccountId>, storage_fee: u128) -> bool {
        let promise_success = is_promise_success();
        if !promise_success.clone() {
            // refund token and near
            self.internal_create_bot_refund_with_near(user, pair, amount + storage_fee, WRAP_TO_WNEAR_ERROR);
            emit::wrap_near_error(user, 0, amount, true);
        } else {
            // deposit
            if !self.internal_deposit(&user.clone(), &self.wnear.clone(), U128::from(amount)) {
                // maybe just need hande one token, but it's ok, no problem
                self.internal_increase_asset(user, &self.wnear.clone(), &U256C::from(amount.clone()));
                self.internal_create_bot_refund_with_near(user, pair, storage_fee, WRAP_TO_WNEAR_ERROR);
                emit::wrap_near_error(user, 0, amount, true);
            } else {
                // request price
                if pair.require_oracle {
                    self.get_price_for_create_bot(pair, user, slippage, entry_price, grid_bot, recommender, storage_fee);
                } else {
                    self.internal_create_bot(None, None, user, slippage, entry_price, pair, recommender, storage_fee, grid_bot);
                }
            }
        }
        promise_success
    }

    #[private]
    fn after_withdraw_near(&mut self, user: &AccountId, amount: u128) -> bool {
        let promise_success = is_promise_success();
        if !promise_success.clone() {
            emit::wrap_near_error(user, 0, amount, false);
            self.internal_increase_asset(user, &self.wnear.clone(), &(U256C::from(amount)));
        } else {
            self.internal_ft_transfer_near(user, amount, true);
        }
        promise_success
    }
}
