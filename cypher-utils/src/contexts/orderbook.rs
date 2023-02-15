#![allow(clippy::ptr_offset_with_cast)]
use agnostic_orderbook::state::{
    critbit::{Slab as AobSlab, INNER_FLAG},
    AccountTag,
};
use anchor_spl::dex::serum_dex::state::MarketState;
use arrayref::array_refs;
use cypher_client::{
    aob::{load_book_side, CallBackInfo},
    serum::Slab,
    Market, Side,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::{fmt::Debug, sync::Arc};

use crate::accounts_cache::AccountsCache;

use super::ContextError;

/// A trait that can be used to generically get data for both AOB and Serum Order Books.
pub trait GenericOrderBook: Send + Sync {
    /// Gets the bids on the book.
    fn get_bids(&self) -> Vec<Order>;

    /// Gets the asks on the book.
    fn get_asks(&self) -> Vec<Order>;
}

/// Represents an order.
#[derive(Default, Clone)]
pub struct Order {
    /// The order side.
    pub side: Side,
    /// The price of the order.
    pub price: u64,
    /// The base quantity of the order.
    pub base_quantity: u64,
    /// The quote quantity of the order.
    pub quote_quantity: u64,
    /// The order id.
    pub order_id: u128,
    /// The order id.
    pub client_order_id: u64,
    /// The maximum timestamp at which it can be filled.
    pub max_ts: u64,
}

impl Debug for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Order")
            .field("side", &format!("{:?}", self.side))
            .field("price", &format!("{}", self.price))
            .field("base_quantity", &format!("{}", self.base_quantity))
            .field("quote_quantity", &format!("{}", self.quote_quantity))
            .field("order_id", &format!("{}", self.order_id))
            .field("client_order_id", &format!("{}", self.client_order_id))
            .field("max_ts", &format!("{}", self.max_ts))
            .finish()
    }
}

/// Gets orders from the AOB's [`Slab`] for a given [`Market`].
///
/// ### Panics
///
/// Panics if there is an overflow doing conversions from lots.
fn get_aob_orders(market: &dyn Market, slab: AobSlab<CallBackInfo>, side: Side) -> Vec<Order> {
    let mut vec: Vec<Order> = Vec::new();
    let ascending = side == Side::Ask;

    let mut search_stack: Vec<u32> = if slab.header.leaf_count == 0 {
        vec![]
    } else {
        vec![slab.root().unwrap()]
    };

    while let Some(current) = search_stack.pop() {
        if current & INNER_FLAG == 0 {
            let node = slab.leaf_nodes[current as usize];
            let scaled_price = node.price();
            let base_quantity = market.unscale_base_amount(node.base_quantity).unwrap();
            let quote_quantity = market
                .get_quote_from_base(base_quantity, scaled_price)
                .unwrap();
            vec.push(Order {
                side,
                price: scaled_price >> 32,
                base_quantity,
                quote_quantity,
                order_id: node.key,
                client_order_id: u64::default(), // The AOB does not store `client_order_id`, cypher stores it in the `OrdersAccount`.
                max_ts: node.max_ts,
            })
        } else {
            let n = &slab.inner_nodes[(!current) as usize];
            search_stack.push(n.children[ascending as usize]);
            search_stack.push(n.children[!ascending as usize]);
            continue;
        }
    }
    vec
}

/// Gets orders from Serum's [`Slab`] for a given [`MarketState`].
///
/// ### Panics
///
/// Panics if there is an overflow doing conversions from lots.
fn get_serum_orders(market: &MarketState, slab: &Slab, side: Side) -> Vec<Order> {
    let ascending = side == Side::Ask;
    let leafs = slab.get_depth(slab.header().leaf_count, ascending);

    leafs
        .iter()
        .map(|l| {
            let price = l.price();
            let base_quantity = l.quantity().checked_mul(market.coin_lot_size).unwrap();
            let quote_quantity = price.checked_mul(base_quantity).unwrap();
            Order {
                side,
                price,
                base_quantity,
                quote_quantity,
                order_id: l.order_id(),
                client_order_id: l.client_order_id(),
                max_ts: u64::MAX, // This version of Serum does not have TIF capability.
            }
        })
        .collect::<Vec<Order>>()
}

/// Represents an orderbook state.
#[derive(Default, Clone)]
pub struct OrderBook {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
}

impl Debug for OrderBook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderBook")
            .field("bids_count", &format!("{}", self.bids.len(),))
            .field("asks_count", &format!("{}", self.asks.len()))
            .finish()
    }
}

impl OrderBook {
    pub fn new(bids: Vec<Order>, asks: Vec<Order>) -> Self {
        Self { bids, asks }
    }

