use {
    crate::constants::{JSON_RPC_URL, PUBSUB_RPC_URL},
    futures::StreamExt,
    log::{info, warn},
    solana_client::{
        client_error::ClientError,
        nonblocking::{
            pubsub_client::{PubsubClient, PubsubClientError},
            rpc_client::RpcClient,
        },
        rpc_response::RpcPrioritizationFee,
    },
    solana_sdk::commitment_config::CommitmentConfig,
    solana_sdk::{hash::Hash, pubkey::Pubkey},
    std::sync::Arc,
    thiserror::Error,
    tokio::sync::broadcast::{channel, Receiver, Sender},
    tokio::{sync::RwLock, time::Duration},
};

#[derive(Debug, Error)]
pub enum ChainMetaServiceError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(transparent)]
    PubsubClientError(#[from] PubsubClientError),
}

/// A map between accounts which are write-locked and their respective [`RpcPrioritizationFee`].
pub struct WriteLockedAccountsMap {
    /// An alias for this group of accounts.
    alias: String,
    /// The accounts which are write-locked.
    accounts: Vec<Pubkey>,
    /// The recent priority fees.
    recent_priority_fees: RwLock<Vec<RpcPrioritizationFee>>,
}

/// A service which asynchronously polls the given [`RpcClient`] for a recent block [`Hash`] and [`RpcPrioritizationFee`]s,
/// as well as subscribes to the given [`PubsubClient`] to be notified of new slots.
///
/// For more efficient usage of priority fees, the service allows you to specify different groups of accounts which require write-locking,
/// for each of these specified groups of accounts it will request the recent priority fees.
pub struct ChainMetaService {
    pub rpc_client: Arc<RpcClient>,
    pub pubsub_client: Arc<PubsubClient>,
    pub accounts_map: RwLock<Vec<WriteLockedAccountsMap>>,
    recent_blockhash: RwLock<Hash>,
    latest_slot: RwLock<u64>,
    recent_priority_fees: RwLock<Vec<RpcPrioritizationFee>>,
    shutdown: RwLock<Receiver<bool>>,
    inner_shutdown: Arc<Sender<bool>>,
    subscribe_slot: bool,
    fetch_priority_fees: bool,
}

impl ChainMetaService {
    pub async fn default() -> Self {
        Self {
            rpc_client: Arc::new(RpcClient::new(JSON_RPC_URL.to_string())),
            pubsub_client: Arc::new(PubsubClient::new(PUBSUB_RPC_URL).await.unwrap()),
            accounts_map: RwLock::new(Vec::new()),
            recent_blockhash: RwLock::new(Hash::default()),
            latest_slot: RwLock::new(u64::default()),
            recent_priority_fees: RwLock::new(Vec::new()),
            shutdown: RwLock::new(channel::<bool>(1).1),
            inner_shutdown: Arc::new(channel::<bool>(1).0),
            subscribe_slot: false,
            fetch_priority_fees: false,
        }
    }

    pub fn new(
        rpc_client: Arc<RpcClient>,
        pubsub_client: Arc<PubsubClient>,
        shutdown_receiver: Receiver<bool>,
        subscribe_slot: bool,
        fetch_priority_fees: bool,
    ) -> ChainMetaService {
        ChainMetaService {
            rpc_client,
            pubsub_client,
            subscribe_slot,
            fetch_priority_fees,
            shutdown: RwLock::new(shutdown_receiver),
            accounts_map: RwLock::new(Vec::new()),
            recent_blockhash: RwLock::new(Hash::default()),
            latest_slot: RwLock::new(u64::default()),
            recent_priority_fees: RwLock::new(Vec::new()),
            inner_shutdown: Arc::new(channel::<bool>(1).0),
        }
    }

    #[inline(always)]
    pub async fn start_service(self: &Arc<Self>) -> Result<(), ChainMetaServiceError> {
        if self.subscribe_slot {
            info!("Starting service with slot subscription.");
            self.start_service_with_slot_subscription().await
        } else {
            info!("Starting service without slot subscription.");
            self.start_service_without_slot_subscription().await
        }
    }

