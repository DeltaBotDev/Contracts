use crate::*;
use near_sdk::{assert_one_yocto, Gas, near_bindgen, Promise, require};
use serde_json::json;
use crate::entity::{GridType, OrderKeyInfo};

#[near_bindgen]
impl GridBotContract {

    #[payable]
    pub fn create_bot(&mut self, pair_id: String, slippage: u16, grid_type: GridType,
                      grid_rate: u16, grid_offset: U256C, first_base_amount: U256C, first_quote_amount: U256C,
                      last_base_amount: U256C, last_quote_amount: U256C, fill_base_or_quote: bool, grid_sell_count: u16, grid_buy_count: u16,
                      trigger_price: U256C, take_profit_price: U256C, stop_loss_price: U256C, valid_until_time: U256C,
                      entry_price: U256C) -> String {
        require!(env::attached_deposit() == STORAGE_FEE, LESS_STORAGE_FEE);
        require!(self.status == GridStatus::Running, PAUSE_OR_SHUTDOWN);
        // last_quote_amount / last_base_amount > first_quote_amount > first_base_amount
        // amount must u128, u128 * u128 <= u256, so, it's ok
        require!(last_quote_amount * first_base_amount > first_quote_amount * last_base_amount , INVALID_FIRST_OR_LAST_AMOUNT);
        // got oracle price
        require!(self.internal_check_oracle_price(entry_price, pair_id.clone(), slippage) , ORACLE_PRICE_EXCEPTION);
        require!(self.pair_map.contains_key(&pair_id), INVALID_PAIR_ID);

        let pair = self.pair_map.get(&pair_id).unwrap().clone();
        let user = env::predecessor_account_id();

        // calculate all assets
        let (base_amount_sell, quote_amount_buy) = GridBotContract::internal_calculate_bot_assets(first_quote_amount.clone(), last_base_amount.clone(), grid_sell_count.clone(), grid_buy_count.clone(),
                                                       grid_type.clone(), grid_rate.clone(), grid_offset.clone(), fill_base_or_quote.clone());
        // check balance
        require!(self.internal_get_user_balance(&user, &(pair.base_token)) >= base_amount_sell, LESS_BASE_TOKEN);
        require!(self.internal_get_user_balance(&user, &(pair.quote_token)) >= quote_amount_buy, LESS_QUOTE_TOKEN);

        // transfer assets
        self.internal_transfer_assets_to_lock(user.clone(), pair.base_token.clone(), base_amount_sell);
        self.internal_transfer_assets_to_lock(user.clone(), pair.quote_token.clone(), quote_amount_buy);

        // create bot id
        let next_bot_id = format!("GRID:{}", self.internal_get_and_use_next_bot_id().to_string());
        // initial orders space
        let grid_count = grid_sell_count.clone() + grid_buy_count.clone();
        // self.order_map.insert(next_bot_id.clone(), vec!(Vec::with_capacity(grid_count.clone()), Vec::with_capacity(grid_count.clone())));
        // self.order_map.insert(&next_bot_id, &(vec![(0..grid_count).map(|_| Order::default()).collect(), (0..grid_count).map(|_| Order::default()).collect()]));
        let grid_orders = GridBotContract::create_default_orders(grid_count);
        self.order_map.insert(&next_bot_id, &grid_orders);

        // create bot
        let mut new_grid_bot = GridBot {active: false, user: user.clone(), bot_id: next_bot_id.clone(), closed: false, pair_id, grid_type,
            grid_sell_count: grid_sell_count.clone(), grid_buy_count: grid_buy_count.clone(), grid_rate, grid_offset,
            first_base_amount, first_quote_amount, last_base_amount, last_quote_amount, fill_base_or_quote,
            trigger_price, trigger_price_above_or_below: false, take_profit_price, stop_loss_price, valid_until_time,
            total_quote_amount: quote_amount_buy, total_base_amount: base_amount_sell, revenue: U256C::from(0)
        };
        // init active status of bot
        self.internal_init_bot_status(&mut new_grid_bot, entry_price);

        // insert bot
        self.bot_map.insert(&(next_bot_id.clone()), &new_grid_bot);

        // record storage fee
        self.storage_fee += env::attached_deposit();
        return next_bot_id.clone();
    }

