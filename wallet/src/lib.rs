//! Monero wallet integration and multisig functionality
//! 
//! This crate provides the core functionality for interacting with
//! Monero wallets, including multisig operations for escrow.

pub mod rpc;
pub mod multisig;
pub mod client;
pub mod tor;

pub use client::MoneroClient;
pub use rpc::MoneroRpcClient;
pub use multisig::MultisigManager;
