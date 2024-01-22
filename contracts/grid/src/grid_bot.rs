use crate::*;
use near_sdk::{assert_one_yocto, near_bindgen, require};
use near_sdk::json_types::U128;
use crate::entity::{GridType};
use crate::events::emit;

#[near_bindgen]
impl GridBotContract {

    #[payable]
    pub fn create_bot(&mut self, name: String, pair_id: String, slippage: u16, grid_type: GridType,
                      grid_rate: u16, grid_offset: U128, first_base_amount: U128, first_quote_amount: U128,
                      last_base_amount: U128, last_quote_amount: U128, fill_base_or_quote: bool, grid_sell_count: u16, grid_buy_count: u16,
                      trigger_price: U128, take_profit_price: U128, stop_loss_price: U128, valid_until_time: U128,
                      entry_price: U128) {
        let grid_offset_256 = U256C::from(grid_offset.0);
        let first_base_amount_256 = U256C::from(first_base_amount.0);
        let first_quote_amount_256 = U256C::from(first_quote_amount.0);
        let last_base_amount_256 = U256C::from(last_base_amount.0);
        let last_quote_amount_256 = U256C::from(last_quote_amount.0);
        let trigger_price_256 = U256C::from(trigger_price.0);
        let take_profit_price_256 = U256C::from(take_profit_price.0);
        let stop_loss_price_256 = U256C::from(stop_loss_price.0);
        let valid_until_time_256 = U256C::from(valid_until_time.0);
        let entry_price_256 = U256C::from(entry_price.0);
        require!(env::attached_deposit() == STORAGE_FEE, LESS_STORAGE_FEE);
        require!(self.status == GridStatus::Running, PAUSE_OR_SHUTDOWN);

        require!(self.pair_map.contains_key(&pair_id), INVALID_PAIR_ID);
        let pair = self.pair_map.get(&pair_id).unwrap().clone();
        let user = env::predecessor_account_id();

        // calculate all assets
        let (base_amount_sell, quote_amount_buy) = GridBotContract::internal_calculate_bot_assets(first_quote_amount_256.clone(), last_base_amount_256.clone(), grid_sell_count.clone(), grid_buy_count.clone(),
                                                       grid_type.clone(), grid_rate.clone(), grid_offset_256.clone(), fill_base_or_quote.clone());

        // last_quote_amount / last_base_amount > first_quote_amount > first_base_amount
        // amount must u128, u128 * u128 <= u256, so, it's ok
        self.internal_check_bot_amount(grid_sell_count, grid_buy_count, first_base_amount_256, first_quote_amount_256,
                                       last_base_amount_256, last_quote_amount_256, &pair, base_amount_sell, quote_amount_buy);

        // check balance
        require!(self.internal_get_user_balance(&user, &(pair.base_token)) >= base_amount_sell, LESS_BASE_TOKEN);
        require!(self.internal_get_user_balance(&user, &(pair.quote_token)) >= quote_amount_buy, LESS_QUOTE_TOKEN);

        // create bot id
        let next_bot_id = format!("GRID:{}", self.internal_get_and_use_next_bot_id().to_string());

        // create bot
        let mut new_grid_bot = GridBot {name, active: false, user: user.clone(), bot_id: next_bot_id.clone(), closed: false, pair_id, grid_type,
            grid_sell_count: grid_sell_count.clone(), grid_buy_count: grid_buy_count.clone(), grid_rate, grid_offset: grid_offset_256,
            first_base_amount: first_base_amount_256, first_quote_amount: first_quote_amount_256, last_base_amount: last_base_amount_256,
            last_quote_amount: last_quote_amount_256, fill_base_or_quote, trigger_price: trigger_price_256, trigger_price_above_or_below: false,
            take_profit_price: take_profit_price_256, stop_loss_price: stop_loss_price_256, valid_until_time: valid_until_time_256,
            total_quote_amount: quote_amount_buy, total_base_amount: base_amount_sell, revenue: U256C::from(0), total_revenue: U256C::from(0)
        };

        // // record storage fee
        // self.storage_fee += env::attached_deposit();

        // request token price
        self.get_price_for_create_bot(&pair, &user, slippage, &entry_price_256, &mut new_grid_bot);
    }

