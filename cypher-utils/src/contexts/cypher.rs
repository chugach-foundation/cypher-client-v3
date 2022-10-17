use cypher_client::{FuturesMarket, PerpetualMarket};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ContextError, MarketContext, PoolContext, SpotMarketContext};

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
    pub pools: RwLock<Vec<PoolContext>>,
    pub perp_markets: RwLock<Vec<MarketContext<PerpetualMarket>>>,
    pub futures_markets: RwLock<Vec<MarketContext<FuturesMarket>>>,
    pub spot_markets: RwLock<Vec<SpotMarketContext>>,
}

impl CypherContext {
    /// Creates a new [`CypherContext`].
    pub fn new(
        pools: Vec<PoolContext>,
        perp_markets: Vec<MarketContext<PerpetualMarket>>,
        futures_markets: Vec<MarketContext<FuturesMarket>>,
        spot_markets: Vec<SpotMarketContext>,
    ) -> Self {
        Self {
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
        let pools = match PoolContext::load_all(rpc_client).await {
            Ok(pools) => pools,
            Err(e) => {
                return Err(e);
            }
        };
        let spot_market_pubkeys = pools
            .iter()
            .map(|p| p.state.dex_market)
            .collect::<Vec<Pubkey>>();
        let spot_markets =
            match SpotMarketContext::load_many(rpc_client, &spot_market_pubkeys).await {
                Ok(markets) => markets,
                Err(e) => {
                    return Err(e);
                }
            };
        Ok(Self::new(pools, Vec::new(), Vec::new(), spot_markets))
    }

    /// Loads the [`CypherContext`] with [`MarketContext<PerpetualMarket>`]s only.
    ///
    /// ### Error
    ///
    /// This function will return an error if something goes wrong during the RPC request.
    pub async fn load_perpetual_markets(rpc_client: &Arc<RpcClient>) -> Result<Self, ContextError> {
        match MarketContext::<PerpetualMarket>::load_all(rpc_client).await {
            Ok(markets) => Ok(Self::new(Vec::new(), markets, Vec::new(), Vec::new())),
            Err(e) => Err(e),
        }
    }

    /// Loads the [`CypherContext`] with [`MarketContext<FuturesMarket>`]s only.
    ///
    /// ### Error
    ///
    /// This function will return an error if something goes wrong during the RPC request.
    pub async fn load_futures_markets(rpc_client: &Arc<RpcClient>) -> Result<Self, ContextError> {
        match MarketContext::<FuturesMarket>::load_all(rpc_client).await {
            Ok(markets) => Ok(Self::new(Vec::new(), Vec::new(), markets, Vec::new())),
            Err(e) => Err(e),
        }
    }
}
