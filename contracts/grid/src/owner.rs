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

    /// Should only be called by this contract on migration.
    /// This is NOOP implementation. KEEP IT if you haven't changed contract state.
    /// If you have, you need to implement migration from old state
    /// (keep the old struct with different name to deserialize it first).
    /// After migration goes live, revert back to this implementation for next updates.
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let contract: GridBotContract = env::state_read().expect(CAN_NOT_READ_STATE);
        contract
        // Self {
        //     owner_id: contract.owner_id.clone(),
        //     oracle: contract.oracle,
        //     oracle_valid_time: DEFAULT_ORACLE_VALID_TIME,
        //     status: GridStatus::Running,
        //     // 1%
        //     protocol_fee_rate: DEFAULT_PROTOCOL_FEE,
        //     taker_fee_rate: DEFAULT_TAKER_FEE,
        //     bot_map: contract.bot_map,
        //     // order_map: LookupMap::new(b"orders".to_vec()),
        //     order_map: contract.order_map,
        //     next_bot_id: contract.next_bot_id,
        //     // oracle_price_map: LookupMap::new(b"oracle".to_vec()),
        //     pair_map: contract.pair_map,
        //     protocol_fee_map: contract.protocol_fee_map,
        //     // storage_fee: 0,
        //     global_balances_map: contract.global_balances_map,
        //     deposit_limit_map: contract.deposit_limit_map,
        //     user_balances_map: contract.user_balances_map,
        //     user_locked_balances_map: contract.user_locked_balances_map,
        //     // user_withdraw_failed_map: LookupMap::new(StorageKey::WithdrawFailedMainKey),
        //     market_user_map: contract.market_user_map,
        //     wnear: contract.wnear,
        //     operator_id: contract.owner_id,
        //     refer_recommender_user_map: LookupMap::new(b"rec_users".to_vec()),
        //     refer_user_recommender_map: LookupMap::new(b"user_rec".to_vec()),
        //     refer_fee_map: LookupMap::new(StorageKey::ReferFeeMainKey),
        //     refer_fee_rate: vec![],
        // }
    }
}

mod upgrade {
    use near_sdk::{require, Gas};
    use near_sys as sys;

    use super::*;

    const GAS_TO_COMPLETE_UPGRADE_CALL: Gas = Gas(Gas::ONE_TERA.0 * 10);
    const GAS_FOR_GET_CONFIG_CALL: Gas = Gas(Gas::ONE_TERA.0 * 5);
    const MIN_GAS_FOR_MIGRATE_STATE_CALL: Gas = Gas(Gas::ONE_TERA.0 * 60);

    /// Self upgrade and call migrate, optimizes gas by not loading into memory the code.
    /// Takes as input non serialized set of bytes of the code.
    #[no_mangle]
    pub extern "C" fn upgrade() {
        env::setup_panic_hook();
        let contract: GridBotContract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        contract.assert_owner();
        let current_account_id = env::current_account_id().as_bytes().to_vec();
        let migrate_method_name = b"migrate".to_vec();
        let query_protocol_fee_rate_method_name = b"query_protocol_fee_rate".to_vec();
        let empty_args = b"{}".to_vec();
        unsafe {
            // Load input (wasm code) into register 0.
            sys::input(0);
            // Create batch action promise for the current contract ID
            let promise_id = sys::promise_batch_create(
                current_account_id.len() as _,
                current_account_id.as_ptr() as _,
            );
            // 1st action in the Tx: "deploy contract" (code is taken from register 0)
            sys::promise_batch_action_deploy_contract(promise_id.clone(), u64::MAX as _, 0);
            // Gas required to complete this call.
            let required_gas =
                env::used_gas() + GAS_TO_COMPLETE_UPGRADE_CALL + GAS_FOR_GET_CONFIG_CALL;
            require!(
                env::prepaid_gas() >= required_gas + MIN_GAS_FOR_MIGRATE_STATE_CALL,
                "Not enough gas to complete state migration"
            );
            let migrate_state_attached_gas = env::prepaid_gas() - required_gas;
            // 2nd action in the Tx: call this_contract.migrate() with remaining gas
            sys::promise_batch_action_function_call(
                promise_id.clone(),
                migrate_method_name.len() as _,
                migrate_method_name.as_ptr() as _,
                empty_args.len() as _,
                empty_args.as_ptr() as _,
                0 as _,
                migrate_state_attached_gas.0,
            );
            // Scheduling to return config after the migration is completed.
            //
            // The upgrade method attaches it as an action, so the entire upgrade including deploy
            // contract action and migration can be rolled back if the config view call can't be
            // returned successfully. The view call deserializes the state and deserializes the
            // config which contains the owner_id. If the contract can deserialize the current config,
            // then it can validate the owner and execute the upgrade again (in case the previous
            // upgrade/migration went badly).
            //
            // It's an extra safety guard for the remote contract upgrades.
            sys::promise_batch_action_function_call(
                promise_id.clone(),
                query_protocol_fee_rate_method_name.len() as _,
                query_protocol_fee_rate_method_name.as_ptr() as _,
                empty_args.len() as _,
                empty_args.as_ptr() as _,
                0 as _,
                GAS_FOR_GET_CONFIG_CALL.0,
            );
            sys::promise_return(promise_id);
        }
    }
}
