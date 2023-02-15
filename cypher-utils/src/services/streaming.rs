use {
    crate::{
        accounts_cache::{AccountState, AccountsCache},
        constants::{JSON_RPC_URL, PUBSUB_RPC_URL},
        services::utils::get_account_info,
    },
    dashmap::DashMap,
    futures::StreamExt,
    log::{info, warn},
    solana_account_decoder::UiAccountEncoding,
    solana_client::{
        client_error::ClientError,
        nonblocking::{
            pubsub_client::{PubsubClient, PubsubClientError},
            rpc_client::RpcClient,
        },
        rpc_config::RpcAccountInfoConfig,
    },
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey},
    std::sync::Arc,
    tokio::sync::broadcast::{channel, error::SendError, Sender},
};

/// A Service which allows subscribing to Accounts and receiving updates
/// to their state via an [`AccountsCache`].
pub struct StreamingAccountInfoService {
    cache: Arc<AccountsCache>,
    pubsub_client: Arc<PubsubClient>,
    rpc_client: Arc<RpcClient>,
    pub subscriptions_map: DashMap<Pubkey, Arc<SubscriptionHandler>>,
    shutdown: Arc<Sender<bool>>,
}

impl Default for StreamingAccountInfoService {
    fn default() -> Self {
        let pubsub_client = futures::executor::block_on(PubsubClient::new(PUBSUB_RPC_URL));
        Self {
            cache: Arc::new(AccountsCache::default()),
            pubsub_client: Arc::new(pubsub_client.unwrap()),
            rpc_client: Arc::new(RpcClient::new(JSON_RPC_URL.to_string())),
            shutdown: Arc::new(channel::<bool>(1).0),
            subscriptions_map: DashMap::new(),
        }
    }
}

impl std::fmt::Debug for StreamingAccountInfoService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamingAccountInfoService").finish()
    }
}

impl StreamingAccountInfoService {
    /// Creates a new [`StreamingAccountInfoService`].
    pub fn new(
        cache: Arc<AccountsCache>,
        pubsub_client: Arc<PubsubClient>,
        rpc_client: Arc<RpcClient>,
        shutdown: Arc<Sender<bool>>,
    ) -> Self {
        Self {
            cache,
            pubsub_client,
            rpc_client,
            shutdown,
            subscriptions_map: DashMap::new(),
        }
    }

    /// Starts the service's work cycle.
    /// Initially fetches the Account's states using the [`RpcClient`]
    /// and then subscribes to changes via [`PubsubClient`].
    #[inline(always)]
    pub async fn start_service(self: &Arc<Self>) {
        let mut shutdown_receiver = self.shutdown.subscribe();

        tokio::select! {
            _ = shutdown_receiver.recv() => {
                info!("Shutting down subscription handlers.");
                for handler in self.subscriptions_map.iter() {
                    match handler.stop().await {
                        Ok(_) => {
                            info!("Successfully sent shutdown signal to handler: {}", handler.account);
                        },
                        Err(e) => {
                            warn!(
                                "There was an error removing subscription handler for account {}: {}",
                                handler.account,
                                e.to_string()
                            );
                        }
                    }
                }
            }
        }
    }

    /// Adds new subscriptions to the service.
    #[inline(always)]
    pub async fn add_subscriptions(self: &Arc<Self>, new_accounts: &[Pubkey]) {
        match self.get_account_infos(new_accounts).await {
            Ok(()) => (),
            Err(e) => {
                warn!(
                    "There was an error while fetching new account infos: {}",
                    e.to_string()
                );
            }
        }

        for account in new_accounts.iter() {
            info!("Adding subscription handler for: {}", account);
            let handler = Arc::new(SubscriptionHandler::new(
                Arc::clone(&self.pubsub_client),
                Arc::clone(&self.cache),
                Arc::new(channel::<bool>(1).0),
                *account,
            ));
            let cloned_handler = Arc::clone(&handler);
            tokio::spawn(async move {
                match cloned_handler.run().await {
                    Ok(_) => {
                        info!(
                            "Subscription handler for account: {} gracefully stopped.",
                            cloned_handler.account
                        );
                    }
                    Err(e) => {
                        warn!(
                            "There was an error running subscription handler for account {}: {}",
                            cloned_handler.account,
                            e.to_string()
                        );
                    }
                }
            });
            self.subscriptions_map.insert(*account, handler);
            info!("Successfully added subscription handler for: {}.", account);
        }
        info!(
            "Successfully added {} new subscriptions.",
            new_accounts.len()
        );
    }

