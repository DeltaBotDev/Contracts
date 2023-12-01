use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use crate::*;
use near_sdk::{Gas, near_bindgen, Promise};
use serde_json::json;
use crate::entity::{GridType, OrderKeyInfo};
use crate::events::emit;

#[near_bindgen]
impl GridBotContract {
    #[payable]
    pub fn create_bot(&mut self, name:String, pair_id: String, slippage: u16, grid_type: GridType,
                      grid_rate: u16, grid_offset: U128C, first_base_amount: U128C, first_quote_amount: U128C,
                      last_base_amount: U128C, last_quote_amount: U128C, fill_base_or_quote: bool, grid_sell_count: u16, grid_buy_count: u16,
                      trigger_price: U128C, take_profit_price: U128C, stop_loss_price: U128C, valid_until_time: u64,
                      entry_price: U128C) {
        assert!(self.status == GridStatus::Running, "PAUSE_OR_SHUTDOWN");
        assert!(self.internal_check_oracle_price(entry_price, pair_id.clone(), slippage) , "ORACLE_PRICE_EXCEPTION");
        assert!(self.pair_map.contains_key(&pair_id), "INVALID_PAIR_ID");
        let pair = self.pair_map.get(&pair_id).unwrap().clone();
        let user = env::predecessor_account_id();

        // calculate all assets
        let (base_amount_sell, quote_amount_buy) = GridBotContract::internal_calculate_bot_assets(first_quote_amount.clone(), last_base_amount.clone(), grid_sell_count.clone(), grid_buy_count.clone(),
                                                       grid_type.clone(), grid_rate.clone(), grid_offset.clone(), fill_base_or_quote.clone());
        // check balance
        assert!(self.internal_get_user_balance(&user, &(pair.base_token)) >= base_amount_sell, "LESS_BASE_TOKEN");
        assert!(self.internal_get_user_balance(&user, &(pair.quote_token)) >= quote_amount_buy, "LESS_QUOTE_TOKEN");

        // transfer assets
        self.internal_transfer_assets_to_lock(user.clone(), pair.base_token.clone(), base_amount_sell);
        self.internal_transfer_assets_to_lock(user.clone(), pair.quote_token.clone(), quote_amount_buy);

        // create bot id
        let next_bot_id = format!("GRID:{}", self.internal_get_and_use_next_bot_id().to_string());
        // initial orders space
        let grid_count = grid_sell_count.clone() as usize + grid_buy_count.clone() as usize;
        self.order_map.insert(next_bot_id.clone(), vec!(Vec::with_capacity(grid_count.clone()), Vec::with_capacity(grid_count.clone())));

        // create bot
        let mut new_grid_bot = GridBot {active: false, user: user.clone(), bot_id: next_bot_id.clone(), closed: false, name, pair_id, grid_type,
            grid_sell_count: grid_sell_count.clone(), grid_buy_count: grid_buy_count.clone(), grid_rate, grid_offset,
            first_base_amount, first_quote_amount, last_base_amount, last_quote_amount, fill_base_or_quote,
            trigger_price, trigger_price_above_or_below: false, take_profit_price, stop_loss_price, valid_until_time,
            total_quote_amount: quote_amount_buy.as_u128(), total_base_amount: base_amount_sell.as_u128(), revenue: 0
        };
        // init active status of bot
        self.internal_init_bot_status(&mut new_grid_bot, entry_price);

        // insert bot
        self.bot_map.insert(next_bot_id.clone(), new_grid_bot);
    }

    pub fn close_bot(&mut self, bot_id: String) {
        assert!(self.bot_map.contains_key(&bot_id), "BOT_NOT_EXIST");
        let bot = self.bot_map.get(&bot_id).unwrap().clone();
        let pair = self.pair_map.get(&bot.pair_id).unwrap().clone();
        // check permission, user self close or take profit or stop loss
        let user = env::predecessor_account_id();
        assert!(self.internal_check_bot_close_permission(&user, &bot), "NO_PERMISSION");

        // sign closed
        self.bot_map.get_mut(&bot_id).unwrap().closed = true;

        // harvest revenue, must fist execute, will split revenue from bot's asset
        let (revenue_token, revenue) = self.internal_harvest_revenue(&bot, &pair, &(bot.user));
        // reget bot
        let bot = self.bot_map.get(&bot_id).unwrap().clone();
        // unlock token
        self.internal_transfer_assets_to_unlock(&(bot.user), &(pair.base_token), bot.total_base_amount.clone());
        self.internal_transfer_assets_to_unlock(&(bot.user), &(pair.quote_token), bot.total_quote_amount.clone());

        // withdraw token
        self.internal_withdraw(&(bot.user), &(pair.base_token), bot.total_base_amount.clone());
        self.internal_withdraw(&(bot.user), &(pair.quote_token), bot.total_quote_amount.clone());
        self.internal_withdraw(&(bot.user), &revenue_token, revenue);
    }

