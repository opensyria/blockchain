//! Open Syria Block Explorer Backend

pub mod api;
pub mod handlers;
pub mod server;
pub mod types;
pub mod websocket;

#[cfg(test)]
mod tests;

pub use server::ExplorerServer;