    /// Attempts to remove existing subscriptions from the service.
    #[inline(always)]
    pub async fn remove_subscriptions(self: &Arc<Self>, accounts: &[Pubkey]) {
        let mut accounts_to_remove = Vec::new();
        let handlers = self.subscriptions_map.iter();

        for (idx, handler_ref) in self.subscriptions_map.iter().enumerate() {
            if accounts.contains(&handler_ref.account) {
                accounts_to_remove.push(handler_ref.account);
            }
        }

        for account in accounts_to_remove.iter() {
            match self.subscriptions_map.remove(&account) {
                Some(handler) => match handler.1.stop().await {
                    Ok(_) => {
                        info!(
                            "Successfully sent shutdown signal to handler for account: {}",
                            handler.0
                        );
                    }
                    Err(e) => {
                        warn!(
                            "There was an error removing subscription handler for account {}: {}",
                            handler.0,
                            e.to_string()
                        );
                        continue;
                    }
                },
                None => {
                    warn!(
                        "Failed to remove subscription handler for account: {}",
                        account
                    );
                }
            };
        }
    }

    #[inline(always)]
    async fn get_account_infos(&self, accounts: &[Pubkey]) -> Result<(), ClientError> {
        info!("Fetching {} account infos.", accounts.len());
        let res = match self
            .rpc_client
            .get_multiple_accounts_with_commitment(accounts, CommitmentConfig::confirmed())
            .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!("Could not fetch account infos: {}", e.to_string());
                return Err(e);
            }
        };

        let mut infos = res.value;
        info!("Fetched {} account infos.", infos.len());

        while !infos.is_empty() {
            let next = infos.pop().unwrap();
            let i = infos.len();
            let key = accounts[i];

            let info = match next {
                Some(ai) => ai,
                None => {
                    warn!("[{}/{}] An account info was missing!!", i, infos.len());
                    continue;
                }
            };
            self.cache
                .insert(
                    key,
                    AccountState {
                        account: key,
                        data: info.data,
                        slot: res.context.slot,
                    },
                )
                .await;
        }

        Ok(())
    }
}

/// The subscription handler which is responsible for processing updates
/// to an Account's state.
pub struct SubscriptionHandler {
    cache: Arc<AccountsCache>,
    pubsub_client: Arc<PubsubClient>,
    shutdown: Arc<Sender<bool>>,
    pub account: Pubkey,
}

impl SubscriptionHandler {
    /// Creates a new [`SubscriptionHandler`].
    pub fn new(
        pubsub_client: Arc<PubsubClient>,
        cache: Arc<AccountsCache>,
        shutdown: Arc<Sender<bool>>,
        account: Pubkey,
    ) -> Self {
        Self {
            cache,
            pubsub_client,
            shutdown,
            account,
        }
    }

    /// Subscribes to the provided Account and processes updates.
    /// While the subscription persists, the handler will update the correspoding entry
    /// for the provided Account in it's [`AccountsCache`].
    #[inline(always)]
    pub async fn run(self: &Arc<Self>) -> Result<(), PubsubClientError> {
        let mut shutdown_receiver = self.shutdown.subscribe();
        let sub = match self
            .pubsub_client
            .account_subscribe(
                &self.account,
                Some(RpcAccountInfoConfig {
                    commitment: Some(CommitmentConfig::confirmed()),
                    encoding: Some(UiAccountEncoding::Base64),
                    ..Default::default()
                }),
            )
            .await
        {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to subscribe to accounts: {}", e.to_string());
                return Err(e);
            }
        };

        let mut stream = sub.0;
        loop {
            tokio::select! {
                update = stream.next() => {
                    match update {
                        Some(account) => {
                            let account_data = match get_account_info(&account.value) {
                                Ok(data) => data,
                                Err(e) => {
                                    warn!("Failed to decode account data: {}", e.to_string());
                                    continue;
                                }
                            };
                            info!("Received account update for {}, updating cache.",  self.account);
                            self.cache.insert(self.account, AccountState {
                                account: self.account,
                                data: account_data,
                                slot: account.context.slot,
                            }).await;
                        }
                        None => ()
                    }
                },
                _ = shutdown_receiver.recv() => {
                    info!("Shutting down subscription handler for {}",  self.account);
                    break;
                }
            }
        }
        Ok(())
    }

    /// Stops the subscription handler from processing additional messages.
    #[inline(always)]
    pub async fn stop(self: &Arc<Self>) -> Result<usize, SendError<bool>> {
        self.shutdown.send(true)
    }
}
