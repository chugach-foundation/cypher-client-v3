use cypher_client::{FuturesMarket, PerpetualMarket};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::accounts_cache::AccountsCache;

use super::{CacheContext, ContextError, MarketContext, PoolContext, SpotMarketContext};

/// Represents the Cypher ecosystem.
///
/// This structure is capable of loading all Pools, Perpetual Markets, Futures Markets,
/// Serum Markets and Cypher Accounts & Sub Accounts.
///
/// Due to the sensitive and heavy nature of these methods, they should be used carefully.
///
/// Consider loading all of the Pools and Markets once and then using the [`PubsubClient`]
/// or even the [`StreamingAccountInfoService`] to subscribe to these accounts instead of polling.
pub struct CypherContext {
    pub cache: RwLock<CacheContext>,
    pub pools: RwLock<Vec<PoolContext>>,
    pub perp_markets: RwLock<Vec<MarketContext<PerpetualMarket>>>,
    pub futures_markets: RwLock<Vec<MarketContext<FuturesMarket>>>,
    pub spot_markets: RwLock<Vec<SpotMarketContext>>,
}

impl CypherContext {
    /// Creates a new [`CypherContext`].
    pub fn new(
        cache: CacheContext,
        pools: Vec<PoolContext>,
        perp_markets: Vec<MarketContext<PerpetualMarket>>,
        futures_markets: Vec<MarketContext<FuturesMarket>>,
        spot_markets: Vec<SpotMarketContext>,
    ) -> Self {
        Self {
            cache: RwLock::new(cache),
            pools: RwLock::new(pools),
            perp_markets: RwLock::new(perp_markets),
            futures_markets: RwLock::new(futures_markets),
            spot_markets: RwLock::new(spot_markets),
        }
    }

    /// Loads the [`CypherContext`] with all of the [`PoolContext`]s, [`MarketContext<PerpetualMarket>`]s, [`MarketContext<FuturesMarket>`]s and [`SpotMarketContext`]s.
    ///
    /// ### Error
    ///
    /// This function will return an error if something goes wrong during the RPC request.
    pub async fn load(rpc_client: &Arc<RpcClient>) -> Result<Self, ContextError> {
        let cache = match CacheContext::load(rpc_client).await {
            Ok(c) => c,
            Err(e) => {
                return Err(e);
            }
        };

        let pools = match PoolContext::load_all(rpc_client).await {
            Ok(pools) => pools,
            Err(e) => {
                return Err(e);
            }
        };
        let futures_markets = match MarketContext::<FuturesMarket>::load_all(rpc_client).await {
            Ok(markets) => markets,
            Err(e) => {
                return Err(e);
            }
        };
        let perpetual_markets = match MarketContext::<PerpetualMarket>::load_all(rpc_client).await {
            Ok(markets) => markets,
            Err(e) => {
                return Err(e);
            }
        };
        let spot_market_pubkeys = pools
            .iter()
            .filter(|p| p.state.dex_market != Pubkey::default())
            .map(|p| p.state.dex_market)
            .collect::<Vec<Pubkey>>();
        let spot_markets =
            match SpotMarketContext::load_many(rpc_client, &spot_market_pubkeys).await {
                Ok(markets) => markets,
                Err(e) => {
                    return Err(e);
                }
            };
        Ok(Self::new(
            cache,
            pools,
            perpetual_markets,
            futures_markets,
            spot_markets,
        ))
    }

    /// Loads the [`CypherContext`] with [`PoolContext`]s and [`SpotMarketContext`]s only.
    ///
    /// ### Error
    ///
    /// This function will return an error if something goes wrong during the RPC request.
    pub async fn load_pools(rpc_client: &Arc<RpcClient>) -> Result<Self, ContextError> {
        let cache = match CacheContext::load(rpc_client).await {
            Ok(c) => c,
            Err(e) => {
                return Err(e);
            }
        };

        let pools = match PoolContext::load_all(rpc_client).await {
            Ok(pools) => pools,
            Err(e) => {
                return Err(e);
            }
        };
        let spot_market_pubkeys = pools
            .iter()
            .filter(|p| p.state.dex_market != Pubkey::default())
            .map(|p| p.state.dex_market)
            .collect::<Vec<Pubkey>>();
        let spot_markets =
            match SpotMarketContext::load_many(rpc_client, &spot_market_pubkeys).await {
                Ok(markets) => markets,
                Err(e) => {
                    return Err(e);
                }
            };
        Ok(Self::new(
            cache,
            pools,
            Vec::new(),
            Vec::new(),
            spot_markets,
        ))
    }

    /// Loads the [`CypherContext`] with [`MarketContext<PerpetualMarket>`]s only.
    ///
    /// ### Error
    ///
    /// This function will return an error if something goes wrong during the RPC request.
    pub async fn load_perpetual_markets(rpc_client: &Arc<RpcClient>) -> Result<Self, ContextError> {
        let cache = match CacheContext::load(rpc_client).await {
            Ok(c) => c,
            Err(e) => {
                return Err(e);
            }
        };
        match MarketContext::<PerpetualMarket>::load_all(rpc_client).await {
            Ok(markets) => Ok(Self::new(
                cache,
                Vec::new(),
                markets,
                Vec::new(),
                Vec::new(),
            )),
            Err(e) => Err(e),
        }
    }

    /// Loads the [`CypherContext`] with [`MarketContext<FuturesMarket>`]s only.
    ///
    /// ### Error
    ///
    /// This function will return an error if something goes wrong during the RPC request.
    pub async fn load_futures_markets(rpc_client: &Arc<RpcClient>) -> Result<Self, ContextError> {
        let cache = match CacheContext::load(rpc_client).await {
            Ok(c) => c,
            Err(e) => {
                return Err(e);
            }
        };
        match MarketContext::<FuturesMarket>::load_all(rpc_client).await {
            Ok(markets) => Ok(Self::new(
                cache,
                Vec::new(),
                Vec::new(),
                markets,
                Vec::new(),
            )),
            Err(e) => Err(e),
        }
    }

    /// Reloads the [`CypherContext`] from an [`AccountsCache`].
    pub async fn reload(&mut self, cache: Arc<AccountsCache>) {
        let mut cache_guard = self.cache.write().await;
        match cache_guard.reload_from_cache(cache.clone()) {
            Ok(()) => (),
            Err(_) => (),
        };

        let mut pools = self.pools.write().await;
        for pool in pools.iter_mut() {
            match pool.reload_from_cache(cache.clone()) {
                Ok(()) => (),
                Err(_) => (),
            };
        }

        let mut perp_markets = self.perp_markets.write().await;
        for perp_market in perp_markets.iter_mut() {
            match perp_market.reload_from_cache(cache.clone()) {
                Ok(()) => (),
                Err(_) => (),
            };
        }

        let mut futures_markets = self.futures_markets.write().await;
        for futures_market in futures_markets.iter_mut() {
            match futures_market.reload_from_cache(cache.clone()) {
                Ok(()) => (),
                Err(_) => (),
            };
        }
    }
}
