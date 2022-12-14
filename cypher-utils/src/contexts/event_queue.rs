use agnostic_orderbook::state::{event_queue::FillEvent, Side as AobSide};
use anchor_spl::dex::serum_dex::{
    matching::Side as DexSide,
    state::{Event, EventView, QueueHeader},
};
use async_trait::async_trait;
use cypher_client::{
    aob::{parse_aob_event_queue, CallBackInfo},
    serum::{parse_dex_event_queue, remove_dex_account_padding},
    Side,
};
use num_traits::cast::FromPrimitive;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::accounts_cache::AccountsCache;

use super::ContextError;

/// Represents an order fill.
#[derive(Debug, Clone)]
pub struct Fill {
    /// The total base quantity.
    pub base_quantity: u64,
    /// The total quote quantity.
    pub quote_quantity: u64,
    /// The price.
    pub price: u64,
    /// The side of the taker order.
    pub taker_side: Side,
    /// The maker order id.
    pub maker_order_id: u128,
}

/// A trait that can be used to generically get data for both AOB and Serum Event Queues.
pub trait GenericEventQueue: Send + Sync {
    /// Gets the fills in the Event Queue.
    fn get_fills(&self) -> Vec<Fill>;

    /// Gets the fills in the Event Queue that have occurred since the given sequence number.
    fn get_fills_since(&self, seq: u64) -> Vec<Fill>;
}

/// Represents an AOB Event Queue.
pub struct AgnosticEventQueueContext {
    pub market: Pubkey,
    pub event_queue: Pubkey,
    pub count: u64,
    pub head: u64,
    pub events: Vec<FillEvent>,
    pub callbacks: Vec<CallBackInfo>,
}

impl Default for AgnosticEventQueueContext {
    fn default() -> Self {
        Self {
            market: Pubkey::default(),
            event_queue: Pubkey::default(),
            count: 0,
            head: 0,
            events: Vec::new(),
            callbacks: Vec::new(),
        }
    }
}

impl GenericEventQueue for AgnosticEventQueueContext {
    fn get_fills(&self) -> Vec<Fill> {
        let events = &self.events;
        let mut fills = Vec::new();

        for event in events.iter() {
            if event.maker_order_id != u128::default() && event.base_size != 0 {
                let aob_side = AobSide::from_u8(event.taker_side).unwrap();
                let taker_side = if aob_side == AobSide::Ask {
                    Side::Ask
                } else {
                    Side::Bid
                };
                fills.push(Fill {
                    base_quantity: event.base_size,
                    quote_quantity: event.quote_size,
                    price: event.quote_size / event.base_size,
                    taker_side,
                    maker_order_id: event.maker_order_id,
                });
            }
        }

        fills
    }

    fn get_fills_since(&self, seq: u64) -> Vec<Fill> {
        let head = self.head;
        let events = &self.events;
        let (sliced_events, _) = events.split_at(head as usize);
        let mut fills = Vec::new();

        for event in sliced_events {
            if event.maker_order_id != u128::default() && event.base_size != 0{
                let aob_side = AobSide::from_u8(event.taker_side).unwrap();
                let taker_side = if aob_side == AobSide::Ask {
                    Side::Ask
                } else {
                    Side::Bid
                };
                fills.push(Fill {
                    base_quantity: event.base_size,
                    quote_quantity: event.quote_size,
                    price: event.quote_size / event.base_size,
                    taker_side,
                    maker_order_id: event.maker_order_id,
                });
            }
        }

        fills
    }
}

impl AgnosticEventQueueContext {
    /// Creates a new [`AgnosticEventQueueContext`].
    pub fn new(
        market: &Pubkey,
        event_queue: &Pubkey,
        count: u64,
        head: u64,
        events: Vec<FillEvent>,
        callbacks: Vec<CallBackInfo>,
    ) -> Self {
        Self {
            market: *market,
            event_queue: *event_queue,
            count,
            head,
            events,
            callbacks,
        }
    }

    /// Loads the [`AgnosticEventQueueContext`].
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request
    /// or the [`Pubkey`]s given are not valid AOB Event Queue Accounts.
    pub async fn load(
        rpc_client: &Arc<RpcClient>,
        market: &Pubkey,
        event_queue: &Pubkey,
    ) -> Result<Self, ContextError> {
        let account_data = match rpc_client.get_account_data(event_queue).await {
            Ok(a) => a,
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };
        let (eq_header, fills, callbacks) = parse_aob_event_queue(&account_data);
        Ok(Self::new(
            market,
            event_queue,
            eq_header.count,
            eq_header.head,
            fills.to_vec(),
            callbacks.to_vec(),
        ))
    }

