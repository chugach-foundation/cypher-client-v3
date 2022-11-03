use async_trait::async_trait;

use super::{GenericOrderBook, Order};

/// A trait that can be used to generically get data for both AOB and Serum Orders Accounts.
#[async_trait(?Send)]
pub trait GenericOpenOrders {
    /// Gets open orders in the orders account and maps them with the existing orders on the given Order Book.
    ///
    /// Callee must make sure that the given [GenericOrderBook] is of a type that is compatible with the
    async fn get_open_orders(&self, orderbook: &dyn GenericOrderBook) -> Vec<Order>;
}

/// Represents the Cypher Open Orders Account Context (used for the AOB).

pub struct AgnosticOpenOrdersContext {}

#[async_trait(?Send)]
impl GenericOpenOrders for AgnosticOpenOrdersContext {
    async fn get_open_orders(&self, orderbook: &dyn GenericOrderBook) -> Vec<Order> {
        Vec::new()
    }
}

/// Represents the Serum Open Orders Account Context.
pub struct SerumOpenOrdersContext {}

#[async_trait(?Send)]
impl GenericOpenOrders for SerumOpenOrdersContext {
    async fn get_open_orders(&self, orderbook: &dyn GenericOrderBook) -> Vec<Order> {
        Vec::new()
    }
}
