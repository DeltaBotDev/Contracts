use crate::*;
use near_sdk::{near_bindgen};
use crate::entity::GridType;

#[near_bindgen]
impl GridBotContract {
    #[payable]
    pub fn create_bot(&mut self, name:String, pair_id: U128C, slippage: u16, grid_type: GridType,
                      grid_rate: u16, grid_offset: U256C, first_base_amount: U256C, first_quote_amount: U256C,
                      last_base_amount: U256C, last_quote_amount: U256C, fill_base_or_quote: u8, grid_count: u16,
                      trigger_price: U256C, take_profit_price: U256C, stop_loss_price: U256C, valid_until_time: u64,
                      entry_price: U256C) {
        assert!(self.status == GridStatus::Running, "PAUSE_OR_SHUTDOWN");
        assert!(self.internal_check_oracle_price(entry_price, pair_id, slippage) , "ORACLE_PRICE_EXCEPTION");

        // TODO Get Asset

        let next_bot_id = format!("GRID:{}", self.internal_get_and_use_next_bot_id().to_string());

        let new_grid_bot = GridBot {user: env::predecessor_account_id(), bot_id: next_bot_id.clone(), closed: false, name, pair_id, grid_type,
            grid_count: grid_count.clone(), grid_rate, grid_offset, first_base_amount, first_quote_amount, last_base_amount,
            last_quote_amount, fill_base_or_quote, trigger_price, take_profit_price, stop_loss_price, valid_until_time,
        };
        self.bot_map.insert(next_bot_id.clone(), new_grid_bot);
        // initial orders space
        self.order_map.insert(next_bot_id.clone(), vec!(Vec::with_capacity(grid_count.clone() as usize), Vec::with_capacity(grid_count.clone() as usize)));
    }

    pub fn close_bot(&mut self, bot_id: String) {
        assert!(self.bot_map.contains_key(&bot_id), "BOT_NOT_EXIST");
        let bot = self.bot_map.get_mut(&bot_id).unwrap();
        // check permission
        assert_eq!(bot.user, env::predecessor_account_id(), "NO_PERMISSION");
        bot.closed = true;
        // TODO Return Asset
    }

    pub fn withdraw(&mut self, token: AccountId) {

    }

    pub fn take_orders(&mut self, take_order: Order, maker_orders: Vec<Order>, slippage: u16) {
        assert!(self.status == GridStatus::Running, "PAUSE_OR_SHUTDOWN");

    }

    pub fn claim(&mut self, bot_id: String) {

    }

    pub fn trigger_bot(&mut self, bot_id: String) {
        assert!(self.status == GridStatus::Running, "PAUSE_OR_SHUTDOWN");

    }

    pub fn take_profit(&mut self, bot_id: String) {
        assert!(self.status == GridStatus::Running, "PAUSE_OR_SHUTDOWN");

    }

    pub fn stop_loss(&mut self, bot_id: String) {
        assert!(self.status == GridStatus::Running, "PAUSE_OR_SHUTDOWN");

    }

    pub fn withdraw_unowned_asset(&mut self, token: AccountId) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "NO_PERMISSION");

    }

    pub fn pause(&mut self) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "NO_PERMISSION");
        self.status = GridStatus::Paused;
    }

    pub fn start(&mut self) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "NO_PERMISSION");
        self.status = GridStatus::Running;
    }

    pub fn shutdown(&mut self) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "NO_PERMISSION");
        self.status = GridStatus::Shutdown;
    }

    pub fn register_pair(&mut self, base_token: AccountId, quote_token: AccountId) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "NO_PERMISSION");
        let pair_key = format!("{}:{}", base_token.clone().to_string(), quote_token.clone().to_string());
        assert!(!self.pair_map.contains_key(&pair_key), "PAIR_EXIST");
        let pair = Pair{
            pair_id: U128C::from(self.internal_get_and_use_next_pair_id()),
            base_token: base_token.clone(),
            quote_token: quote_token.clone(),
        };
        self.pair_map.insert(pair_key, pair);
        if !self.token_map.contains_key(&(base_token.clone())) {
            self.token_map.insert(base_token.clone(), U128C::from(0));
        }
        if !self.token_map.contains_key(&(quote_token.clone())) {
            self.token_map.insert(quote_token.clone(), U128C::from(0));
        }
    }

}
