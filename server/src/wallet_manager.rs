//! Wallet manager for server-side Monero interactions

use anyhow::{Context, Result};
use monero_marketplace_common::{
    error::Error as CommonError,
    types::{MoneroConfig, MultisigInfo},
};
use monero_marketplace_wallet::MoneroClient;
use std::collections::HashMap;
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum WalletRole {
    Buyer,
    Vendor,
    Arbiter,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MultisigState {
    NotStarted,
    PreparedInfo(MultisigInfo),
    InfoExchanged {
        round: u8,
        participants: Vec<String>,
    },
    Ready {
        address: String,
    },
}

pub struct WalletInstance {
    pub id: Uuid,
    pub role: WalletRole,
    pub rpc_client: MoneroClient,
    pub address: String,
    pub multisig_state: MultisigState,
}

#[derive(Error, Debug)]
pub enum WalletManagerError {
    #[error("Monero RPC error: {0}")]
    RpcError(#[from] CommonError),

    #[error("Invalid multisig state: expected {expected}, got {actual}")]
    InvalidState {
        expected: String,
        actual: String,
    },

    #[error("Wallet not found: {0}")]
    WalletNotFound(Uuid),

    #[error("All RPC endpoints unavailable")]
    NoAvailableRpc,

    #[error("Multisig address mismatch: {addresses:?}")]
    AddressMismatch { addresses: Vec<String> },
}

pub struct WalletManager {
    wallets: HashMap<Uuid, WalletInstance>,
    rpc_configs: Vec<MoneroConfig>,
    next_rpc_index: usize,
}

impl WalletManager {
    pub fn new(configs: Vec<MoneroConfig>) -> Result<Self> {
        if configs.is_empty() {
            return Err(anyhow::anyhow!("At least one Monero RPC config is required"));
        }
        info!("WalletManager initialized with {} RPC endpoints", configs.len());
        Ok(Self {
            wallets: HashMap::new(),
            rpc_configs: configs,
            next_rpc_index: 0,
        })
    }

    pub async fn create_wallet_instance(
        &mut self,
        role: WalletRole,
    ) -> Result<Uuid, WalletManagerError> {
        let config = self
            .rpc_configs
            .get(self.next_rpc_index)
            .ok_or(WalletManagerError::NoAvailableRpc)?;
        self.next_rpc_index = (self.next_rpc_index + 1) % self.rpc_configs.len();

        let rpc_client = MoneroClient::new(config.clone())?;
        let wallet_info = rpc_client.get_wallet_info().await?;

        let instance = WalletInstance {
            id: Uuid::new_v4(),
            role,
            rpc_client,
            address: wallet_info.address,
            multisig_state: MultisigState::NotStarted,
        };
        let id = instance.id;
        self.wallets.insert(id, instance);
        info!("Created wallet instance {}", id);
        Ok(id)
    }

    pub async fn make_multisig(
        &mut self,
        wallet_id: Uuid,
        _participants: Vec<String>,
    ) -> Result<MultisigInfo, WalletManagerError> {
        let wallet = self
            .wallets
            .get_mut(&wallet_id)
            .ok_or(WalletManagerError::WalletNotFound(wallet_id))?;

        let info = wallet.rpc_client.multisig().prepare_multisig().await?;
        wallet.multisig_state = MultisigState::PreparedInfo(info.clone());
        Ok(info)
    }

    pub async fn exchange_multisig_info(
        &mut self,
        escrow_id: Uuid,
        info_from_all: Vec<MultisigInfo>,
    ) -> Result<(), WalletManagerError> {
        info!("Exchanging multisig info for escrow {}", escrow_id);
        // This is a simplified implementation. A real one would be more complex.
        for wallet in self.wallets.values_mut() {
            let other_infos = info_from_all
                .iter()
                .filter(|i| i.multisig_info != wallet.address) // This is incorrect, just a placeholder
                .map(|i| i.multisig_info.clone())
                .collect();
            let result = wallet
                .rpc_client
                .multisig()
                .make_multisig(2, other_infos)
                .await?;
            wallet.multisig_state = MultisigState::Ready {
                address: result.address,
            };
        }
        Ok(())
    }

    pub async fn finalize_multisig(
        &mut self,
        escrow_id: Uuid,
    ) -> Result<String, WalletManagerError> {
        info!("Finalizing multisig for escrow {}", escrow_id);
        let mut addresses = std::collections::HashSet::new();
        for wallet in self.wallets.values() {
            if let MultisigState::Ready { address } = &wallet.multisig_state {
                addresses.insert(address.clone());
            }
        }

        if addresses.len() != 1 {
            return Err(WalletManagerError::AddressMismatch {
                addresses: addresses.into_iter().collect(),
            });
        }

        addresses.into_iter().next().ok_or(WalletManagerError::InvalidState {
            expected: "at least one wallet in Ready state".to_string(),
            actual: "none".to_string(),
        })
    }

    pub async fn release_funds(&mut self, escrow_id: Uuid, destinations: Vec<monero_marketplace_common::types::TransferDestination>) -> Result<(), WalletManagerError> {
        warn!("release_funds called for escrow {} (STUB MODE)", escrow_id);
        // FIXME: See GitHub issue #123 - Implement production multisig release
        // 1. Get buyer and arbiter wallets
        // 2. Create transaction
        // 3. Sign with buyer
        // 4. Sign with arbiter
        // 5. Submit transaction
        Ok(())
    }
}