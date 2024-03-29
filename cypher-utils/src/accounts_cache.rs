use log::debug;

use {
    dashmap::{mapref::one::Ref, DashMap},
    log::{info, warn},
    solana_sdk::pubkey::Pubkey,
    std::sync::Arc,
    tokio::sync::{
        broadcast::{channel, Receiver, Sender},
        RwLock,
    },
};

#[derive(Debug, PartialEq)]
pub enum AccountsCacheError {
    ChannelSendError,
}

pub struct SubscriptionMap {
    accounts: Vec<Pubkey>,
    sender: Arc<Sender<AccountState>>,
}

/// An Account cache which allows subscribing to cache updates.
pub struct AccountsCache {
    map: DashMap<Pubkey, AccountState>,
    subscriptions: RwLock<Vec<SubscriptionMap>>,
    sender: Arc<Sender<AccountState>>,
}

impl Default for AccountsCache {
    fn default() -> Self {
        Self {
            map: DashMap::default(),
            subscriptions: RwLock::new(Vec::new()),
            sender: Arc::new(channel::<AccountState>(u16::MAX as usize).0),
        }
    }
}

/// Represent's an on-chain Account's state at a given slot.
#[derive(Debug, Clone)]
pub struct AccountState {
    /// The Account pubkey.
    pub account: Pubkey,
    /// The Account data.
    pub data: Vec<u8>,
    /// The slot at which this Account's data was seen.
    pub slot: u64,
}

impl AccountsCache {
    /// Creates a new [`AccountsCache`].
    pub fn new() -> Self {
        AccountsCache {
            map: DashMap::new(),
            subscriptions: RwLock::new(Vec::new()),
            sender: Arc::new(channel::<AccountState>(u16::MAX as usize).0),
        }
    }

    /// Gets a [`Receiver`] handle that will receive cache updates after the call to `subscribe`.
    pub fn subscribe_all(&self) -> Receiver<AccountState> {
        self.sender.subscribe()
    }

    /// Gets a [`Receiver`] handle that will receive cache updates after the call to `subscribe`.
    pub async fn subscribe(&self, accounts: &[Pubkey]) -> Receiver<AccountState> {
        let mut subscriptions = self.subscriptions.write().await;
        let sender = Arc::new(channel::<AccountState>(u16::MAX as usize).0);
        subscriptions.push(SubscriptionMap {
            accounts: accounts.to_vec(),
            sender: sender.clone(),
        });
        sender.subscribe()
    }

    /// Get the Account state associated with the given pubkey.
    pub fn get(&self, key: &Pubkey) -> Option<Ref<'_, Pubkey, AccountState>> {
        self.map.get(key)
    }

    /// Updates the Account state associated with the given pubkey.
    pub async fn insert(&self, key: Pubkey, data: AccountState) {
        // get the previous state and compare the slot
        // if the previous state has an higher slot, discard this insert altogether
        let maybe_state = self.get(&key);
        if maybe_state.is_some() {
            let state = maybe_state.unwrap();
            if state.slot > data.slot {
                debug!("Attempted to update key: {} with older data!", key);
                return;
            }
        }
        let subscriptions = self.subscriptions.read().await;
        for sub in subscriptions.iter() {
            if sub.accounts.contains(&key) {
                match sub.sender.send(data.clone()) {
                    Ok(r) => {
                        debug!("Sent updated Account state to {} receivers.", r);
                    }
                    Err(_) => {
                        debug!(
                            "Failed to send message about updated Account {}",
                            key.to_string()
                        );
                    }
                }
            }
        }
        match self.sender.send(data.clone()) {
            Ok(r) => {
                debug!("Sent updated Account state to {} receivers.", r);
            }
            Err(_) => {
                debug!(
                    "Failed to send message about updated Account {}",
                    key.to_string()
                );
            }
        }
        self.map.insert(key, data);
    }
}
