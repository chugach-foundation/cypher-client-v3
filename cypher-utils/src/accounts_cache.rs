use {
    dashmap::{mapref::one::Ref, DashMap},
    log::{info, warn},
    solana_sdk::pubkey::Pubkey,
    tokio::sync::broadcast::{channel, Receiver, Sender},
};

#[derive(Debug, PartialEq)]
pub enum AccountsCacheError {
    ChannelSendError,
}

/// An Account cache which allows subscribing to cache updates.
pub struct AccountsCache {
    map: DashMap<Pubkey, AccountState>,
    sender: Sender<AccountState>,
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
    pub fn default() -> Self {
        Self {
            map: DashMap::default(),
            sender: channel::<AccountState>(u16::MAX as usize).0,
        }
    }

    /// Creates a new [`AccountsCache`].
    pub fn new() -> Self {
        AccountsCache {
            map: DashMap::new(),
            sender: channel::<AccountState>(u16::MAX as usize).0,
        }
    }

    /// Gets a [`Receiver`] handle that will receive cache updates after the call to `subscribe`.
    pub fn subscribe(&self) -> Receiver<AccountState> {
        self.sender.subscribe()
    }

    /// Get the Account state associated with the given pubkey.
    pub fn get(&self, key: &Pubkey) -> Option<Ref<'_, Pubkey, AccountState>> {
        self.map.get(key)
    }

    /// Updates the Account state associated with the given pubkey.
    pub fn insert(&self, key: Pubkey, data: AccountState) {
        // get the previous state and compare the slot
        // if the previous state has an higher slot, discard this insert altogether
        let maybe_state = self.get(&key);
        if maybe_state.is_some() {
            let state = maybe_state.unwrap();
            if state.slot > data.slot {
                info!("[CACHE] Attempted to update key: {} with older data!", key);
                return;
            }
        }
        match self.sender.send(data.clone()) {
            Ok(r) => {
                info!("[CACHE] Sent updated Account state to {} recivers.", r);
            }
            Err(_) => {
                warn!(
                    "[CACHE] Failed to send message about updated Account {}",
                    key.to_string()
                );
            }
        }

        self.map.insert(key, data);
    }
}
