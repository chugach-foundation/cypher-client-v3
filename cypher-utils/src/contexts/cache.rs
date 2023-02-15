use cypher_client::{cache_account, utils::get_zero_copy_account, Cache, CacheAccount};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::{fmt::Debug, sync::Arc};

use crate::{accounts_cache::AccountsCache, utils::get_cypher_zero_copy_account};

use super::ContextError;

#[derive(Clone)]
pub struct CacheContext {
    pub state: Box<CacheAccount>,
}

impl Debug for CacheContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheContext")
            .field("address", &format!("{}", cache_account::id()))
            .finish()
    }
}

impl Default for CacheContext {
    fn default() -> Self {
        Self {
            state: Box::new(CacheAccount {
                authority: Pubkey::default(),
                caches: [Cache::default(); 512],
            }),
        }
    }
}

impl CacheContext {
    /// Creates a new [`CacheContext`].
    pub fn new(cache: Box<CacheAccount>) -> Self {
        Self { state: cache }
    }

    /// Loads the cache account.
    pub async fn load(rpc_client: &Arc<RpcClient>) -> Result<Self, ContextError> {
        match get_cypher_zero_copy_account::<CacheAccount>(rpc_client, &cache_account::id()).await {
            Ok(s) => Ok(Self::new(s)),
            Err(e) => Err(ContextError::ClientError(e)),
        }
    }

    /// Reloads the [`CacheContext`] from the given [`AccountsCache`],
    /// if the corresponding EventQueue's account state exists in the cache.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn reload_from_cache(&mut self, cache: Arc<AccountsCache>) -> Result<(), ContextError> {
        let cache_state = match cache.get(&cache_account::id()) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };

        self.state = get_zero_copy_account(&cache_state.data);

        Ok(())
    }

    pub fn reload_from_account_data(&mut self, account_data: &[u8]) {
        self.state = get_zero_copy_account(account_data);
    }
}
