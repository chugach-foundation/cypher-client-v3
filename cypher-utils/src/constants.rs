#[cfg(not(feature = "mainnet-beta"))]
pub const PUBSUB_RPC_URL: &str = "wss://api.devnet.solana.com";
#[cfg(not(feature = "mainnet-beta"))]
pub const JSON_RPC_URL: &str = "https://api.devnet.solana.com";

#[cfg(feature = "mainnet-beta")]
pub const PUBSUB_RPC_URL: &str = "wss://api.mainnet-beta.solana.com";
#[cfg(feature = "mainnet-beta")]
pub const JSON_RPC_URL: &str = "https://api.mainnet-beta.solana.com";