    /// Loads the [`AgnosticEventQueueContext`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn from_account_data(
        market: &Pubkey,
        event_queue: &Pubkey,
        data: &[u8],
    ) -> Self {
        let (eq_header, fills, callbacks) = parse_aob_event_queue(&data);

        Self::new(
            market,
            event_queue,
            eq_header.count,
            eq_header.head,
            fills.to_vec(),
            callbacks.to_vec(),
        )
    }

    /// Loads the [`AgnosticEventQueueContext`] from the given [`AccountsCache`],
    /// if the corresponding EventQueue's account state exists in the cache.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn from_cache(
        cache: Arc<AccountsCache>,
        market: &Pubkey,
        event_queue: &Pubkey,
    ) -> Result<Self, ContextError> {
        let eq_state = match cache.get(event_queue) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };

        let (eq_header, fills, callbacks) = parse_aob_event_queue(&eq_state.data);

        Ok(Self::new(
            market,
            event_queue,
            eq_header.count,
            eq_header.head,
            fills.to_vec(),
            callbacks.to_vec(),
        ))
    }

    /// Reloads the [`AgnosticEventQueueContext`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn reload_from_account_data(&mut self, data: &[u8]) {
        let (eq_header, new_fills, new_callbacks) = parse_aob_event_queue(&data);

        self.count = eq_header.count;
        self.head = eq_header.head;
        self.callbacks = new_callbacks.to_vec();
        self.events = new_fills.to_vec();
    }

    /// Reloads the [`AgnosticEventQueueContext`] from the given [`AccountsCache`],
    /// if the corresponding EventQueue's account state exists in the cache.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn reload_from_cache(
        &mut self,
        cache: Arc<AccountsCache>,
    ) -> Result<(), ContextError> {
        let eq_state = match cache.get(&self.event_queue) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };

        let (eq_header, new_fills, new_callbacks) = parse_aob_event_queue(&eq_state.data);

        self.count = eq_header.count;
        self.head = eq_header.head;
        self.callbacks = new_callbacks.to_vec();
        self.events = new_fills.to_vec();

        Ok(())
    }
}

/// Represents a Serum Event Queue.
pub struct SerumEventQueueContext {
    pub market: Pubkey,
    pub event_queue: Pubkey,
    pub count: u64,
    pub head: u64,
    pub events: Vec<Event>,
}

impl Default for SerumEventQueueContext {
    fn default() -> Self {
        Self {
            market: Pubkey::default(),
            event_queue: Pubkey::default(),
            count: 0,
            head: 0,
            events: Vec::new(),
        }
    }
}

impl GenericEventQueue for SerumEventQueueContext {
    fn get_fills(&self) -> Vec<Fill> {
        let events = &self.events;
        let mut fills = Vec::new();

        for event in events.iter() {
            match event.as_view() {
                Ok(a) => {
                    match a {
                        EventView::Fill {
                            side,
                            maker,
                            native_qty_paid,
                            native_qty_received,
                            order_id,
                            ..
                        } => {
                            if order_id != u128::default() {
                                let taker_side = if maker {
                                    // is maker
                                    if side == DexSide::Ask {
                                        Side::Bid
                                    } else {
                                        Side::Ask
                                    }
                                } else {
                                    // not maker
                                    if side == DexSide::Ask {
                                        Side::Ask
                                    } else {
                                        Side::Bid
                                    }
                                };
                                let base_quantity = if side == DexSide::Ask {
                                    native_qty_paid
                                } else {
                                    native_qty_received
                                };
                                let quote_quantity = if side == DexSide::Ask {
                                    native_qty_received
                                } else {
                                    native_qty_paid
                                };
                                fills.push(Fill {
                                    base_quantity,
                                    quote_quantity,
                                    price: quote_quantity / base_quantity,
                                    taker_side,
                                    maker_order_id: order_id,
                                });
                            }
                        }
                        _ => continue,
                    }
                }
                Err(_) => continue,
            };
        }

        fills
    }

