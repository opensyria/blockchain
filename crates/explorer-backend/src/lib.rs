//! Block Explorer Backend API
//!
//! REST API for querying blockchain state, blocks, and transactions.

pub mod api;
pub mod handlers;
pub mod server;
pub mod types;

#[cfg(test)]
mod tests;

pub use server::ExplorerServer;
