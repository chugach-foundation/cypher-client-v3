use cypher_utils::{
    constants::{JSON_RPC_URL, PUBSUB_RPC_URL},
    logging::init_logger,
    services::ChainMetaService,
};
use log::{info, warn};
use solana_client::nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient};
use solana_sdk::pubkey::Pubkey;
use std::{str::FromStr, sync::Arc};
use tokio::sync::broadcast::channel;

#[tokio::main]
async fn main() {
    init_logger().unwrap();

    // you need to make sure that the provided rpc already supports the required methods
    let rpc_client = Arc::new(RpcClient::new(JSON_RPC_URL.to_string()));
    let pubsub_client = Arc::new(PubsubClient::new(PUBSUB_RPC_URL).await.unwrap());

    let shutdown = channel::<bool>(1).0;
    let cms = Arc::new(ChainMetaService::new(
        rpc_client.clone(),
        pubsub_client.clone(),
        shutdown.subscribe(),
        false,
        true,
    ));
    let cms_clone = cms.clone();
    let handle = tokio::spawn(async move {
        match cms_clone.start_service().await {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to run CMS: {}", e.to_string());
            }
        }
    });

    // this is the sol/usdc openbook market
    let accounts = vec![Pubkey::from_str("8BnEgHoWFysVcuFFX7QztDmzuH8r5ZFvyP3sYwn1XTh6").unwrap()];
    cms.add_priority_fees_accounts("OpenBook SOL/USDC", &accounts)
        .await;

    match tokio::join!(handle) {
        (Ok(()),) => {
            info!("Successfully stopped chain meta service.");
        }
        (Err(e),) => {
            warn!(
                "Failed to join with chain meta service task: {}",
                e.to_string()
            );
        }
    }
}