    #[payable]
    pub fn close_bot(&mut self, bot_id: String) {
        assert_one_yocto();
        require!(self.bot_map.contains_key(&bot_id), BOT_NOT_EXIST);
        let mut bot = self.bot_map.get(&bot_id).unwrap().clone();
        let pair = self.pair_map.get(&bot.pair_id).unwrap().clone();
        // check permission, user self close or take profit or stop loss
        let user = env::predecessor_account_id();
        require!(self.internal_check_bot_close_permission(&user, &bot), NO_PERMISSION);

        // sign closed
        bot.closed = true;

        // harvest revenue, must fist execute, will split revenue from bot's asset
        let (revenue_token, revenue) = self.internal_harvest_revenue(&mut bot, &pair);
        // unlock token
        self.internal_transfer_assets_to_unlock(&(bot.user), &(pair.base_token), bot.total_base_amount.clone());
        self.internal_transfer_assets_to_unlock(&(bot.user), &(pair.quote_token), bot.total_quote_amount.clone());

        // withdraw token
        self.internal_withdraw(&(bot.user), &(pair.base_token), bot.total_base_amount);
        self.internal_withdraw(&(bot.user), &(pair.quote_token), bot.total_quote_amount);
        self.internal_withdraw(&(bot.user), &revenue_token, revenue);

        self.bot_map.insert(&bot_id, &bot);
    }

    #[payable]
    // TODO take can transfer then take orders
    pub fn take_orders(&mut self, take_order: &Order, maker_orders: Vec<OrderKeyInfo>) {
        // assert_one_yocto();
        require!(self.status == GridStatus::Running, PAUSE_OR_SHUTDOWN);
        require!(maker_orders.len() > 0, INVALID_MAKER_ORDERS);
        require!(take_order.amount_sell != U256C::from(0), INVALID_ORDER_AMOUNT);
        require!(take_order.amount_buy != U256C::from(0), INVALID_ORDER_AMOUNT);
        let user = env::predecessor_account_id();
        require!(self.internal_get_user_balance(&user, &(take_order.token_sell)) >= take_order.amount_sell, LESS_TOKEN_SELL);
        let mut took_amount_sell = U256C::from(0);
        let mut took_amount_buy = U256C::from(0);
        // loop take order
        for maker_order in maker_orders.iter() {
            let (taker_sell, taker_buy) = self.internal_take_order(maker_order.bot_id.clone(), maker_order.forward_or_reverse.clone(), maker_order.level.clone(), &take_order, took_amount_sell.clone(), took_amount_buy.clone());
            took_amount_sell += taker_sell;
            took_amount_buy += taker_buy;
        }
        // transfer taker's asset
        self.internal_reduce_asset(&user, &(take_order.token_sell), &took_amount_sell);
        self.internal_increase_asset(&user, &(take_order.token_buy), &took_amount_buy);

        // withdraw for taker
        self.internal_withdraw(&user, &(take_order.token_buy), took_amount_buy);
        // TODO need to withdraw the left asset, with return
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
        let oracle_price = self.internal_get_oracle_price(bot.pair_id.clone());
        if bot.trigger_price_above_or_below && bot.trigger_price <= oracle_price {
            // self.bot_map.get_mut(&bot_id).unwrap().active = true;
            bot.active = true;
            self.bot_map.insert(&bot_id, &bot);
        } else if !bot.trigger_price_above_or_below.clone() && bot.trigger_price.clone() >= oracle_price {
            // self.bot_map.get_mut(&bot_id).unwrap().active = true;
            bot.active = true;
            self.bot_map.insert(&bot_id, &bot);
        }
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
    pub fn register_pair(&mut self, base_token: AccountId, quote_token: AccountId, base_min_deposit: U256C, quote_min_deposit: U256C) {
        self.assert_owner();
        require!(base_token != quote_token, INVALID_TOKEN);
        let pair_key = GridBotContract::internal_get_pair_key(base_token.clone(), quote_token.clone());
        require!(!self.pair_map.contains_key(&pair_key), PAIR_EXIST);
        let pair = Pair{
            base_token: base_token.clone(),
            quote_token: quote_token.clone(),
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

    // TODO Test
    #[payable]
    pub fn set_oracle_price(&mut self, price: U256C, pair_id: String) {
        self.assert_owner();
        let price_info = OraclePrice {
            valid_timestamp: env::block_timestamp_ms() + 3600000,
            price,
        };
        self.oracle_price_map.insert(&pair_id, &price_info);
    }
}
