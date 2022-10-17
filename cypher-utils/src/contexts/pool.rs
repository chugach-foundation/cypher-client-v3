use cypher_client::{utils::get_zero_copy_account, Pool};
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_filter::RpcFilterType};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

use crate::utils::{
    get_cypher_program_account, get_multiple_cypher_program_accounts, get_program_accounts,
};

use super::ContextError;

/// Represents a [Pool].
pub struct PoolContext {
    pub address: Pubkey,
    pub state: Pool,
}

impl PoolContext {
    /// Creates a new [`PoolContext`].
    pub fn new(address: &Pubkey, state: &Pool) -> Self {
        Self {
            address: *address,
            state: state.clone(),
        }
    }

    /// Loads the given [`Pool`].
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request
    /// or the Pool's [`Pubkey`] given is not a valid [`Pool`] Account.
    pub async fn load(rpc_client: &Arc<RpcClient>, pool: &Pubkey) -> Result<Self, ContextError> {
        match get_cypher_program_account::<Pool>(rpc_client, pool).await {
            Ok(s) => Ok(Self::new(pool, s.as_ref())),
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
        match get_multiple_cypher_program_accounts::<Pool>(&rpc_client, pools).await {
            Ok(s) => Ok(s
                .iter()
                .enumerate()
                .map(|(idx, state)| Self::new(&pools[idx], state.as_ref()))
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
                .map(|state| {
                    Self::new(
                        &state.0,
                        get_zero_copy_account::<Pool>(&state.1.data).as_ref(),
                    )
                })
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
        self.state = match get_cypher_program_account::<Pool>(rpc_client, &self.address).await {
            Ok(s) => s.as_ref().clone(),
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };
        Ok(())
    }
}
