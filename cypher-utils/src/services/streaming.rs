use {
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
    tokio::sync::{
        broadcast::{channel, Receiver, Sender},
        RwLock,
    },
};

use tokio::sync::broadcast::error::SendError;

use crate::{
    accounts_cache::{AccountState, AccountsCache},
    services::utils::get_account_info,
};

/// A Service which allows subscribing to Accounts and receiving updates
/// to their state via an [`AccountsCache`].
pub struct StreamingAccountInfoService {
    cache: Arc<AccountsCache>,
    pubsub_client: Arc<PubsubClient>,
    rpc_client: Arc<RpcClient>,
    shutdown: RwLock<Receiver<bool>>,
    pub accounts: RwLock<Vec<Pubkey>>,
    handlers: RwLock<Vec<Arc<SubscriptionHandler>>>,
}

impl StreamingAccountInfoService {
    pub async fn default() -> Self {
        Self {
            cache: Arc::new(AccountsCache::default()),
            pubsub_client: Arc::new(
                PubsubClient::new("wss://api.devnet.solana.com")
                    .await
                    .unwrap(),
            ),
            rpc_client: Arc::new(RpcClient::new("https://api.devnet.solana.com".to_string())),
            shutdown: RwLock::new(channel::<bool>(1).1),
            accounts: RwLock::new(Vec::new()),
            handlers: RwLock::new(Vec::new()),
        }
    }

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
                    "[AIS] There was an error while fetching initial account infos: {}",
                    e.to_string()
                );
            }
        }

        let mut handlers = self.handlers.write().await;
        let accounts = self.accounts.read().await;

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
                        warn!("[AIS] There was an error running subscription handler for account {}: {}", cloned_handler.account, e.to_string());
                    }
                }
            });
        }

        // drop the reference to handlers so new subscriptions can be added
        // after we start waiting on the shutdown receiver
        drop(handlers);

        let mut shutdown_receiver = self.shutdown.write().await;

        tokio::select! {
            _ = shutdown_receiver.recv() => {
                info!("[AIS] Shutting down subscription handlers.");
                let handlers = self.handlers.read().await;
                for handler in handlers.iter() {
                    match handler.stop().await {
                        Ok(_) => (),
                        Err(e) => {
                            warn!("[AIS] There was an error removing subscription handler for account {}: {}", handler.account, e.to_string());
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
                    "[AIS] There was an error while fetching new account infos: {}",
                    e.to_string()
                );
            }
        }
        let mut handlers = self.handlers.write().await;
        let mut accounts = self.accounts.write().await;

        for account in new_accounts.iter() {
            let handler = Arc::new(SubscriptionHandler::new(
                Arc::clone(&self.pubsub_client),
                Arc::clone(&self.cache),
                channel::<bool>(1).0,
                *account,
            ));
            let cloned_handler = Arc::clone(&handler);
            tokio::spawn(async move {
                match cloned_handler.run().await {
                    Ok(_) => (),
                    Err(e) => {
                        warn!("[AIS] There was an error running subscription handler for account {}: {}", cloned_handler.account, e.to_string());
                    }
                }
            });
            accounts.push(*account);
            handlers.push(handler);
        }
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
                        warn!("[AIS] There was an error removing subscription handler for account {}: {}", handler.account, e.to_string());
                        continue;
                    }
                }
                idxs.push(idx);
            }
        }

        // check if we actually have indices to remove to avoid getting the locks
        if !idxs.is_empty() {
            // reverse the indices so we don't have to worry about
            // the elements shifting inside the vector whenever we remove an element
            idxs.reverse();
            let mut handlers = self.handlers.write().await;
            let mut accounts = self.accounts.write().await;
            for idx in idxs.iter() {
                handlers.remove(*idx);
                accounts.remove(*idx);
            }
        }
    }

    #[inline(always)]
    async fn get_account_infos(&self, accounts: &[Pubkey]) -> Result<(), ClientError> {
        let rpc_result = self
            .rpc_client
            .get_multiple_accounts_with_commitment(accounts, CommitmentConfig::confirmed())
            .await;

        let res = match rpc_result {
            Ok(r) => r,
            Err(e) => {
                warn!("[AIS] Could not fetch account infos: {}", e.to_string());
                return Err(e);
            }
        };

        let mut infos = res.value;
        info!("[AIS] Fetched {} account infos.", infos.len());

        while !infos.is_empty() {
            let next = infos.pop().unwrap();
            let i = infos.len();
            let key = accounts[i];

            let info = match next {
                Some(ai) => ai,
                None => {
                    warn!(
                        "[AIS] [{}/{}] An account info was missing!!",
                        i,
                        infos.len()
                    );
                    continue;
                }
            };
            self.cache.insert(
                key,
                AccountState {
                    account: key,
                    data: info.data,
                    slot: res.context.slot,
                },
            );
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
    pub async fn run(self: &Arc<Self>) -> Result<(), PubsubClientError> {
        let shutdown = self.shutdown.read().await;
        let mut shutdown_receiver = shutdown.subscribe();
        let sub = self
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
            .unwrap();

        let mut stream = sub.0;
        loop {
            tokio::select! {
                update = stream.next() => {
                    if update.is_some() {
                        let account_res = update.unwrap();
                        let account_data = get_account_info(&account_res.value).unwrap();
                        info!("[AIS] Received account update for {}, updating cache.", self.account);
                        self.cache.insert(self.account, AccountState {
                            account: self.account,
                            data: account_data,
                            slot: account_res.context.slot,
                        });
                    }
                },
                _ = shutdown_receiver.recv() => {
                    info!("[AIS] Shutting down subscription handler for {}", self.account);
                    break;
                }
            }
        }
        Ok(())
    }

    /// Stops the subscription handler from processing additional messages.
    pub async fn stop(self: &Arc<Self>) -> Result<usize, SendError<bool>> {
        let shutdown = self.shutdown.write().await;
        shutdown.send(true)
    }
}