    /// Gets the native impact price for the given size and order side.
    /// The returning value, if it exists, already represents the lot price.
    ///
    /// If not enough liquidity is available on the book to match the requested size,
    /// this method returns none.
    pub fn get_impact_price(&self, size: u64, side: Side) -> Option<u64> {
        let mut cumulative_size = 0;
        let mut impact_price;

        if side == Side::Ask {
            for bid in self.bids.iter() {
                println!("Price: {}", bid.price);
                impact_price = bid.price;
                cumulative_size += bid.base_quantity;
                if cumulative_size >= size {
                    return Some(impact_price);
                }
            }
        } else {
            for ask in self.asks.iter() {
                println!("Price: {}", ask.price);
                impact_price = ask.price;
                cumulative_size += ask.base_quantity;
                if cumulative_size >= size {
                    return Some(impact_price);
                }
            }
        }

        None
    }
}

/// Represents an AOB [`OrderBook`].
#[derive(Default)]
pub struct AgnosticOrderBookContext {
    pub market: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub state: OrderBook,
}

impl GenericOrderBook for AgnosticOrderBookContext {
    fn get_bids(&self) -> Vec<Order> {
        self.state.bids.clone()
    }

    fn get_asks(&self) -> Vec<Order> {
        self.state.asks.clone()
    }
}

impl AgnosticOrderBookContext {
    /// Creates a new [`AgnosticOrderBookContext`].
    pub fn new(market: &Pubkey, bids: &Pubkey, asks: &Pubkey, state: OrderBook) -> Self {
        Self {
            market: *market,
            bids: *bids,
            asks: *asks,
            state,
        }
    }

    /// Loads the [`AgnosticOrderBookContext`].
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request
    /// or the [`Pubkey`]s given are not valid AOB Slab Accounts.
    pub async fn load(
        rpc_client: &Arc<RpcClient>,
        market_state: &dyn Market,
        market: &Pubkey,
        bids: &Pubkey,
        asks: &Pubkey,
    ) -> Result<Self, ContextError> {
        let accounts = match rpc_client.get_multiple_accounts(&[*bids, *asks]).await {
            Ok(a) => a,
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };

        let bids_account = if accounts[0].is_some() {
            accounts[0].as_ref().unwrap()
        } else {
            return Err(ContextError::MissingAccountState);
        };
        let mut bids_data = bids_account.data.clone();
        let bids_state: AobSlab<CallBackInfo> = load_book_side(&mut bids_data, AccountTag::Bids);

        let asks_account = if accounts[1].is_some() {
            accounts[1].as_ref().unwrap()
        } else {
            return Err(ContextError::MissingAccountState);
        };
        let mut asks_data = asks_account.data.clone();
        let asks_state: AobSlab<CallBackInfo> = load_book_side(&mut asks_data, AccountTag::Asks);

        let bid_orders = get_aob_orders(market_state, bids_state, Side::Bid);
        let ask_orders = get_aob_orders(market_state, asks_state, Side::Ask);

        Ok(Self::new(
            market,
            bids,
            asks,
            OrderBook::new(bid_orders, ask_orders),
        ))
    }

