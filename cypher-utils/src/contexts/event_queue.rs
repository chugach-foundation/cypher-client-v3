use agnostic_orderbook::state::event_queue::FillEvent;
use anchor_spl::dex::serum_dex::state::{Event, QueueHeader};
use cypher_client::{
    aob::{parse_aob_event_queue, CallBackInfo},
    serum::{parse_dex_event_queue, remove_dex_account_padding},
};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::accounts_cache::AccountsCache;

use super::ContextError;

/// Represents an AOB Event Queue.
pub struct AgnosticEventQueueContext {
    pub market: Pubkey,
    pub event_queue: Pubkey,
    pub count: RwLock<u64>,
    pub head: RwLock<u64>,
    pub events: RwLock<Vec<FillEvent>>,
    pub callbacks: RwLock<Vec<CallBackInfo>>,
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
            count: RwLock::new(count),
            head: RwLock::new(head),
            events: RwLock::new(events),
            callbacks: RwLock::new(callbacks),
        }
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
    ) -> Result<Self, ContextError> {
        let (eq_header, fills, callbacks) = parse_aob_event_queue(&data);

        Ok(Self::new(
            market,
            event_queue,
            eq_header.count,
            eq_header.head,
            fills.to_vec(),
            callbacks.to_vec(),
        ))
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
    pub async fn reload_from_account_data(&mut self, data: &[u8]) {
        let (eq_header, new_fills, new_callbacks) = parse_aob_event_queue(&data);

        let mut count = self.count.write().await;
        *count = eq_header.count;
        let mut head = self.head.write().await;
        *head = eq_header.head;
        let mut callbacks = self.callbacks.write().await;
        *callbacks = new_callbacks.to_vec();
        let mut events = self.events.write().await;
        *events = new_fills.to_vec();
    }

    /// Reloads the [`AgnosticEventQueueContext`] from the given [`AccountsCache`],
    /// if the corresponding EventQueue's account state exists in the cache.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub async fn reload_from_cache(
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

        let mut count = self.count.write().await;
        *count = eq_header.count;
        let mut head = self.head.write().await;
        *head = eq_header.head;
        let mut callbacks = self.callbacks.write().await;
        *callbacks = new_callbacks.to_vec();
        let mut events = self.events.write().await;
        *events = new_fills.to_vec();

        Ok(())
    }
}

/// Represents a Serum Event Queue.
pub struct SerumEventQueueContext {
    pub market: Pubkey,
    pub event_queue: Pubkey,
    pub count: RwLock<u64>,
    pub head: RwLock<u64>,
    pub events: RwLock<Vec<Event>>,
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
            count: RwLock::new(count),
            head: RwLock::new(head),
            events: RwLock::new(events),
        }
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
    ) -> Result<Self, ContextError> {
        let data_words = remove_dex_account_padding(data);
        let (header, seg0, seg1) = parse_dex_event_queue(&data_words);

        Ok(Self::new(
            market,
            event_queue,
            header.count(),
            header.head(),
            [seg0, seg1].concat(),
        ))
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
    pub async fn reload_from_account_data(&mut self, data: &[u8]) {
        let data_words = remove_dex_account_padding(data);
        let (header, seg0, seg1) = parse_dex_event_queue(&data_words);

        let mut count = self.count.write().await;
        *count = header.count();
        let mut head = self.head.write().await;
        *head = header.head();
        let mut events = self.events.write().await;
        *events = [seg0, seg1].concat();
    }

    /// Reloads the [`SerumEventQueueContext`] from the given [`AccountsCache`],
    /// if the corresponding EventQueue's account state exists in the cache.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub async fn reload_from_cache(
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

        let mut count = self.count.write().await;
        *count = header.count();
        let mut head = self.head.write().await;
        *head = header.head();
        let mut events = self.events.write().await;
        *events = [seg0, seg1].concat();

        Ok(())
    }
}
