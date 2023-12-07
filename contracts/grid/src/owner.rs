use near_sdk::{assert_one_yocto, require};
use crate::*;

#[near_bindgen]
impl GridBotContract {
    /// Change owner. Only can be called by owner.
    pub fn set_owner(&mut self, owner_id: AccountId) {
        self.assert_owner();
        self.owner_id = owner_id.clone();
    }

    /// Get the owner of this account.
    pub fn get_owner(&self) -> AccountId {
        self.owner_id.clone()
    }

    pub(crate) fn assert_owner(&self) {
        assert_one_yocto();
        require!(env::predecessor_account_id() == self.owner_id, ERR_NOT_ALLOWED);
    }

    // /// Migration function.
    // /// For next version upgrades, change this function.
    // #[init(ignore_state)]
    // #[private]
    // pub fn migrate() -> Self {
    //     let prev: GridBotContract = env::state_read().expect("ERR_NOT_INITIALIZED");
    //     prev
    // }
}