    /// Loads the [`AgnosticOrderBookContext`] from the given [`AccountsCache`], if the given Market's
    /// orderbook accounts state exists in the cache.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn from_cache(
        cache: Arc<AccountsCache>,
        market_state: &dyn Market,
        market: &Pubkey,
        bids: &Pubkey,
        asks: &Pubkey,
    ) -> Result<Self, ContextError> {
        let bids_account_state = match cache.get(bids) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };
        let mut bids_data = bids_account_state.data.clone();
        let bids_state: AobSlab<CallBackInfo> = load_book_side(&mut bids_data, AccountTag::Bids);

        let asks_account_state = match cache.get(asks) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };
        let mut asks_data = asks_account_state.data.clone();
        let asks_state: AobSlab<CallBackInfo> = load_book_side(&mut asks_data, AccountTag::Asks);

        let bid_orders = get_aob_orders(market_state, bids_state, Side::Bid);
        let ask_orders = get_aob_orders(market_state, asks_state, Side::Ask);

        Ok(Self::new(
            market,
            bids,
            asks,
            OrderBook::new(bid_orders, ask_orders),
        ))
    }

    /// Loads one [`Side`] of the [`AgnosticOrderBookContext`] from the given account data.
    pub fn from_account_data(
        market: &Pubkey,
        bids: &Pubkey,
        asks: &Pubkey,
        market_state: &dyn Market,
        data: &[u8],
        side: Side,
    ) -> Self {
        let account_tag = if side == Side::Bid {
            AccountTag::Bids
        } else {
            AccountTag::Asks
        };
        let mut data = data.to_vec();
        let side_state: AobSlab<CallBackInfo> = load_book_side(&mut data, account_tag);

        let orders = get_aob_orders(market_state, side_state, side);

        let book = if side == Side::Bid {
            OrderBook::new(orders, Vec::new())
        } else {
            OrderBook::new(Vec::new(), orders)
        };
        Self::new(market, bids, asks, book)
    }

    /// Reloads one [`Side`] of the [`AgnosticOrderBookContext`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn reload_from_account_data(&mut self, market_state: &dyn Market, data: &[u8], side: Side) {
        let account_tag = if side == Side::Bid {
            AccountTag::Bids
        } else {
            AccountTag::Asks
        };
        let mut data = data.to_vec();
        let side_state: AobSlab<CallBackInfo> = load_book_side(&mut data, account_tag);

        let opposite_side_state = if side == Side::Bid {
            self.state.asks.clone() // take the asks if we're updating the bids
        } else {
            self.state.bids.clone() // take the bids if we're updating the asks
        };

        let orders = get_aob_orders(market_state, side_state, side);
        self.state = if side == Side::Bid {
            OrderBook::new(orders, opposite_side_state)
        } else {
            OrderBook::new(opposite_side_state, orders)
        };
    }

    /// Reloads the [`AgnosticOrderBookContext`] from the given [`AccountsCache`],
    /// if the corresponding Slab's account state exists in the cache.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn reload_from_cache(
        &mut self,
        cache: Arc<AccountsCache>,
        market_state: &dyn Market,
    ) -> Result<(), ContextError> {
        let bids_account_state = match cache.get(&self.bids) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };
        let mut bids_data = bids_account_state.data.clone();
        let bids_state: AobSlab<CallBackInfo> = load_book_side(&mut bids_data, AccountTag::Bids);

        let asks_account_state = match cache.get(&self.asks) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };
        let mut asks_data = asks_account_state.data.clone();
        let asks_state: AobSlab<CallBackInfo> = load_book_side(&mut asks_data, AccountTag::Asks);

        let bid_orders = get_aob_orders(market_state, bids_state, Side::Bid);
        let ask_orders = get_aob_orders(market_state, asks_state, Side::Ask);

        self.state = OrderBook::new(bid_orders, ask_orders);

        Ok(())
    }

    /// Gets the native impact price for the given size and order side.
    /// The returning value, if it exists, already represents the lot price.
    ///
    /// If not enough liquidity is available on the book to match the requested size,
    /// this method returns none.
    pub fn get_impact_price(&self, size: u64, side: Side) -> Option<u64> {
        self.state.get_impact_price(size, side)
    }
}

/// Represents a Serum [OrderBook].
#[derive(Default)]
pub struct SerumOrderBookContext {
    pub market: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub state: OrderBook,
}

impl GenericOrderBook for SerumOrderBookContext {
    fn get_bids(&self) -> Vec<Order> {
        self.state.bids.clone()
    }

    fn get_asks(&self) -> Vec<Order> {
        self.state.asks.clone()
    }
}

impl SerumOrderBookContext {
    /// Creates a new [`SerumOrderBookContext`].
    pub fn new(market: &Pubkey, bids: &Pubkey, asks: &Pubkey, state: OrderBook) -> Self {
        Self {
            market: *market,
            bids: *bids,
            asks: *asks,
            state,
        }
    }

    /// Loads the [`SerumOrderBookContext`].
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request
    /// or the [`Pubkey`]s given are not valid Serum Slab Accounts.
    #[allow(clippy::ptr_offset_with_cast)]
    pub async fn load(
        rpc_client: &Arc<RpcClient>,
        market_state: &MarketState,
        market: &Pubkey,
        bids: &Pubkey,
        asks: &Pubkey,
    ) -> Result<Self, ContextError> {
        let accounts = match rpc_client.get_multiple_accounts(&[*bids, *asks]).await {
            Ok(a) => a,
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };

        let bids_account = if accounts[0].is_some() {
            accounts[0].as_ref().unwrap()
        } else {
            return Err(ContextError::MissingAccountState);
        };
        let (_bid_head, bid_data, _bid_tail) = array_refs![&bids_account.data, 5; ..; 7];
        let bid_data = &mut bid_data[8..].to_vec();
        let bids_state = Slab::new(bid_data);

        let asks_account = if accounts[1].is_some() {
            accounts[1].as_ref().unwrap()
        } else {
            return Err(ContextError::MissingAccountState);
        };
        let (_ask_head, ask_data, _ask_tai) = array_refs![&asks_account.data, 5; ..; 7];
        let ask_data = &mut ask_data[8..].to_vec();
        let asks_state = Slab::new(ask_data);

        let bid_orders = get_serum_orders(market_state, bids_state, Side::Bid);
        let ask_orders = get_serum_orders(market_state, asks_state, Side::Ask);

        Ok(Self::new(
            market,
            bids,
            asks,
            OrderBook::new(bid_orders, ask_orders),
        ))
    }

