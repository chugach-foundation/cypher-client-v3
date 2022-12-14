use anchor_spl::dex::serum_dex::state::OpenOrders;
use async_trait::async_trait;
use cypher_client::{
    serum::parse_dex_account, utils::get_zero_copy_account, OpenOrder, OrdersAccount, Side,
};
use solana_sdk::pubkey::Pubkey;
use tokio::sync::RwLock;

use super::{GenericOrderBook, Order};

/// A trait that can be used to generically get data for both AOB and Serum Orders Accounts.
pub trait GenericOpenOrders: Send + Sync {
    /// Gets open orders in the orders account and maps them with the existing orders on the given Order Book.
    ///
    /// Callee must make sure that the given [GenericOrderBook] is of a type that is compatible with the
    fn get_open_orders(&self, orderbook: &dyn GenericOrderBook) -> Vec<Order>;
}

/// Represents the Cypher Open Orders Account Context (used for the AOB).
pub struct AgnosticOpenOrdersContext {
    pub account: Pubkey,
    pub state: Box<OrdersAccount>,
}

impl Default for AgnosticOpenOrdersContext {
    fn default() -> Self {
        Self {
            account: Pubkey::default(),
            state: Box::new(OrdersAccount {
                order_count: u8::default(),
                padding: [0; 7],
                authority: Pubkey::default(),
                market: Pubkey::default(),
                master_account: Pubkey::default(),
                base_token_free: [0; 24],
                base_token_locked: [0; 24],
                quote_token_free: [0; 24],
                quote_token_locked: [0; 24],
                open_orders: [OpenOrder::default(); 128],
            }),
        }
    }
}

impl GenericOpenOrders for AgnosticOpenOrdersContext {
    fn get_open_orders(&self, orderbook: &dyn GenericOrderBook) -> Vec<Order> {
        let mut orders = Vec::new();

        let open_orders = self.state.open_orders;

        for i in 0..open_orders.len() {
            let order = self.state.open_orders[i];

            if order.order_id != u128::default() {
                let ob_order = get_orderbook_line(orderbook, order.order_id, order.side.into());

                if ob_order.is_some() {
                    let ob_order = ob_order.unwrap();
                    orders.push(Order {
                        side: order.side,
                        order_id: order.order_id,
                        client_order_id: order.client_order_id,
                        price: ob_order.price,
                        base_quantity: ob_order.base_quantity,
                        quote_quantity: ob_order.quote_quantity,
                        max_ts: ob_order.max_ts,
                    })
                }
            }
        }

        orders
    }
}

impl AgnosticOpenOrdersContext {
    /// Creates a new [`AgnosticOpenOrdersContext`].
    pub fn new(account: &Pubkey, state: Box<OrdersAccount>) -> Self {
        Self {
            account: *account,
            state,
        }
    }

    /// Loads the [`OrdersAccount`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account data is invalid.
    pub fn from_account_data(account: &Pubkey, account_data: &[u8]) -> Self {
        let state = get_zero_copy_account::<OrdersAccount>(account_data);

        Self::new(account, state)
    }

    /// Loads the [`OrdersAccount`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account data is invalid.
    pub fn reload_from_account_data(&mut self, account_data: &[u8]) {
        self.state = get_zero_copy_account::<OrdersAccount>(account_data);
    }
}

/// Represents the Serum Open Orders Account Context.
pub struct SerumOpenOrdersContext {
    pub account: Pubkey,
    pub state: OpenOrders,
}

impl Default for SerumOpenOrdersContext {
    fn default() -> Self {
        Self {
            account: Pubkey::default(),
            state: OpenOrders {
                account_flags: u64::default(),
                market: [0; 4],
                owner: [0; 4],
                native_coin_free: u64::default(),
                native_coin_total: u64::default(),
                native_pc_free: u64::default(),
                native_pc_total: u64::default(),
                free_slot_bits: u128::default(),
                is_bid_bits: u128::default(),
                orders: [0; 128],
                client_order_ids: [0; 128],
                referrer_rebates_accrued: u64::default(),
            },
        }
    }
}

impl GenericOpenOrders for SerumOpenOrdersContext {
    fn get_open_orders(&self, orderbook: &dyn GenericOrderBook) -> Vec<Order> {
        let mut orders = Vec::new();

        let order_ids = self.state.orders;
        let client_order_ids = self.state.client_order_ids;

        for i in 0..order_ids.len() {
            let order_id = order_ids[i];
            let client_order_id = client_order_ids[i];

            if order_id != u128::default() {
                let price = (order_id >> 64) as u64;
                let side = self.state.slot_side(i as u8).unwrap();
                let ob_order = get_orderbook_line(orderbook, order_id, side.into());

                if ob_order.is_some() {
                    let ob_order = ob_order.unwrap();
                    orders.push(Order {
                        side: side.into(),
                        order_id,
                        client_order_id,
                        price,
                        base_quantity: ob_order.base_quantity,
                        quote_quantity: ob_order.quote_quantity,
                        max_ts: ob_order.max_ts,
                    })
                }
            }
        }

        orders
    }
}

impl SerumOpenOrdersContext {
    /// Creates a new [`SerumOpenOrdersContext`].
    pub fn new(account: &Pubkey, state: OpenOrders) -> Self {
        Self {
            account: *account,
            state,
        }
    }

    /// Loads the [`OrdersAccount`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account data is invalid.
    pub fn from_account_data(account: &Pubkey, account_data: &[u8]) -> Self {
        let state = parse_dex_account::<OpenOrders>(account_data);

        Self::new(account, state)
    }

    /// Loads the [`OpenOrders`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account data is invalid.
    pub fn reload_from_account_data(&mut self, account_data: &[u8]) {
        self.state = parse_dex_account::<OpenOrders>(account_data);
    }
}

fn get_orderbook_line(
    orderbook: &dyn GenericOrderBook,
    order_id: u128,
    side: Side,
) -> Option<Order> {
    match side {
        Side::Bid => {
            let bids = orderbook.get_bids();

            for order in bids.iter() {
                if order.order_id == order_id {
                    return Some(order.clone());
                }
            }

            None
        }
        Side::Ask => {
            let asks = orderbook.get_asks();

            for order in asks.iter() {
                if order.order_id == order_id {
                    return Some(order.clone());
                }
            }

            None
        }
    }
}
