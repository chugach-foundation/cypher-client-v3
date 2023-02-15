use cypher_client::{utils::get_zero_copy_account, Pool, PoolNode};
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_filter::RpcFilterType};
use solana_sdk::pubkey::Pubkey;
use std::{fmt::Debug, sync::Arc};

use crate::{
    accounts_cache::AccountsCache,
    utils::{
        get_cypher_zero_copy_account, get_multiple_cypher_zero_copy_accounts, get_program_accounts,
    },
};

use super::ContextError;

/// Represents a [PoolNode]
#[derive(Default, Clone)]
pub struct PoolNodeContext {
    pub address: Pubkey,
    pub state: Box<PoolNode>,
}

impl Debug for PoolNodeContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PoolNodeContext")
            .field("address", &format!("{}", self.address))
            .finish()
    }
}

impl PoolNodeContext {
    /// Creates a new [`PoolNodeContext`]
    pub fn new(address: &Pubkey, state: Box<PoolNode>) -> Self {
        Self {
            address: *address,
            state,
        }
    }

    /// Loads the [`PoolNode`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account data is invalid.
    pub fn from_account_data(account_data: &[u8], pool_node: &Pubkey) -> Self {
        let state = get_zero_copy_account::<PoolNode>(account_data);

        Self::new(pool_node, state)
    }

    /// Loads the given [`PoolNode`].
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request
    /// or the Pool's [`Pubkey`] given is not a valid [`PoolNode`] Account.
    pub async fn load(
        rpc_client: &Arc<RpcClient>,
        pool_node: &Pubkey,
    ) -> Result<Self, ContextError> {
        match get_cypher_zero_copy_account::<PoolNode>(rpc_client, pool_node).await {
            Ok(s) => Ok(Self::new(pool_node, s)),
            Err(e) => {
                Err(ContextError::ClientError(e))
            }
        }
    }

    /// Loads the given [`PoolNode`]s, if they exist.
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request,
    /// the [`Pubkey`] given is not a valid [`PoolNode`] Account or the underlying account does not
    /// have the correct Anchor discriminator for the provided type.
    pub async fn load_many(
        rpc_client: &Arc<RpcClient>,
        pool_nodes: &[Pubkey],
    ) -> Result<Vec<Self>, ContextError> {
        match get_multiple_cypher_zero_copy_accounts::<PoolNode>(rpc_client, pool_nodes).await {
            Ok(s) => Ok(s
                .iter()
                .enumerate()
                .map(|(idx, state)| Self::new(&pool_nodes[idx], state.clone()))
                .collect()),
            Err(e) => {
                Err(ContextError::ClientError(e))
            }
        }
    }

    /// Loads all [`PoolNode`], if they exist.
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request.
    pub async fn load_all(rpc_client: &Arc<RpcClient>) -> Result<Vec<Self>, ContextError> {
        let filters = vec![RpcFilterType::DataSize(
            std::mem::size_of::<PoolNode>() as u64 + 8,
        )];
        match get_program_accounts(rpc_client, filters, &cypher_client::id()).await {
            Ok(s) => Ok(s
                .iter()
                .map(|state| Self::new(&state.0, get_zero_copy_account::<PoolNode>(&state.1.data)))
                .collect()),
            Err(e) => {
                Err(ContextError::ClientError(e))
            }
        }
    }

    /// Reloads the [`PoolNode`]'s state.
    ///
    /// # Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request.
    pub async fn reload(&mut self, rpc_client: &Arc<RpcClient>) -> Result<(), ContextError> {
        self.state = match get_cypher_zero_copy_account::<PoolNode>(rpc_client, &self.address).await
        {
            Ok(s) => s,
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };
        Ok(())
    }

    /// Reloads the [`PoolNode`]'s state from the given account data.
    pub fn reload_from_account_data(&mut self, account_data: &[u8]) {
        self.state = get_zero_copy_account::<PoolNode>(account_data);
    }

