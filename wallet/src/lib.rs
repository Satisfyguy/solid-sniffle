//! Monero wallet integration and multisig functionality
//!
//! This crate provides the core functionality for interacting with
//! Monero wallets, including multisig operations for escrow.

pub mod client;
pub mod multisig;
pub mod rpc;
pub mod tor;

pub use client::MoneroClient;
pub use multisig::MultisigManager;
pub use rpc::MoneroRpcClient;
