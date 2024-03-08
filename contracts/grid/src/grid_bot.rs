use crate::*;
use near_sdk::{assert_one_yocto, Gas, near_bindgen, Promise, require};
use near_sdk::json_types::U128;
use serde_json::json;
use crate::entity::{GridType};
use crate::events::emit;
use crate::GridStatus::Shutdown;

#[near_bindgen]
impl GridBotContract {

    #[payable]
    pub fn create_bot(&mut self, name: String, pair_id: String, slippage: u16, grid_type: GridType,
                      grid_rate: u16, grid_offset: U128, first_base_amount: U128, first_quote_amount: U128,
                      last_base_amount: U128, last_quote_amount: U128, fill_base_or_quote: bool, grid_sell_count: u16, grid_buy_count: u16,
                      trigger_price: U128, take_profit_price: U128, stop_loss_price: U128, valid_until_time: U128,
                      entry_price: U128, recommender: Option<AccountId>) {
        // record storage fee
        let initial_storage_usage = env::storage_usage();
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

        require!(valid_until_time.0 > env::block_timestamp_ms() as u128, INVALID_UNTIL_TIME);

        require!(self.pair_map.contains_key(&pair_id), INVALID_PAIR_ID);
        let pair = self.pair_map.get(&pair_id).unwrap().clone();
        let user = env::predecessor_account_id();

        // require!(self.status == GridStatus::Running, PAUSE_OR_SHUTDOWN);
        if self.status != GridStatus::Running {
            self.internal_create_bot_refund_with_near(&user, &pair, env::attached_deposit(), PAUSE_OR_SHUTDOWN);
            return;
        }

        if grid_buy_count + grid_sell_count > MAX_GRID_COUNT {
            self.internal_create_bot_refund_with_near(&user, &pair, env::attached_deposit(), MORE_THAN_MAX_GRID_COUNT);
            return;
        }

        // calculate all assets
        let (base_amount_sell, quote_amount_buy) = GridBotContract::internal_calculate_bot_assets(first_quote_amount_256.clone(), last_base_amount_256.clone(), grid_sell_count.clone(), grid_buy_count.clone(),
                                                       grid_type.clone(), grid_rate.clone(), grid_offset_256.clone(), fill_base_or_quote.clone());

        // require!(env::attached_deposit() >= STORAGE_FEE, LESS_STORAGE_FEE);
        if !self.internal_check_near_amount(&user, &pair, env::attached_deposit(), base_amount_sell, quote_amount_buy) {
            self.internal_create_bot_refund_with_near(&user, &pair, env::attached_deposit(), INVALID_AMOUNT);
            return;
        }
        // last_quote_amount / last_base_amount > first_quote_amount > first_base_amount
        // amount must u128, u128 * u128 <= u256, so, it's ok
        let (result, reason) = self.internal_check_bot_amount(grid_sell_count, grid_buy_count, first_base_amount_256, first_quote_amount_256,
                                                            last_base_amount_256, last_quote_amount_256, &pair, base_amount_sell, quote_amount_buy);
        if !result {
            self.internal_create_bot_refund_with_near(&user, &pair, env::attached_deposit(), &reason);
            return;
        }

        // create bot
        let mut new_grid_bot = GridBot {name, active: false, user: user.clone(), bot_id: "".to_string(), closed: false, pair_id, grid_type,
            grid_sell_count: grid_sell_count.clone(), grid_buy_count: grid_buy_count.clone(), grid_rate, grid_offset: grid_offset_256,
            first_base_amount: first_base_amount_256, first_quote_amount: first_quote_amount_256, last_base_amount: last_base_amount_256,
            last_quote_amount: last_quote_amount_256, fill_base_or_quote, trigger_price: trigger_price_256, trigger_price_above_or_below: false,
            take_profit_price: take_profit_price_256, stop_loss_price: stop_loss_price_256, valid_until_time: valid_until_time_256,
            total_quote_amount: quote_amount_buy, total_base_amount: base_amount_sell, revenue: U256C::from(0), total_revenue: U256C::from(0)
        };

        if self.internal_need_wrap_near(&user, &pair, base_amount_sell, quote_amount_buy) {
            // wrap near to wnear first
            let bot_near_amount = self.internal_get_bot_near_amount(&new_grid_bot, &pair);
            // check storage fee
            require!(env::attached_deposit() - bot_near_amount >= self.base_create_storage_fee + self.per_grid_storage_fee * (grid_buy_count + grid_sell_count) as u128, LESS_STORAGE_FEE);
            self.deposit_near_to_get_wnear_for_create_bot(&pair, &user, slippage, &entry_price_256, &mut new_grid_bot, bot_near_amount, recommender, env::attached_deposit() - bot_near_amount, initial_storage_usage);
        } else {
            // check storage fee
            require!(env::attached_deposit() >= self.base_create_storage_fee + self.per_grid_storage_fee * (grid_buy_count + grid_sell_count) as u128, LESS_STORAGE_FEE);
            // request token price
            self.get_price_for_create_bot(&pair, &user, slippage, &entry_price_256, &mut new_grid_bot, recommender, env::attached_deposit(), initial_storage_usage);
        }
    }