    async fn start_service_without_slot_subscription(
        self: &Arc<Self>,
    ) -> Result<(), ChainMetaServiceError> {
        let mut shutdown = self.shutdown.write().await;

        let aself = self.clone();
        let polling_handle = tokio::spawn(async move {
            aself.update_chain_meta_replay().await;
        });

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    info!("Received shutdown signal, stopping tasks.");
                    match self.inner_shutdown.send(true) {
                        Ok(_) => {
                            info!("Successfully sent stop signal to internal tasks.");
                        }
                        Err(e) => {
                            warn!("Failed to send stop signal to internal tasks: {}", e.to_string());
                        }
                    };
                    break;
                }
            }
        }

        match tokio::join!(polling_handle) {
            (Ok(()),) => {
                info!("Successfully stopped polling task.");
            }
            (Err(e),) => {
                warn!("Failed to join with task: {}", e.to_string());
            }
        }

        Ok(())
    }

    #[inline(always)]
    async fn start_service_with_slot_subscription(
        self: &Arc<Self>,
    ) -> Result<(), ChainMetaServiceError> {
        let mut shutdown = self.shutdown.write().await;
        let mut stream = match self.pubsub_client.slot_subscribe().await {
            Ok(res) => res.0,
            Err(e) => {
                warn!(
                    "Failed to fetch recent prioritization fees: {}",
                    e.to_string()
                );
                return Err(ChainMetaServiceError::PubsubClientError(e));
            }
        };

        let aself = self.clone();
        let polling_handle = tokio::spawn(async move {
            aself.update_chain_meta_replay().await;
        });

        loop {
            tokio::select! {
                update = stream.next() => {
                    match update {
                        Some(slot_info) => {
                            info!("Received latest slot update: {}", slot_info.slot);
                            *self.latest_slot.write().await = slot_info.slot;
                        }
                        None => {
                            warn!("Something went wrong while receiving slot info update.");
                        }
                    }
                },
                _ = shutdown.recv() => {
                    info!("Received shutdown signal, stopping tasks.");
                    match self.inner_shutdown.send(true) {
                        Ok(_) => {
                            info!("Successfully sent stop signal to internal tasks.");
                        }
                        Err(e) => {
                            warn!("Failed to send stop signal to internal tasks: {}", e.to_string());
                        }
                    };
                    break;
                }
            }
        }

        match tokio::join!(polling_handle) {
            (Ok(()),) => {
                info!("Successfully stopped polling task.");
            }
            (Err(e),) => {
                warn!("Failed to join with task: {}", e.to_string());
            }
        }

        Ok(())
    }

    #[inline(always)]
    pub async fn add_priority_fees_accounts(self: &Arc<Self>, alias: &str, accounts: &[Pubkey]) {
        let mut accounts_map = self.accounts_map.write().await;
        accounts_map.push(WriteLockedAccountsMap {
            alias: alias.to_string(),
            accounts: accounts.to_vec(),
            recent_priority_fees: RwLock::new(Vec::new()),
        });
    }

    #[inline(always)]
    async fn update_chain_meta_replay(self: Arc<Self>) {
        let mut interval = tokio::time::interval(Duration::from_millis(5000));
        let mut shutdown = self.inner_shutdown.subscribe();
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    match self.update_chain_meta().await {
                        Ok(()) => (),
                        Err(e) => {
                            warn!("Failed to get new chain meta: {}", e.to_string());
                        }
                    }
                }
                _ = shutdown.recv() => {
                    info!("Received shutdown signal, stopping.");
                    break;
                }
            }
        }
    }

    #[inline(always)]
    async fn update_chain_meta(self: &Arc<Self>) -> Result<(), ClientError> {
        let hash = match self
            .rpc_client
            .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
            .await
        {
            Ok(h) => h.0,
            Err(e) => {
                warn!("Failed to fetch recent block hash: {}", e.to_string());
                return Err(e);
            }
        };
        info!(
            "Successfully fetched recent block hash: {}",
            hash.to_string()
        );
        *self.recent_blockhash.write().await = hash;

        if self.fetch_priority_fees {
            let mut accounts_map = self.accounts_map.write().await;
            if !accounts_map.is_empty() {
                for map in accounts_map.iter_mut() {
                    let mut fees_res = match self
                        .get_recent_prioritization_fees(Some(&map.accounts))
                        .await
                    {
                        Ok(rpf) => rpf,
                        Err(e) => {
                            warn!(
                                "Failed to fetch recent prioritization fees: {}",
                                e.to_string()
                            );
                            return Err(e);
                        }
                    };
                    fees_res.sort_by_key(|f| f.slot);
                    fees_res.reverse();
                    let sum_fees: u64 = fees_res
                        .iter()
                        .take(10)
                        .map(|f| f.prioritization_fee)
                        .collect::<Vec<u64>>()
                        .iter()
                        .sum();
                    info!(
                    "Successfully fetched prioritization fees for accounts: {}. Average 5s Rolling Fee: {}",
                    map.alias,
                    sum_fees / 10
                );
                    *map.recent_priority_fees.write().await = fees_res;
                }
            }
            let mut fees_res = match self.get_recent_prioritization_fees(None).await {
                Ok(rpf) => rpf,
                Err(e) => {
                    warn!(
                        "Failed to fetch recent prioritization fees: {}",
                        e.to_string()
                    );
                    return Err(e);
                }
            };
            fees_res.sort_by_key(|f| f.slot);
            fees_res.reverse();
            let sum_fees: u64 = fees_res
                .iter()
                .take(10)
                .map(|f| f.prioritization_fee)
                .collect::<Vec<u64>>()
                .iter()
                .sum();
            info!(
                "Successfully fetched general prioritization fees. Average 5s Rolling Fee: {}",
                sum_fees / 10
            );
            *self.recent_priority_fees.write().await = fees_res;
        }

        Ok(())
    }

    #[inline(always)]
    async fn get_recent_prioritization_fees(
        self: &Arc<Self>,
        accounts: Option<&[Pubkey]>,
    ) -> Result<Vec<RpcPrioritizationFee>, ClientError> {
        let account_addressess = if let Some(a) = accounts {
            a.to_vec()
        } else {
            vec![]
        };
        match self
            .rpc_client
            .get_recent_prioritization_fees(&account_addressess)
            .await
        {
            Ok(rpf) => Ok(rpf),
            Err(e) => {
                warn!(
                    "Failed to fetch recent prioritization fees: {}",
                    e.to_string()
                );
                Err(e)
            }
        }
    }

    /// Gets the latest block [`Hash`] cached.
    #[inline(always)]
    pub async fn get_latest_blockhash(self: &Arc<Self>) -> Hash {
        *self.recent_blockhash.read().await
    }

    /// Gets the general recent priority fees.
    #[inline(always)]
    pub async fn get_priority_fees(self: &Arc<Self>) -> Vec<RpcPrioritizationFee> {
        self.recent_priority_fees.read().await.clone()
    }

    /// Gets the recent priority fees for a given account.
    #[inline(always)]
    pub async fn get_priority_fees_for_accounts(
        self: &Arc<Self>,
        alias: &str,
    ) -> Vec<RpcPrioritizationFee> {
        let accounts_map = self.accounts_map.read().await;
        match accounts_map.iter().find(|am| am.alias == alias) {
            Some(am) => {
                let fees = am.recent_priority_fees.read().await;
                fees.clone()
            }
            None => Vec::new(),
        }
    }
}
