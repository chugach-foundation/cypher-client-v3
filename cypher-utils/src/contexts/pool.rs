use cypher_client::{utils::get_zero_copy_account, Pool};
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_filter::RpcFilterType};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

use crate::{
    accounts_cache::AccountsCache,
    utils::{
        get_cypher_zero_copy_account, get_multiple_cypher_zero_copy_accounts, get_program_accounts,
    },
};

use super::ContextError;

/// Represents a [Pool].
#[derive(Default)]
pub struct PoolContext {
    pub address: Pubkey,
    pub state: Box<Pool>,
}

impl PoolContext {
    /// Creates a new [`PoolContext`].
    pub fn new(address: &Pubkey, state: Box<Pool>) -> Self {
        Self {
            address: *address,
            state,
        }
    }

    /// Loads the [`Pool`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account data is invalid.
    pub fn from_account_data(account_data: &[u8], market: &Pubkey) -> Result<Self, ContextError> {
        let state = get_zero_copy_account::<Pool>(account_data);

        Ok(Self::new(market, state))
    }

    /// Loads the given [`Pool`].
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request
    /// or the Pool's [`Pubkey`] given is not a valid [`Pool`] Account.
    pub async fn load(rpc_client: &Arc<RpcClient>, pool: &Pubkey) -> Result<Self, ContextError> {
        match get_cypher_zero_copy_account::<Pool>(rpc_client, pool).await {
            Ok(s) => Ok(Self::new(pool, s)),
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        }
    }

    /// Loads the given [`Pool`]s, if they exist.
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request,
    /// the [`Pubkey`] given is not a valid [`Pool`] Account or the underlying account does not
    /// have the correct Anchor discriminator for the provided type.
    pub async fn load_many(
        rpc_client: &Arc<RpcClient>,
        pools: &[Pubkey],
    ) -> Result<Vec<Self>, ContextError> {
        match get_multiple_cypher_zero_copy_accounts::<Pool>(&rpc_client, pools).await {
            Ok(s) => Ok(s
                .iter()
                .enumerate()
                .map(|(idx, state)| Self::new(&pools[idx], state.clone()))
                .collect()),
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        }
    }

    /// Loads all [`Pool`], if they exist.
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request.
    pub async fn load_all(rpc_client: &Arc<RpcClient>) -> Result<Vec<Self>, ContextError> {
        let filters = vec![RpcFilterType::DataSize(
            std::mem::size_of::<Pool>() as u64 + 8,
        )];
        match get_program_accounts(&rpc_client, filters, &cypher_client::id()).await {
            Ok(s) => Ok(s
                .iter()
                .map(|state| Self::new(&state.0, get_zero_copy_account::<Pool>(&state.1.data)))
                .collect()),
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        }
    }

    /// Reloads the [`Pool`]'s state.
    ///
    /// # Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request.
    pub async fn reload(&mut self, rpc_client: &Arc<RpcClient>) -> Result<(), ContextError> {
        self.state = match get_cypher_zero_copy_account::<Pool>(rpc_client, &self.address).await {
            Ok(s) => s,
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };
        Ok(())
    }

    /// Reloads the [`CacheContext`] from the given [`AccountsCache`],
    /// if the corresponding EventQueue's account state exists in the cache.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn reload_from_cache(&mut self, cache: Arc<AccountsCache>) -> Result<(), ContextError> {
        let cache_state = match cache.get(&self.address) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };

        self.state = get_zero_copy_account(&cache_state.data);

        Ok(())
    }
}
