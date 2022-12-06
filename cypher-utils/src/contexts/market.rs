use anchor_lang::{Owner, ZeroCopy};
use anchor_spl::dex::serum_dex::state::MarketState;
use bytemuck::bytes_of;
use cypher_client::{
    serum::parse_dex_account,
    utils::{derive_market_address, get_zero_copy_account},
};
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_filter::RpcFilterType};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

use crate::{
    accounts_cache::AccountsCache,
    utils::{
        encode_string, get_cypher_zero_copy_account, get_multiple_cypher_zero_copy_accounts,
        get_program_accounts,
    },
};

use super::ContextError;

/// A generic market context.
#[derive(Default)]
pub struct MarketContext<T> {
    pub address: Pubkey,
    pub state: Box<T>,
}

impl<T> MarketContext<T>
where
    T: ZeroCopy + Owner + Default,
{
    /// Creates a new [`MarketContext<T>`].
    pub fn new(market: &Pubkey, state: Box<T>) -> Self {
        Self {
            address: *market,
            state,
        }
    }

    /// Loads the [`T`] from the given [`AccountsCache`], if the given [`T`]'s
    /// account state exists in the cache and will spawn a task that will process updates on this state.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn from_cache(cache: Arc<AccountsCache>, market: &Pubkey) -> Result<Self, ContextError> {
        let account_state = match cache.get(market) {
            Some(a) => a,
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };

        let state = get_zero_copy_account::<T>(&account_state.data);

        Ok(Self::new(market, state))
    }

    /// Loads the [`T`] from the given account data.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account data is invalid.
    pub fn from_account_data(account_data: &[u8], market: &Pubkey) -> Result<Self, ContextError> {
        let state = get_zero_copy_account::<T>(account_data);

        Ok(Self::new(market, state))
    }

    /// Loads the [`T`] with the given name, if it exists.
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request
    /// or the Pool's [`Pubkey`] given is not a valid [`T`] Account.
    pub async fn load_with_name(
        rpc_client: &Arc<RpcClient>,
        market_name: &str,
    ) -> Result<Self, ContextError> {
        let market_name_bytes = encode_string(market_name);
        let (market_address, _) = derive_market_address(&market_name_bytes);

        MarketContext::load(rpc_client, &market_address).await
    }

    /// Loads the given [`T`], if it exists.
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request,
    /// the [`Pubkey`] given is not a valid [`T`] Account or the underlying account does not
    /// have the correct Anchor discriminator for the provided type.
    pub async fn load(rpc_client: &Arc<RpcClient>, market: &Pubkey) -> Result<Self, ContextError> {
        match get_cypher_zero_copy_account::<T>(&rpc_client, market).await {
            Ok(s) => Ok(Self::new(market, s)),
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        }
    }

    /// Loads the given [`T`]s, if they exist.
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request,
    /// the [`Pubkey`] given is not a valid [`T`] Account or the underlying account does not
    /// have the correct Anchor discriminator for the provided type.
    pub async fn load_many(
        rpc_client: &Arc<RpcClient>,
        markets: &[Pubkey],
    ) -> Result<Vec<Self>, ContextError> {
        match get_multiple_cypher_zero_copy_accounts::<T>(&rpc_client, markets).await {
            Ok(s) => Ok(s
                .iter()
                .enumerate()
                .map(|(idx, state)| Self::new(&markets[idx], state.clone()))
                .collect()),
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        }
    }

    /// Loads all [`T`]s, if they exist.
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request.
    pub async fn load_all(rpc_client: &Arc<RpcClient>) -> Result<Vec<Self>, ContextError> {
        let filters = vec![RpcFilterType::DataSize(std::mem::size_of::<T>() as u64 + 8)];
        match get_program_accounts(&rpc_client, filters, &cypher_client::id()).await {
            Ok(s) => Ok(s
                .iter()
                .map(|state| Self::new(&state.0, get_zero_copy_account::<T>(&state.1.data)))
                .collect()),
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        }
    }

    /// Reloads the [`T`]'s state.
    ///
    /// # Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request.
    pub async fn reload(&mut self, rpc_client: &Arc<RpcClient>) -> Result<(), ContextError> {
        let state_res = get_cypher_zero_copy_account::<T>(&rpc_client, &self.address).await;
        self.state = match state_res {
            Ok(s) => s.clone(),
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

        self.state = get_zero_copy_account::<T>(&cache_state.data);

        Ok(())
    }
}

/// Represents a Serum Market
pub struct SpotMarketContext {
    pub address: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub event_queue: Pubkey,
    pub base_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_mint: Pubkey,
    pub quote_vault: Pubkey,

    pub state: MarketState,
}

