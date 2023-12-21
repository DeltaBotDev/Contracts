use crate::*;
use near_sdk::{assert_one_yocto, Gas, near_bindgen, Promise, require};
use serde_json::json;
use crate::entity::{GridType};

#[near_bindgen]
impl GridBotContract {

    #[payable]
    pub fn create_bot(&mut self, pair_id: String, slippage: u16, grid_type: GridType,
                      grid_rate: u16, grid_offset: U256C, first_base_amount: U256C, first_quote_amount: U256C,
                      last_base_amount: U256C, last_quote_amount: U256C, fill_base_or_quote: bool, grid_sell_count: u16, grid_buy_count: u16,
                      trigger_price: U256C, take_profit_price: U256C, stop_loss_price: U256C, valid_until_time: U256C,
                      entry_price: U256C) {
        require!(env::attached_deposit() == STORAGE_FEE, LESS_STORAGE_FEE);
        require!(self.status == GridStatus::Running, PAUSE_OR_SHUTDOWN);
        // last_quote_amount / last_base_amount > first_quote_amount > first_base_amount
        // amount must u128, u128 * u128 <= u256, so, it's ok
        require!(last_quote_amount * first_base_amount > first_quote_amount * last_base_amount , INVALID_FIRST_OR_LAST_AMOUNT);
        require!(self.pair_map.contains_key(&pair_id), INVALID_PAIR_ID);
        let pair = self.pair_map.get(&pair_id).unwrap().clone();
        let user = env::predecessor_account_id();

        // calculate all assets
        let (base_amount_sell, quote_amount_buy) = GridBotContract::internal_calculate_bot_assets(first_quote_amount.clone(), last_base_amount.clone(), grid_sell_count.clone(), grid_buy_count.clone(),
                                                       grid_type.clone(), grid_rate.clone(), grid_offset.clone(), fill_base_or_quote.clone());
        // check balance
        require!(self.internal_get_user_balance(&user, &(pair.base_token)) >= base_amount_sell, LESS_BASE_TOKEN);
        require!(self.internal_get_user_balance(&user, &(pair.quote_token)) >= quote_amount_buy, LESS_QUOTE_TOKEN);

        // create bot id
        let next_bot_id = format!("GRID:{}", self.internal_get_and_use_next_bot_id().to_string());

        // create bot
        let mut new_grid_bot = GridBot {active: false, user: user.clone(), bot_id: next_bot_id.clone(), closed: false, pair_id, grid_type,
            grid_sell_count: grid_sell_count.clone(), grid_buy_count: grid_buy_count.clone(), grid_rate, grid_offset,
            first_base_amount, first_quote_amount, last_base_amount, last_quote_amount, fill_base_or_quote,
            trigger_price, trigger_price_above_or_below: false, take_profit_price, stop_loss_price, valid_until_time,
            total_quote_amount: quote_amount_buy, total_base_amount: base_amount_sell, revenue: U256C::from(0)
        };

        // record storage fee
        self.storage_fee += env::attached_deposit();

        // request token price
        self.get_price_for_create_bot(&pair, &user, slippage, &entry_price, &mut new_grid_bot);
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

        self.internal_close_bot(&bot_id, &mut bot, &pair);
    }

    pub fn auto_close_bot(&mut self, bot_id: String) {
        require!(self.bot_map.contains_key(&bot_id), BOT_NOT_EXIST);
        let mut bot = self.bot_map.get(&bot_id).unwrap().clone();
        let pair = self.pair_map.get(&bot.pair_id).unwrap().clone();

        self.get_price_for_close_bot(&pair, &mut bot);
    }

    pub fn claim(&mut self, bot_id: String) {
        require!(self.bot_map.contains_key(&bot_id), BOT_NOT_EXIST);
        let mut bot = self.bot_map.get(&bot_id).unwrap().clone();
        let pair = self.pair_map.get(&(bot.pair_id)).unwrap().clone();
        // harvest revenue
        let (revenue_token, revenue) = self.internal_harvest_revenue(&mut bot, &pair);
        self.internal_withdraw(&(bot.user), &revenue_token, revenue);
        self.bot_map.insert(&bot_id, &bot);
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
    pub fn withdraw_protocol_fee(&mut self, token: AccountId, to_user: AccountId, amount: U256C) {
        self.assert_owner();
        require!(self.protocol_fee_map.contains_key(&token), INVALID_TOKEN);
        let protocol_fee = self.internal_get_protocol_fee(&token);
        require!(protocol_fee >= amount, INVALID_AMOUNT);
        self.internal_withdraw_protocol_fee(&to_user, &token, amount);
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
                Self::ext(self.owner_id.clone())
                    .with_static_gas(GAS_FOR_AFTER_FT_TRANSFER)
                    .after_ft_balance_of_for_withdraw_unowned_asset(
                        token.clone(),
                        to_user,
                    )
            );
    }

    #[payable]
    pub fn set_protocol_fee_rate(&mut self, new_protocol_fee_rate: U256C) {
        self.assert_owner();
        require!(new_protocol_fee_rate.as_u128() <= MAX_PROTOCOL_FEE, INVALID_PROTOCOL_FEE);
        self.protocol_fee_rate = new_protocol_fee_rate.as_u128();
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
    pub fn register_pair(&mut self, base_token: AccountId, quote_token: AccountId, base_min_deposit: U256C, quote_min_deposit: U256C, base_oracle_id: String, quote_oracle_id: String) {
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
    pub fn set_min_deposit(&mut self, token: AccountId, min_deposit: U256C) {
        self.assert_owner();
        self.deposit_limit_map.insert(&token, &min_deposit);
    }

    #[payable]
    pub fn storage_deposit(&mut self, token: AccountId, storage_fee: U256C) {
        require!(env::predecessor_account_id() == self.owner_id, ERR_NOT_ALLOWED);
        require!(env::attached_deposit() == storage_fee.as_u128(), LESS_TOKEN_STORAGE_FEE);
        self.internal_storage_deposit(&env::current_account_id(), &token, storage_fee.as_u128());
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

    // #[payable]
    // pub fn set_oracle_price(&mut self, price: U256C, pair_id: String) {
    //     self.assert_owner();
    //     let price_info = OraclePrice {
    //         valid_timestamp: env::block_timestamp_ms() + 3600000,
    //         price,
    //     };
    //     self.oracle_price_map.insert(&pair_id, &price_info);
    // }
}
