use cypher_client::{
    constants::QUOTE_TOKEN_DECIMALS,
    utils::{convert_price_to_decimals_fixed, fixed_to_ui, native_to_ui_fixed},
    Market, PerpetualMarket, Side,
};
use cypher_utils::{
    accounts_cache::AccountsCache,
    constants::{JSON_RPC_URL, PUBSUB_RPC_URL},
    contexts::{AgnosticOrderBookContext, MarketContext},
    logging::init_logger,
    services::StreamingAccountInfoService,
};
use log::warn;
use solana_client::nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient};
use std::env;
use std::sync::Arc;
use tokio::sync::broadcast::channel;

#[tokio::main]
async fn main() {
    init_logger().unwrap();

    // if no env variables are set, will default to public ones and will experience rate limiting
    let market_name = env::var("MARKET").unwrap_or(("SOL-PERP").to_string());
    let rpc_addy = env::var("RPC").unwrap_or(JSON_RPC_URL.to_string());
    let pubsub_addy = env::var("WSS").unwrap_or(PUBSUB_RPC_URL.to_string());

    println!("Market: {}", market_name);
    println!("RPC: {}", rpc_addy);
    println!("WSS: {}", pubsub_addy);

    // you need to make sure that the provided rpc already supports the required methods
    let rpc_client = Arc::new(RpcClient::new(rpc_addy.to_string()));
    let pubsub_client = Arc::new(PubsubClient::new(pubsub_addy.as_str()).await.unwrap());
    let shutdown = Arc::new(channel::<bool>(1).0);

    let accounts_cache = Arc::new(AccountsCache::new());

    let obs = Arc::new(StreamingAccountInfoService::new(
        accounts_cache.clone(),
        pubsub_client.clone(),
        rpc_client.clone(),
        shutdown.clone(),
    ));
    let obs_clone = obs.clone();
    let handle = tokio::spawn(async move {
        obs_clone.start_service().await;
    });

    let market_ctx =
        MarketContext::<PerpetualMarket>::load_with_name(&rpc_client, market_name.as_str())
            .await
            .unwrap();

    let market_state = market_ctx.state.as_ref();
    let market = &market_ctx.address;
    let bids = market_ctx.state.inner.bids;
    let asks = market_ctx.state.inner.asks;

    let mut order_book =
        AgnosticOrderBookContext::load(&rpc_client, market_state, &market, &bids, &asks)
            .await
            .unwrap();

    let accounts = vec![bids, asks];
    obs.add_subscriptions(&accounts).await;

    let mut account_receiver = accounts_cache.subscribe(&accounts).await;

    loop {
        tokio::select! {
            account_state_update = account_receiver.recv() => {
                match account_state_update {
                    Ok(account_state) => {
                        if account_state.account == bids {
                            order_book.reload_from_account_data(market_state, &account_state.data, Side::Bid);
                        } else if account_state.account == asks {
                            order_book.reload_from_account_data(market_state, &account_state.data, Side::Ask);
                        }
                        display_orderbook(&order_book, market_state);
                    },
                    Err(e) => {
                        warn!("There was an error receiving account state update. Error: {:?}", e);
                    }
                }
            }
        }
    }
}

pub fn display_orderbook(book_ctx: &AgnosticOrderBookContext, market: &dyn Market) {
    let bids = &book_ctx.state.bids;
    let asks = &book_ctx.state.asks;

    println!("Bids: {} - Asks: {}", bids.len(), asks.len());

    let longest = if bids.len() > asks.len() {
        bids.len()
    } else {
        asks.len()
    };

    println!("\n| {:^43} | {:^43} |", "Bids", "Asks",);
    println!(
        "| {:^20} | {:^20} | {:^20} | {:^20} |",
        "Price", "Size", "Size", "Price",
    );

    for i in 0..longest {
        let bid = bids.get(i);
        let ask = asks.get(i);

        let output = match (bid, ask) {
            (None, None) => format!("| {:^20.4} | {:^20.4} | {:^20.4} | {:^20.4} |", 0, 0, 0, 0),
            (None, Some(ask)) => format!(
                "| {:^20.4} | {:^20.4} | {:^20.4} | {:^20.4} |",
                0,
                0,
                // at this point the order size already comes in native units
                // so we do not need to convert from lots to native
                native_to_ui_fixed(ask.base_quantity, market.decimals()),
                fixed_to_ui(
                    convert_price_to_decimals_fixed(
                        ask.price,
                        market.base_multiplier(),
                        10u64.pow(market.decimals() as u32),
                        market.quote_multiplier()
                    ),
                    QUOTE_TOKEN_DECIMALS
                ),
            ),
            (Some(bid), None) => format!(
                "| {:^20.4} | {:^20.4} | {:^20.4} | {:^20.4} |",
                fixed_to_ui(
                    convert_price_to_decimals_fixed(
                        bid.price,
                        market.base_multiplier(),
                        10u64.pow(market.decimals() as u32),
                        market.quote_multiplier()
                    ),
                    QUOTE_TOKEN_DECIMALS
                ),
                // at this point the order size already comes in native units
                // so we do not need to convert from lots to native
                native_to_ui_fixed(bid.base_quantity, market.decimals()),
                0,
                0
            ),
            (Some(bid), Some(ask)) => format!(
                "| {:^20.4} | {:^20.4} | {:^20.4} | {:^20.4} |",
                fixed_to_ui(
                    convert_price_to_decimals_fixed(
                        bid.price,
                        market.base_multiplier(),
                        10u64.pow(market.decimals() as u32),
                        market.quote_multiplier()
                    ),
                    QUOTE_TOKEN_DECIMALS
                ),
                // at this point the order size already comes in native units
                // so we do not need to convert from lots to native
                native_to_ui_fixed(bid.base_quantity, market.decimals()),
                native_to_ui_fixed(ask.base_quantity, market.decimals()),
                fixed_to_ui(
                    convert_price_to_decimals_fixed(
                        ask.price,
                        market.base_multiplier(),
                        10u64.pow(market.decimals() as u32),
                        market.quote_multiplier()
                    ),
                    QUOTE_TOKEN_DECIMALS
                ),
            ),
        };

        println!("{}", output);
    }
}