    pub fn withdraw(&mut self, token: AccountId) {
        let user = env::predecessor_account_id();
        let balance = self.internal_get_user_balance(&user, &token);
        self.internal_withdraw(&user, &token, balance.as_u128());
    }

    pub fn take_orders(&mut self, mut take_order: Order, maker_orders: Vec<OrderKeyInfo>) {
        assert!(self.status == GridStatus::Running, "PAUSE_OR_SHUTDOWN");
        assert!(maker_orders.len() > 0, "VALID_MAKER_ORDERS");
        let user = env::predecessor_account_id();
        assert!(self.internal_get_user_balance(&user, &(take_order.token_sell)) >= take_order.amount_sell, "LESS_TOKEN_SELL");
        let taker_amount_sell = take_order.amount_sell.clone();
        let taker_amount_buy = take_order.amount_buy.clone();
        // loop take order
        for maker_order in maker_orders.iter() {
            let (taker_sell, taker_buy) = self.internal_take_order(maker_order.bot_id.clone(), maker_order.forward_or_reverse.clone(), maker_order.level.clone(), &take_order);
            take_order.amount_sell -= taker_sell;
            take_order.amount_buy -= taker_buy;
        }
        // calculate taker actually sell and buy amount
        let total_taker_sell = taker_amount_sell - take_order.amount_sell;
        let total_taker_buy = taker_amount_buy - take_order.amount_buy;
        // transfer taker's asset
        self.internal_reduce_asset(&user, &(take_order.token_sell), total_taker_sell.as_u128());
        self.internal_increase_asset(&user, &(take_order.token_buy), total_taker_buy.as_u128());

        // withdraw for taker
        self.internal_withdraw(&user, &(take_order.token_buy), total_taker_buy.as_u128());
    }

    pub fn claim(&mut self, bot_id: String) {
        assert!(self.bot_map.contains_key(&bot_id), "BOT_NOT_EXIST");
        let bot = self.bot_map.get(&bot_id).unwrap().clone();
        let user = env::predecessor_account_id();
        let pair = self.pair_map.get(&(bot.pair_id)).unwrap().clone();
        // check permission
        assert_eq!(bot.user, user, "NO_PERMISSION");
        // harvest revenue
        let (revenue_token, revenue) = self.internal_harvest_revenue(&bot, &pair, &user);
        self.internal_withdraw(&user, &revenue_token, revenue);
    }

    pub fn trigger_bot(&mut self, bot_id: String) {
        assert!(self.status == GridStatus::Running, "PAUSE_OR_SHUTDOWN");
        let bot = self.bot_map.get(&bot_id).unwrap().clone();
        let oracle_price = self.internal_get_oracle_price(bot.pair_id);
        if bot.trigger_price_above_or_below && bot.trigger_price <= oracle_price {
            self.bot_map.get_mut(&bot_id).unwrap().active = true;
        } else if !bot.trigger_price_above_or_below.clone() && bot.trigger_price.clone() >= oracle_price {
            self.bot_map.get_mut(&bot_id).unwrap().active = true;
        }
    }

    pub fn withdraw_unowned_asset(&mut self, token: AccountId) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "NO_PERMISSION");
        // Promise::new(token)
        //     .function_call(
        //         "ft_balance_of".to_string(),
        //         json!({"account_id": env::current_account_id()}).to_string().into_bytes(),
        //         0,
        //         Gas(0),
        //     )
        //     .then(
        //         self::ext(env::current_account_id())
        //             .with_static_gas(Gas(10_000_000_000_000))
        //             .withdraw_unowned_asset_callback()
        //     ).into()
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
