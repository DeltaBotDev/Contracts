use crate::*;
use near_sdk::{near_bindgen};
use crate::entity::GridType;

#[near_bindgen]
impl GridBotContract {
    #[payable]
    pub fn create_bot(&mut self, name:String, pair_id: String, slippage: u16, grid_type: GridType,
                      grid_rate: u16, grid_offset: U128C, first_base_amount: U128C, first_quote_amount: U128C,
                      last_base_amount: U128C, last_quote_amount: U128C, fill_base_or_quote: bool, grid_sell_count: u16, grid_buy_count: u16,
                      trigger_price: U256C, take_profit_price: U256C, stop_loss_price: U256C, valid_until_time: u64,
                      entry_price: U256C) {
        assert!(self.status == GridStatus::Running, "PAUSE_OR_SHUTDOWN");
        assert!(self.internal_check_oracle_price(entry_price, pair_id.clone(), slippage) , "ORACLE_PRICE_EXCEPTION");
        assert!(self.pair_map.contains_key(&pair_id), "INVALID_PAIR_ID");
        let pair = self.pair_map.get(&pair_id).unwrap().clone();
        let user = env::predecessor_account_id();

        // calculate all assets
        let (base_amount_sell, quote_amount_buy) = GridBotContract::internal_calculate_bot_assets(first_quote_amount.clone(), last_base_amount.clone(), grid_sell_count.clone(), grid_buy_count.clone(),
                                                       grid_type.clone(), grid_rate.clone(), grid_offset.clone(), fill_base_or_quote.clone());
        // check balance
        assert!(self.internal_get_balance(user.clone(), pair.base_token.clone()) >= base_amount_sell, "LESS_BASE_TOKEN");
        assert!(self.internal_get_balance(user.clone(), pair.quote_token.clone()) >= quote_amount_buy, "LESS_QUOTE_TOKEN");

        // transfer assets
        self.internal_transfer_assets_to_lock(user.clone(), pair.base_token.clone(), base_amount_sell);
        self.internal_transfer_assets_to_lock(user.clone(), pair.quote_token.clone(), quote_amount_buy);

        // create bot id
        let next_bot_id = format!("GRID:{}", self.internal_get_and_use_next_bot_id().to_string());
        // initial orders space
        let grid_count = grid_sell_count.clone() as usize + grid_buy_count.clone() as usize;
        self.order_map.insert(next_bot_id.clone(), vec!(Vec::with_capacity(grid_count.clone()), Vec::with_capacity(grid_count.clone())));

        // create bot
        let new_grid_bot = GridBot {user: user.clone(), bot_id: next_bot_id.clone(), closed: false, name, pair_id, grid_type,
            grid_sell_count: grid_sell_count.clone(), grid_buy_count: grid_buy_count.clone(), grid_rate, grid_offset,
            first_base_amount, first_quote_amount, last_base_amount, last_quote_amount, fill_base_or_quote,
            trigger_price, take_profit_price, stop_loss_price, valid_until_time,
        };
        // insert bot
        self.bot_map.insert(next_bot_id.clone(), new_grid_bot);
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
        assert_ne!(base_token, quote_token, "VALID_TOKEN");
        let pair_key = GridBotContract::internal_get_pair_key(base_token.clone(), quote_token.clone());
        assert!(!self.pair_map.contains_key(&pair_key), "PAIR_EXIST");
        let pair = Pair{
            // pair_id: U128C::from(self.internal_get_and_use_next_pair_id()),
            base_token: base_token.clone(),
            quote_token: quote_token.clone(),
        };
        self.pair_map.insert(pair_key, pair.clone());
        if !self.token_map.contains_key(&(base_token.clone())) {
            self.token_map.insert(base_token.clone(), U128C::from(0));
        }
        if !self.token_map.contains_key(&(quote_token.clone())) {
            self.token_map.insert(quote_token.clone(), U128C::from(0));
        }
    }

}