impl SpotMarketContext {
    /// Creates a new [`SpotMarketContext`].
    pub fn new(
        address: &Pubkey,
        bids: &Pubkey,
        asks: &Pubkey,
        event_queue: &Pubkey,
        base_mint: &Pubkey,
        base_vault: &Pubkey,
        quote_mint: &Pubkey,
        quote_vault: &Pubkey,
        state: MarketState,
    ) -> Self {
        Self {
            address: *address,
            bids: *bids,
            asks: *asks,
            event_queue: *event_queue,
            base_mint: *base_mint,
            base_vault: *base_vault,
            quote_mint: *quote_mint,
            quote_vault: *quote_vault,
            state,
        }
    }

    /// Loads the Market from the given [`AccountsCache`], if the given Market's
    /// account state exists in the cache and will spawn a task that will process updates on this state.
    ///
    /// ### Errors
    ///
    /// This function will return an error if the account state does not exist in the cache.
    pub fn from_cache(cache: Arc<AccountsCache>, market: &Pubkey) -> Result<Self, ContextError> {
        let state: MarketState = match cache.get(market) {
            Some(a) => parse_dex_account::<MarketState>(&a.data),
            None => {
                return Err(ContextError::MissingAccountState);
            }
        };

        // copying the field contents to local variables to avoid
        // warnings due to unaligned references
        // see issue #82523 <https://github.com/rust-lang/rust/issues/82523
        let bids = state.bids;
        let asks = state.asks;
        let event_q = state.event_q;
        let coin_mint = state.coin_mint;
        let coin_vault = state.coin_vault;
        let pc_mint = state.pc_mint;
        let pc_vault = state.pc_vault;

        Ok(Self::new(
            market,
            &Pubkey::new(bytes_of(&bids)),
            &Pubkey::new(bytes_of(&asks)),
            &Pubkey::new(bytes_of(&event_q)),
            &Pubkey::new(bytes_of(&coin_mint)),
            &Pubkey::new(bytes_of(&coin_vault)),
            &Pubkey::new(bytes_of(&pc_mint)),
            &Pubkey::new(bytes_of(&pc_vault)),
            state,
        ))
    }

    /// Loads the given [`MarketState`], if it exists.
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request,
    /// the [`Pubkey`] given is not a valid [`MarketState`] Account.
    pub async fn load(rpc_client: &Arc<RpcClient>, market: &Pubkey) -> Result<Self, ContextError> {
        let state = match rpc_client.get_account_data(market).await {
            Ok(a) => parse_dex_account::<MarketState>(&a),
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };

        // copying the field contents to local variables to avoid
        // warnings due to unaligned references
        // see issue #82523 <https://github.com/rust-lang/rust/issues/82523
        let bids = state.bids;
        let asks = state.asks;
        let event_q = state.event_q;
        let coin_mint = state.coin_mint;
        let coin_vault = state.coin_vault;
        let pc_mint = state.pc_mint;
        let pc_vault = state.pc_vault;

        Ok(Self::new(
            market,
            &Pubkey::new(bytes_of(&bids)),
            &Pubkey::new(bytes_of(&asks)),
            &Pubkey::new(bytes_of(&event_q)),
            &Pubkey::new(bytes_of(&coin_mint)),
            &Pubkey::new(bytes_of(&coin_vault)),
            &Pubkey::new(bytes_of(&pc_mint)),
            &Pubkey::new(bytes_of(&pc_vault)),
            state,
        ))
    }

    /// Loads the given [`MarketState`]s, if they exist.
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC request,
    /// the [`Pubkey`] given is not a valid [`MarketState`] Account or the underlying account does not
    /// have the correct Anchor discriminator for the provided type.
    pub async fn load_many(
        rpc_client: &Arc<RpcClient>,
        markets: &[Pubkey],
    ) -> Result<Vec<Self>, ContextError> {
        match rpc_client.get_multiple_accounts(markets).await {
            Ok(s) => Ok(s
                .iter()
                .enumerate()
                .filter(|a| a.1.is_some())
                .map(|(idx, state)| {
                    let state = parse_dex_account::<MarketState>(&state.as_ref().unwrap().data);
                    // copying the field contents to local variables to avoid
                    // warnings due to unaligned references
                    // see issue #82523 <https://github.com/rust-lang/rust/issues/82523
                    let bids = state.bids;
                    let asks = state.asks;
                    let event_q = state.event_q;
                    let coin_mint = state.coin_mint;
                    let coin_vault = state.coin_vault;
                    let pc_mint = state.pc_mint;
                    let pc_vault = state.pc_vault;

                    Self::new(
                        &markets[idx],
                        &Pubkey::new(bytes_of(&bids)),
                        &Pubkey::new(bytes_of(&asks)),
                        &Pubkey::new(bytes_of(&event_q)),
                        &Pubkey::new(bytes_of(&coin_mint)),
                        &Pubkey::new(bytes_of(&coin_vault)),
                        &Pubkey::new(bytes_of(&pc_mint)),
                        &Pubkey::new(bytes_of(&pc_vault)),
                        state,
                    )
                })
                .collect()),
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        }
    }
}