    #[payable]
    pub fn take_orders(&mut self, take_order: RequestOrder, maker_orders: Vec<OrderKeyInfo>) {
        assert_one_yocto();
        require!(self.market_user_map.contains_key(&(env::predecessor_account_id())), INVALID_USER);
        self.internal_take_orders(&(env::predecessor_account_id()), &take_order.to_order(), maker_orders);
    }

    #[payable]
    pub fn close_bot(&mut self, bot_id: String) {
        assert_one_yocto();
        require!(self.bot_map.contains_key(&bot_id), BOT_NOT_EXIST);
        let mut bot = self.bot_map.get(&bot_id).unwrap().clone();
        let pair = self.pair_map.get(&bot.pair_id).unwrap().clone();
        // check permission, user self close or take profit or stop loss
        // let user = env::predecessor_account_id();
        require!(env::predecessor_account_id() == bot.user, INVALID_USER);
        // require!(self.internal_check_bot_close_permission(&user, &bot), NO_PERMISSION);

        self.internal_close_bot(&env::predecessor_account_id(), &bot_id, &mut bot, &pair);
    }

    pub fn auto_close_bot(&mut self, bot_id: String) {
        require!(self.bot_map.contains_key(&bot_id), BOT_NOT_EXIST);
        let mut bot = self.bot_map.get(&bot_id).unwrap().clone();
        let pair = self.pair_map.get(&bot.pair_id).unwrap().clone();

        self.get_price_for_close_bot(&env::predecessor_account_id(), &pair, &mut bot);
    }

    pub fn claim(&mut self, bot_id: String) {
        require!(self.bot_map.contains_key(&bot_id), BOT_NOT_EXIST);
        let mut bot = self.bot_map.get(&bot_id).unwrap().clone();
        let pair = self.pair_map.get(&(bot.pair_id)).unwrap().clone();
        // harvest revenue
        let (revenue_token, revenue) = self.internal_harvest_revenue(&mut bot, &pair);
        self.internal_withdraw(&(bot.user), &revenue_token, revenue);
        self.bot_map.insert(&bot_id, &bot);
        // event
        emit::claim(&env::predecessor_account_id(), &(bot.user), bot_id, &revenue_token, revenue);
    }

    pub fn trigger_bot(&mut self, bot_id: String) {
        require!(self.status == GridStatus::Running, PAUSE_OR_SHUTDOWN);
        let mut bot = self.bot_map.get(&bot_id).unwrap().clone();
        require!(bot.active.clone() == false, BOT_IS_ACTIVE);
        let pair = self.pair_map.get(&bot.pair_id).unwrap().clone();
        self.get_price_for_trigger_bot(&pair, &mut bot);
    }

    #[payable]
    pub fn withdraw(&mut self, token: AccountId) {
        assert_one_yocto();
        let user = env::predecessor_account_id();
        let balance = self.internal_get_user_balance(&user, &token);
        self.internal_withdraw(&user, &token, balance);
    }

    //################################################## Owner #####################################

    #[payable]
    pub fn withdraw_protocol_fee(&mut self, token: AccountId, to_user: AccountId, amount: U128) {
        self.assert_owner();
        require!(self.protocol_fee_map.contains_key(&token), INVALID_TOKEN);
        let protocol_fee = self.internal_get_protocol_fee(&token);
        require!(protocol_fee.as_u128() >= amount.0, INVALID_AMOUNT);
        self.internal_withdraw_protocol_fee(&to_user, &token, U256C::from(amount.0));
    }

