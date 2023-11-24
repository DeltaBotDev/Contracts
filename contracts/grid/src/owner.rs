use crate::*;

#[near_bindgen]
impl GridBotContract {
    /// Change owner. Only can be called by owner.
    pub fn set_owner(&mut self, owner_id: ValidAccountId) {
        self.assert_owner();
        self.owner_id = owner_id.as_ref().clone();
    }

    /// Get the owner of this account.
    pub fn get_owner(&self) -> AccountId {
        self.owner_id.clone()
    }

    pub fn modify_reward_per_sec(&mut self, reward_per_sec: U128, distribute_before_change: bool) {
        self.assert_owner();
        if distribute_before_change {
            self.distribute_reward();
        }
        self.reward_per_sec = reward_per_sec.into();
    }

    pub fn reset_reward_genesis_time_in_sec(&mut self, reward_genesis_time_in_sec: u32) {
        self.assert_owner();
        let cur_time = nano_to_sec(env::block_timestamp());
        if reward_genesis_time_in_sec < cur_time {
            env::panic(b"ERR_RESET_TIME_IS_PAST_TIME");
        } else if self.reward_genesis_time_in_sec < cur_time {
            env::panic(b"ERR_REWARD_GENESIS_TIME_PASSED");
        }
        self.reward_genesis_time_in_sec = reward_genesis_time_in_sec;
        self.prev_distribution_time_in_sec = reward_genesis_time_in_sec;
    }

    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "ERR_NOT_ALLOWED"
        );
    }

    /// Migration function.
    /// For next version upgrades, change this function.
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let prev: Contract = env::state_read().expect("ERR_NOT_INITIALIZED");
        prev
    }
}


#[cfg(target_arch = "wasm32")]
mod upgrade {
    use near_sdk::env::BLOCKCHAIN_INTERFACE;
    use near_sdk::Gas;

    use super::*;

    const BLOCKCHAIN_INTERFACE_NOT_SET_ERR: &str = "Blockchain interface not set.";

    /// Gas for calling migration call.
    pub const GAS_FOR_MIGRATE_CALL: Gas = 5_000_000_000_000;

    /// Self upgrade and call migrate, optimizes gas by not loading into memory the code.
    /// Takes as input non serialized set of bytes of the code.
    #[no_mangle]
    pub extern "C" fn upgrade() {
        env::setup_panic_hook();
        env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
        let contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        contract.assert_owner();
        let current_id = env::current_account_id().into_bytes();
        let method_name = "migrate".as_bytes().to_vec();
        unsafe {
            BLOCKCHAIN_INTERFACE.with(|b| {
                // Load input into register 0.
                b.borrow()
                    .as_ref()
                    .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                    .input(0);
                let promise_id = b
                    .borrow()
                    .as_ref()
                    .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                    .promise_batch_create(current_id.len() as _, current_id.as_ptr() as _);
                b.borrow()
                    .as_ref()
                    .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                    .promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
                let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_MIGRATE_CALL;
                b.borrow()
                    .as_ref()
                    .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                    .promise_batch_action_function_call(
                        promise_id,
                        method_name.len() as _,
                        method_name.as_ptr() as _,
                        0 as _,
                        0 as _,
                        0 as _,
                        attached_gas,
                    );
            });
        }
    }

}