    /// Loads the [`SerumOrderBookContext`] from the given [`AccountsCache`], if the given Market's
    /// account state exists in the cache.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    #[allow(clippy::ptr_offset_with_cast)]
    pub fn from_cache(
        cache: Arc<AccountsCache>,
        market_state: &MarketState,
        market: &Pubkey,
        bids: &Pubkey,
        asks: &Pubkey,
    ) -> Result<Self, ContextError> {
        let bids_account_state = match cache.get(bids) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };
        let (_bid_head, bid_data, _bid_tail) = array_refs![&bids_account_state.data, 5; ..; 7];
        let bid_data = &mut bid_data[8..].to_vec();
        let bids_state = Slab::new(bid_data);

        let asks_account_state = match cache.get(asks) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };
        let (_ask_head, ask_data, _ask_tai) = array_refs![&asks_account_state.data, 5; ..; 7];
        let ask_data = &mut ask_data[8..].to_vec();
        let asks_state = Slab::new(ask_data);

        let bid_orders = get_serum_orders(market_state, bids_state, Side::Bid);
        let ask_orders = get_serum_orders(market_state, asks_state, Side::Ask);

        Ok(Self::new(
            market,
            bids,
            asks,
            OrderBook::new(bid_orders, ask_orders),
        ))
    }

    /// Reloads one [`Side`] of the [`SerumOrderBookContext`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn reload_from_account_data(
        &mut self,
        market_state: &MarketState,
        data: &[u8],
        side: Side,
    ) {
        let (_side_head, side_data, _side_tail) = array_refs![&data, 5; ..; 7];
        let side_data = &mut side_data[8..].to_vec();
        let side_state = Slab::new(side_data);

        let opposite_side_state = if side == Side::Bid {
            self.state.asks.clone() // take the asks if we're updating the bids
        } else {
            self.state.bids.clone() // take the bids if we're updating the asks
        };

        let orders = get_serum_orders(market_state, side_state, side);

        self.state = if side == Side::Bid {
            OrderBook::new(orders, opposite_side_state)
        } else {
            OrderBook::new(opposite_side_state, orders)
        };
    }

    /// Loads one [`Side`] of the [`SerumOrderBookContext`] from the given account data.
    #[allow(clippy::ptr_offset_with_cast)]
    pub fn from_account_data(
        market: &Pubkey,
        bids: &Pubkey,
        asks: &Pubkey,
        market_state: &MarketState,
        data: &[u8],
        side: Side,
    ) -> Self {
        let (_side_head, side_data, _side_tail) = array_refs![&data, 5; ..; 7];
        let side_data = &mut side_data[8..].to_vec();
        let side_state = Slab::new(side_data);

        let orders = get_serum_orders(market_state, side_state, side);

        let book = if side == Side::Bid {
            OrderBook::new(orders, Vec::new())
        } else {
            OrderBook::new(Vec::new(), orders)
        };
        Self::new(market, bids, asks, book)
    }

    /// Reloads the [`SerumOrderBookContext`] from the given [`AccountsCache`],
    /// if the corresponding Slab's account state exists in the cache.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    #[allow(clippy::ptr_offset_with_cast)]
    pub fn reload_from_cache(
        &mut self,
        market_state: &MarketState,
        cache: Arc<AccountsCache>,
    ) -> Result<(), ContextError> {
        let bids_account_state = match cache.get(&self.bids) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };
        let (_bid_head, bid_data, _bid_tail) = array_refs![&bids_account_state.data, 5; ..; 7];
        let bid_data = &mut bid_data[8..].to_vec();
        let bids_state = Slab::new(bid_data);

        let asks_account_state = match cache.get(&self.asks) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };
        let (_ask_head, ask_data, _ask_tail) = array_refs![&asks_account_state.data, 5; ..; 7];
        let ask_data = &mut ask_data[8..].to_vec();
        let asks_state = Slab::new(ask_data);

        let bid_orders = get_serum_orders(market_state, bids_state, Side::Bid);
        let ask_orders = get_serum_orders(market_state, asks_state, Side::Ask);

        self.state = OrderBook::new(bid_orders, ask_orders);

        Ok(())
    }

    /// Gets the native impact price for the given size and order side.
    /// The returning value, if it exists, already represents the lot price.
    ///
    /// If not enough liquidity is available on the book to match the requested size,
    /// this method returns none.
    pub fn get_impact_price(&self, size: u64, side: Side) -> Option<u64> {
        self.state.get_impact_price(size, side)
    }
}
