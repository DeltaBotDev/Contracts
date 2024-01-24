use near_sdk::{AccountId, env, is_promise_success};
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
    pub fn withdraw_near(&mut self, user: &AccountId, amount: u128) {
        ext_wnear::ext(self.wnear.clone())
            .with_attached_deposit(1)
            .near_withdraw(U128::from(amount))
            .then(
                Self::ext(env::current_account_id())
                    .after_withdraw_near(
                        user,
                        amount,
                    )
            );
    }

    pub fn deposit_near_to_get_wnear(&mut self, pair: &Pair, user: &AccountId, slippage: u16, entry_price: &U256C,
                                     grid_bot: &mut GridBot, amount: u128) {
        ext_wnear::ext(self.wnear.clone())
            .with_attached_deposit(amount)
            .near_deposit()
            .then(
            Self::ext(env::current_account_id())
                .after_wrap_near(
                    pair,
                    user,
                    slippage,
                    entry_price,
                    grid_bot,
                    amount,
                )
        );
    }
}


#[ext_contract(ext_self)]
trait ExtSelf {
    fn after_wrap_near(&mut self, pair: &Pair, user: &AccountId, slippage: u16, entry_price: &U256C,
                       grid_bot: &mut GridBot, amount: u128) -> bool;
    fn after_withdraw_near(&mut self, user: &AccountId, amount: u128) -> bool;
}

#[near_bindgen]
impl ExtSelf for GridBotContract {
    #[private]
    fn after_wrap_near(&mut self, pair: &Pair, user: &AccountId, slippage: u16, entry_price: &U256C, grid_bot: &mut GridBot, amount: u128) -> bool {
        let promise_success = is_promise_success();
        if !promise_success.clone() {
            emit::wrap_near_error(user, 0, amount, true);
        } else {
            // deposit
            self.internal_deposit(&user.clone(), &self.wnear.clone(), U128::from(amount));
            // request price
            self.get_price_for_create_bot(pair, user, slippage, entry_price, grid_bot);
        }
        promise_success
    }

    #[private]
    fn after_withdraw_near(&mut self, user: &AccountId, amount: u128) -> bool {
        let promise_success = is_promise_success();
        if !promise_success.clone() {
            emit::wrap_near_error(user, 0, amount, false);
        } else {
            self.internal_ft_transfer_near(user, amount);
        }
        promise_success
    }
}
