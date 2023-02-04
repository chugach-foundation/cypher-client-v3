use {
    crate::{
        accounts_cache::{AccountState, AccountsCache},
        constants::{JSON_RPC_URL, PUBSUB_RPC_URL},
        services::utils::get_account_info,
    },
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
    tokio::{
        runtime::Handle,
        sync::{
            broadcast::{channel, error::SendError, Receiver, Sender},
            RwLock,
        },
    },
};

/// A Service which allows subscribing to Accounts and receiving updates
/// to their state via an [`AccountsCache`].
pub struct StreamingAccountInfoService {
    pub cache: Arc<AccountsCache>,
    pub pubsub_client: Arc<PubsubClient>,
    pub rpc_client: Arc<RpcClient>,
    pub accounts: RwLock<Vec<Pubkey>>,
    shutdown: RwLock<Receiver<bool>>,
    handlers: RwLock<Vec<Arc<SubscriptionHandler>>>,
}

impl Default for StreamingAccountInfoService {
    fn default() -> Self {
        let pubsub_client = futures::executor::block_on(PubsubClient::new(PUBSUB_RPC_URL));
        Self {
            cache: Arc::new(AccountsCache::default()),
            pubsub_client: Arc::new(pubsub_client.unwrap()),
            rpc_client: Arc::new(RpcClient::new(JSON_RPC_URL.to_string())),
            accounts: RwLock::new(Vec::new()),
            shutdown: RwLock::new(channel::<bool>(1).1),
            handlers: RwLock::new(Vec::new()),
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
        shutdown: Receiver<bool>,
        accounts: &[Pubkey],
    ) -> Self {
        Self {
            cache,
            pubsub_client,
            rpc_client,
            shutdown: RwLock::new(shutdown),
            accounts: RwLock::new(accounts.to_vec()),
            handlers: RwLock::new(Vec::new()),
        }
    }

    /// Starts the service's work cycle.
    /// Initially fetches the Account's states using the [`RpcClient`]
    /// and then subscribes to changes via [`PubsubClient`].
    #[inline(always)]
    pub async fn start_service(self: &Arc<Self>) {
        let accounts = self.accounts.read().await;
        match self.get_account_infos(&accounts).await {
            Ok(()) => (),
            Err(e) => {
                warn!(
                    "There was an error while fetching initial account infos: {}",
                    e.to_string()
                );
            }
        }

        let mut handlers = self.handlers.write().await;

        for account in accounts.iter() {
            let handler = Arc::new(SubscriptionHandler::new(
                Arc::clone(&self.pubsub_client),
                Arc::clone(&self.cache),
                channel::<bool>(1).0,
                *account,
            ));
            let cloned_handler = Arc::clone(&handler);
            handlers.push(handler);
            tokio::spawn(async move {
                match cloned_handler.run().await {
                    Ok(_) => (),
                    Err(e) => {
                        warn!(
                            "There was an error running subscription handler for account {}: {}",
                            cloned_handler.account,
                            e.to_string()
                        );
                    }
                }
            });
        }

        // drop the references so new subscriptions can be added
        // after we start waiting on the shutdown receiver
        drop(handlers);
        drop(accounts);

        let mut shutdown_receiver = self.shutdown.write().await;

        tokio::select! {
            _ = shutdown_receiver.recv() => {
                info!("Shutting down subscription handlers.");
                let handlers = self.handlers.read().await;
                for handler in handlers.iter() {
                    match handler.stop().await {
                        Ok(_) => (),
                        Err(e) => {
                            warn!(
                                "There was an error removing subscription handler for account {}: {}",

                                handler.account,
                                e.to_string()
                            );
                            continue;
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

        let mut handlers_vec = Vec::new();
        let mut accounts_vec = Vec::new();

        for account in new_accounts.iter() {
            info!("Adding subscription handler for: {}", account);
            let handler = Arc::new(SubscriptionHandler::new(
                Arc::clone(&self.pubsub_client),
                Arc::clone(&self.cache),
                channel::<bool>(1).0,
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
            accounts_vec.push(*account);
            handlers_vec.push(handler);
            info!("Successfully added subscription handler for: {}.", account);
        }
        let mut handlers = self.handlers.write().await;
        handlers.extend(handlers_vec);
        drop(handlers);
        let mut accounts = self.accounts.write().await;
        accounts.extend(accounts_vec);
        drop(accounts);
        info!(
            "Successfully added {} new subscriptions.",
            new_accounts.len()
        );
    }

    /// Attempts to remove existing subscriptions from the service.
    #[inline(always)]
    pub async fn remove_subscriptions(self: &Arc<Self>, accounts: &[Pubkey]) {
        let mut idxs: Vec<usize> = Vec::new();
        let handlers = self.handlers.read().await;

        for (idx, handler) in handlers.iter().enumerate() {
            if accounts.contains(&handler.account) {
                match handler.stop().await {
                    Ok(_) => (),
                    Err(e) => {
                        warn!(
                            "There was an error removing subscription handler for account {}: {}",
                            handler.account,
                            e.to_string()
                        );
                        continue;
                    }
                }
                idxs.push(idx);
            }
        }
        drop(handlers);

        // check if we actually have indices to remove to avoid getting the locks
        if !idxs.is_empty() {
            // reverse the indices so we don't have to worry about
            // the elements shifting inside the vector whenever we remove an element
            idxs.sort();
            idxs.reverse();
            let mut handlers = self.handlers.write().await;
            let mut accounts = self.accounts.write().await;
            for idx in idxs.iter() {
                handlers.remove(*idx);
                accounts.remove(*idx);
            }
            drop(handlers);
            drop(accounts);
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
struct SubscriptionHandler {
    cache: Arc<AccountsCache>,
    pubsub_client: Arc<PubsubClient>,
    shutdown: RwLock<Sender<bool>>,
    pub account: Pubkey,
}

impl SubscriptionHandler {
    /// Creates a new [`SubscriptionHandler`].
    pub fn new(
        pubsub_client: Arc<PubsubClient>,
        cache: Arc<AccountsCache>,
        shutdown: Sender<bool>,
        account: Pubkey,
    ) -> Self {
        Self {
            cache,
            pubsub_client,
            shutdown: RwLock::new(shutdown),
            account,
        }
    }

    /// Subscribes to the provided Account and processes updates.
    /// While the subscription persists, the handler will update the correspoding entry
    /// for the provided Account in it's [`AccountsCache`].
    #[inline(always)]
    pub async fn run(self: &Arc<Self>) -> Result<(), PubsubClientError> {
        let shutdown = self.shutdown.read().await;
        let mut shutdown_receiver = shutdown.subscribe();
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
        let shutdown = self.shutdown.write().await;
        shutdown.send(true)
    }
}