    #[payable]
    pub fn take_orders(&mut self, take_order: RequestOrder, maker_orders: Vec<OrderKeyInfo>) {
        assert_one_yocto();
        require!(self.market_user_map.contains_key(&(env::predecessor_account_id())), INVALID_USER);
        require!(take_order.amount_sell.0 >= self.deposit_limit_map.get(&take_order.token_sell).unwrap().as_u128(), INVALID_AMOUNT);
        self.internal_take_orders(&(env::predecessor_account_id()), &take_order.to_order(), maker_orders);
    }

    #[payable]
    pub fn close_bot(&mut self, bot_id: String) {
        assert_one_yocto();
        require!(self.bot_map.contains_key(&bot_id), BOT_NOT_EXIST);
        let mut bot = self.bot_map.get(&bot_id).unwrap().clone();
        require!(!bot.closed, INVALID_BOT_STATUS);
        let pair = self.pair_map.get(&bot.pair_id).unwrap().clone();
        // check permission, user self close
        require!(env::predecessor_account_id() == bot.user, INVALID_USER);

        self.internal_close_bot(&env::predecessor_account_id(), &bot_id, &mut bot, &pair);
    }

    pub fn auto_close_bot(&mut self, bot_id: String) {
        require!(self.bot_map.contains_key(&bot_id), BOT_NOT_EXIST);
        let mut bot = self.bot_map.get(&bot_id).unwrap().clone();
        require!(!bot.closed, INVALID_BOT_STATUS);
        let pair = self.pair_map.get(&bot.pair_id).unwrap().clone();

        self.get_price_for_close_bot(&env::predecessor_account_id(), &pair, &mut bot);
    }

    #[payable]
    pub fn claim(&mut self, bot_id: String) {
        assert_one_yocto();
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
        self.internal_withdraw_all(&user, &token);
    }

    #[payable]
    pub fn withdraw_refer_fee(&mut self, token: AccountId, amount: U128) {
        assert_one_yocto();
        let user = env::predecessor_account_id();
        self.internal_withdraw_refer_fee(&user, &token, amount);
    }

    #[payable]
    pub fn token_storage_deposit(&mut self, user: AccountId, token: AccountId) {
        require!(env::attached_deposit() == BASE_CREATE_STORAGE_FEE);
        self.internal_increase_asset(&user, &token, &U256C::from(0));
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

    #[payable]
    pub fn withdraw_near_after_shutdown(&mut self, to_user: AccountId, amount: U128) {
        require!(self.status == Shutdown, INVALID_STATUS);
        self.assert_owner();
        self.internal_ft_transfer_near_without_result(&to_user, amount.0);
    }

    #[payable]
    pub fn withdraw_unowned_asset(&mut self, token: AccountId, to_user: AccountId) {
        self.assert_owner();
        Promise::new(token.clone())
            .function_call(
                "ft_balance_of".to_string(),
                json!({"account_id": env::current_account_id()}).to_string().into_bytes(),
                0,
                Gas(0),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_AFTER_FT_TRANSFER)
                    .after_ft_balance_of_for_withdraw_unowned_asset(
                        token.clone(),
                        to_user,
                    )
            );
    }

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

    #[payable]
    pub fn set_operator(&mut self, new_operator: AccountId) {
        self.assert_owner();
        self.operator_id = new_operator;
    }

    #[payable]
    pub fn set_refer_fee_rate(&mut self, new_refer_fee_rate: Vec<u32>) {
        self.assert_owner();
        self.refer_fee_rate = new_refer_fee_rate;
    }

    #[payable]
    pub fn set_base_create_storage_fee(&mut self, new_base_create_storage_fee: U128) {
        self.assert_owner();
        self.base_create_storage_fee = new_base_create_storage_fee.0;
    }

    #[payable]
    pub fn set_storage_price_per_byte(&mut self, new_storage_price_per_byte: U128) {
        self.assert_owner();
        self.storage_price_per_byte = new_storage_price_per_byte.0;
    }

    #[payable]
    pub fn set_per_grid_storage_fee(&mut self, new_per_grid_storage_fee: U128) {
        self.assert_owner();
        self.per_grid_storage_fee = new_per_grid_storage_fee.0;
    }
}
