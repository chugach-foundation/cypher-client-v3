pub mod cache;
pub mod cypher;
pub mod event_queue;
pub mod market;
pub mod open_orders;
pub mod orderbook;
pub mod pool;
pub mod user;

pub use cache::*;
pub use cypher::*;
pub use event_queue::*;
pub use market::*;
pub use open_orders::*;
pub use orderbook::*;
pub use pool::*;
pub use user::*;

use solana_client::client_error::ClientError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Account state is not cached.")]
    MissingAccountState,
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[error(transparent)]
    ClientError(#[from] ClientError),
}