    // #[payable]
    // pub fn withdraw_unowned_asset(&mut self, token: AccountId, to_user: AccountId) {
    //     self.assert_owner();
    //     require!(self.status != GridStatus::Shutdown, PAUSE_OR_SHUTDOWN);
    //     Promise::new(token.clone())
    //         .function_call(
    //             "ft_balance_of".to_string(),
    //             json!({"account_id": env::current_account_id()}).to_string().into_bytes(),
    //             0,
    //             Gas(0),
    //         )
    //         .then(
    //             Self::ext(self.owner_id.clone())
    //                 .with_static_gas(GAS_FOR_AFTER_FT_TRANSFER)
    //                 .after_ft_balance_of_for_withdraw_unowned_asset(
    //                     token.clone(),
    //                     to_user,
    //                 )
    //         );
    // }

    #[payable]
    pub fn set_protocol_fee_rate(&mut self, new_protocol_fee_rate: U128, new_taker_fee_rate: U128) {
        self.assert_owner();
        require!(new_protocol_fee_rate.0 <= MAX_PROTOCOL_FEE, INVALID_PROTOCOL_FEE);
        require!(new_taker_fee_rate.0 <= MAX_PROTOCOL_FEE, INVALID_PROTOCOL_FEE);
        self.protocol_fee_rate = new_protocol_fee_rate.0;
        self.taker_fee_rate = new_taker_fee_rate.0;
    }

    #[payable]
    pub fn pause(&mut self) {
        self.assert_owner();
        require!(self.status != GridStatus::Shutdown, HAD_SHUTDOWN);
        self.status = GridStatus::Paused;
    }

    #[payable]
    pub fn start(&mut self) {
        self.assert_owner();
        require!(self.status != GridStatus::Shutdown, HAD_SHUTDOWN);
        self.status = GridStatus::Running;
    }

    #[payable]
    pub fn shutdown(&mut self) {
        self.assert_owner();
        self.status = GridStatus::Shutdown;
    }

    #[payable]
    pub fn register_pair(&mut self, base_token: AccountId, quote_token: AccountId, base_min_deposit: U128, quote_min_deposit: U128, base_oracle_id: String, quote_oracle_id: String) {
        require!(env::attached_deposit() == DEFAULT_TOKEN_STORAGE_FEE * 2, LESS_TOKEN_STORAGE_FEE);
        require!(env::predecessor_account_id() == self.owner_id, ERR_NOT_ALLOWED);
        require!(base_token != quote_token, INVALID_TOKEN);
        let pair_key = GridBotContract::internal_get_pair_key(base_token.clone(), quote_token.clone());
        require!(!self.pair_map.contains_key(&pair_key), PAIR_EXIST);
        let pair = Pair{
            base_token: base_token.clone(),
            quote_token: quote_token.clone(),
            base_oracle_id: self.internal_format_price_identifier(base_oracle_id),
            quote_oracle_id: self.internal_format_price_identifier(quote_oracle_id),
        };
        self.pair_map.insert(&pair_key, &pair);
        self.internal_init_token(base_token, base_min_deposit);
        self.internal_init_token(quote_token, quote_min_deposit);
    }

    #[payable]
    pub fn set_min_deposit(&mut self, token: AccountId, min_deposit: U128) {
        self.assert_owner();
        self.deposit_limit_map.insert(&token, &U256C::from(min_deposit.0));
    }

    #[payable]
    pub fn storage_deposit(&mut self, token: AccountId, storage_fee: U128) {
        require!(env::predecessor_account_id() == self.owner_id, ERR_NOT_ALLOWED);
        require!(env::attached_deposit() == storage_fee.0, LESS_TOKEN_STORAGE_FEE);
        self.internal_storage_deposit(&env::current_account_id(), &token, storage_fee.0);
    }

    #[payable]
    pub fn set_oracle(&mut self, new_oracle: AccountId) {
        self.assert_owner();
        self.oracle = new_oracle;
    }

    #[payable]
    pub fn set_oracle_valid_time(&mut self, new_valid_time: u64) {
        self.assert_owner();
        self.oracle_valid_time = new_valid_time;
    }

    #[payable]
    pub fn set_market_user(&mut self, market_user: AccountId, enable: bool) {
        self.assert_owner();
        self.market_user_map.insert(&market_user, &enable);
    }
}
