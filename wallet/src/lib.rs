//! Monero wallet integration and multisig functionality
//!
//! This crate provides the core functionality for interacting with
//! Monero wallets, including multisig operations for escrow.

pub mod client;
pub mod escrow;
pub mod multisig;
pub mod rpc;
pub mod tor;
pub mod transaction;

pub use client::MoneroClient;
pub use escrow::EscrowManager;
pub use multisig::MultisigManager;
pub use rpc::MoneroRpcClient;
pub use transaction::TransactionManager;