    /// Reloads the [`PoolNode`] from the given [`AccountsCache`],
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

/// Represents a [Pool].
#[derive(Default, Clone)]
pub struct PoolContext {
    pub address: Pubkey,
    pub state: Box<Pool>,
    pub pool_nodes: Vec<PoolNodeContext>,
}

impl Debug for PoolContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PoolContext")
            .field("address", &format!("{}", self.address))
            .finish()
    }
}

impl PoolContext {
    /// Creates a new [`PoolContext`].
    pub fn new(address: &Pubkey, state: Box<Pool>, pool_nodes: Vec<PoolNodeContext>) -> Self {
        Self {
            address: *address,
            state,
            pool_nodes,
        }
    }

    /// Loads the [`Pool`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account data is invalid.
    pub fn from_account_data(account_data: &[u8], pool: &Pubkey) -> Self {
        let state = get_zero_copy_account::<Pool>(account_data);

        Self::new(pool, state, vec![])
    }

    /// Loads the given [`Pool`].
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request
    /// or the Pool's [`Pubkey`] given is not a valid [`Pool`] Account.
    pub async fn load(rpc_client: &Arc<RpcClient>, pool: &Pubkey) -> Result<Self, ContextError> {
        let pool_state = match get_cypher_zero_copy_account::<Pool>(rpc_client, pool).await {
            Ok(s) => s,
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };
        let nodes = pool_state
            .nodes
            .iter()
            .filter(|n| n.pool_node != Pubkey::default())
            .map(|n| n.pool_node)
            .collect::<Vec<_>>();
        let pool_nodes = match PoolNodeContext::load_many(rpc_client, &nodes).await {
            Ok(pns) => pns,
            Err(e) => {
                return Err(e);
            }
        };
        Ok(Self::new(pool, pool_state, pool_nodes))
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
        match get_multiple_cypher_zero_copy_accounts::<Pool>(rpc_client, pools).await {
            Ok(s) => Ok(s
                .iter()
                .enumerate()
                .map(|(idx, state)| Self::new(&pools[idx], state.clone(), vec![]))
                .collect()),
            Err(e) => {
                Err(ContextError::ClientError(e))
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
        let mut pools = match get_program_accounts(rpc_client, filters, &cypher_client::id()).await
        {
            Ok(s) => s
                .iter()
                .map(|state| {
                    Self::new(
                        &state.0,
                        get_zero_copy_account::<Pool>(&state.1.data),
                        vec![],
                    )
                })
                .collect::<Vec<PoolContext>>(),
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };

        for pool_ctx in pools.iter_mut() {
            let nodes = pool_ctx
                .state
                .nodes
                .iter()
                .filter(|n| n.pool_node != Pubkey::default())
                .map(|n| n.pool_node)
                .collect::<Vec<_>>();
            pool_ctx.pool_nodes = match PoolNodeContext::load_many(rpc_client, &nodes).await {
                Ok(pns) => pns,
                Err(e) => {
                    return Err(e);
                }
            };
        }
        Ok(pools)
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

    /// Reloads the [`Pool`]'s state from the given account data.
    pub fn reload_from_account_data(&mut self, account_data: &[u8]) {
        self.state = get_zero_copy_account::<Pool>(account_data);
    }

    /// Reloads the given [`PoolNode`] state from the given account data.
    pub fn reload_pool_node_from_account_data(&mut self, pool_node: &Pubkey, account_data: &[u8]) {
        if !self
            .pool_nodes
            .iter()
            .map(|pnc| pnc.address)
            .collect::<Vec<_>>()
            .contains(pool_node)
        {
            for n in self.state.nodes.iter() {
                if n.pool_node == *pool_node {
                    self.pool_nodes
                        .push(PoolNodeContext::from_account_data(account_data, pool_node));
                }
            }
        } else {
            for pn in self.pool_nodes.iter_mut() {
                if pn.address == *pool_node {
                    pn.reload_from_account_data(account_data);
                }
            }
        }
    }

    /// Reloads the [`Pool`] from the given [`AccountsCache`],
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