    fn get_fills_since(&self, seq: u64) -> Vec<Fill> {
        let head = self.head;
        let events = &self.events;
        let (sliced_events, _) = events.split_at(head as usize);
        let mut fills = Vec::new();

        for event in sliced_events {
            match event.as_view() {
                Ok(a) => {
                    match a {
                        EventView::Fill {
                            side,
                            maker,
                            native_qty_paid,
                            native_qty_received,
                            order_id,
                            ..
                        } => {
                            if order_id != u128::default() {
                                let taker_side = if maker {
                                    // is maker
                                    if side == DexSide::Ask {
                                        Side::Bid
                                    } else {
                                        Side::Ask
                                    }
                                } else {
                                    // not maker
                                    if side == DexSide::Ask {
                                        Side::Ask
                                    } else {
                                        Side::Bid
                                    }
                                };
                                let base_quantity = if side == DexSide::Ask {
                                    native_qty_received
                                } else {
                                    native_qty_paid
                                };
                                let quote_quantity = if side == DexSide::Ask {
                                    native_qty_paid
                                } else {
                                    native_qty_received
                                };
                                fills.push(Fill {
                                    base_quantity,
                                    quote_quantity,
                                    price: quote_quantity / base_quantity,
                                    taker_side,
                                    maker_order_id: order_id,
                                });
                            }
                        }
                        _ => continue,
                    }
                }
                Err(_) => continue,
            };
        }

        fills
    }
}

impl SerumEventQueueContext {
    /// Creates a new [`SerumEventQueueContext`].
    pub fn new(
        market: &Pubkey,
        event_queue: &Pubkey,
        count: u64,
        head: u64,
        events: Vec<Event>,
    ) -> Self {
        Self {
            market: *market,
            event_queue: *event_queue,
            count,
            head,
            events,
        }
    }

    /// Loads the [`SerumEventQueueContext`].
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request
    /// or the [`Pubkey`]s given are not valid Serum Event Queue Accounts.
    pub async fn load(
        rpc_client: &Arc<RpcClient>,
        market: &Pubkey,
        event_queue: &Pubkey,
    ) -> Result<Self, ContextError> {
        let account_data = match rpc_client.get_account_data(event_queue).await {
            Ok(a) => a,
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };
        let data_words = remove_dex_account_padding(&account_data);
        let (header, seg0, seg1) = parse_dex_event_queue(&data_words);

        Ok(Self::new(
            market,
            event_queue,
            header.count(),
            header.head(),
            [seg0, seg1].concat(),
        ))
    }

    /// Loads the [`SerumEventQueueContext`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn from_account_data(
        market: &Pubkey,
        event_queue: &Pubkey,
        data: &[u8],
    ) -> Self {
        let data_words = remove_dex_account_padding(data);
        let (header, seg0, seg1) = parse_dex_event_queue(&data_words);

        Self::new(
            market,
            event_queue,
            header.count(),
            header.head(),
            [seg0, seg1].concat(),
        )
    }

    /// Loads the [`SerumEventQueueContext`] from the given [`AccountsCache`], if the given EventQueue's
    /// account state exists in the cache.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn from_cache(
        cache: Arc<AccountsCache>,
        market: &Pubkey,
        event_queue: &Pubkey,
    ) -> Result<Self, ContextError> {
        let eq_state = match cache.get(event_queue) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };
        let data_words = remove_dex_account_padding(eq_state.data.as_slice());
        let (header, seg0, seg1) = parse_dex_event_queue(&data_words);

        Ok(Self::new(
            market,
            event_queue,
            header.count(),
            header.head(),
            [seg0, seg1].concat(),
            // This appears to be more efficient than doing
            // seg0.into_ter().chain(seg1.into_iter()).collect::<Vec<Event>>()
        ))
    }

    /// Reloads the [`SerumEventQueueContext`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn reload_from_account_data(&mut self, data: &[u8]) {
        let data_words = remove_dex_account_padding(data);
        let (header, seg0, seg1) = parse_dex_event_queue(&data_words);

        self.count = header.count();
        self.head = header.head();
        self.events = [seg0, seg1].concat();
    }

    /// Reloads the [`SerumEventQueueContext`] from the given [`AccountsCache`],
    /// if the corresponding EventQueue's account state exists in the cache.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn reload_from_cache(
        &mut self,
        cache: Arc<AccountsCache>,
    ) -> Result<(), ContextError> {
        let eq_state = match cache.get(&self.event_queue) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };

        let data_words = remove_dex_account_padding(eq_state.data.as_slice());
        let (header, seg0, seg1) = parse_dex_event_queue(&data_words);

        self.count = header.count();
        self.head = header.head();
        self.events = [seg0, seg1].concat();

        Ok(())
    }
}
