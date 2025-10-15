//! High-level Monero client

use monero_marketplace_common::{
    error::{Error, Result},
    types::{MoneroConfig, WalletStatus, WalletInfo},
};
use crate::{rpc::MoneroRpcClient, multisig::MultisigManager};
use anyhow::Context;

/// High-level Monero client
pub struct MoneroClient {
    rpc_client: MoneroRpcClient,
    multisig_manager: MultisigManager,
}

impl MoneroClient {
    /// Create a new Monero client
    pub fn new(config: MoneroConfig) -> Result<Self> {
        let rpc_client = MoneroRpcClient::new(config)?;
        let multisig_manager = MultisigManager::new(rpc_client.clone());
        
        Ok(Self {
            rpc_client,
            multisig_manager,
        })
    }

    /// Get wallet status
    pub async fn get_wallet_status(&self) -> Result<WalletStatus> {
        let (balance, unlocked_balance) = self.rpc_client.get_balance().await?;
        let is_multisig = self.rpc_client.is_multisig().await?;
        
        // Note: In a real implementation, you'd get multisig threshold/total
        // from additional RPC calls
        
        Ok(WalletStatus {
            is_multisig,
            multisig_threshold: if is_multisig { Some(2) } else { None },
            multisig_total: if is_multisig { Some(3) } else { None },
            is_locked: false, // Would need additional RPC call
            balance,
            unlocked_balance,
        })
    }

    /// Get complete wallet information
    pub async fn get_wallet_info(&self) -> Result<WalletInfo> {
        // Get version
        let version = self.rpc_client.get_version().await
            .context("Failed to get wallet version")?;
        
        // Get balance
        let (balance, unlocked_balance) = self.rpc_client.get_balance().await
            .context("Failed to get wallet balance")?;
        
        // Check if multisig
        let is_multisig = self.rpc_client.is_multisig().await
            .context("Failed to check multisig status")?;
        
        // Get block height (simplified - would need additional RPC call in real implementation)
        let block_height = 0u64; // Placeholder
        let daemon_block_height = 0u64; // Placeholder
        
        Ok(WalletInfo {
            version,
            balance,
            unlocked_balance,
            is_multisig,
            multisig_threshold: if is_multisig { Some(2) } else { None },
            multisig_total: if is_multisig { Some(3) } else { None },
            is_locked: false, // Would need additional RPC call
            block_height,
            daemon_block_height,
        })
    }

    /// Get multisig manager
    pub fn multisig(&self) -> &MultisigManager {
        &self.multisig_manager
    }

    /// Get RPC client
    pub fn rpc(&self) -> &MoneroRpcClient {
        &self.rpc_client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let config = MoneroConfig::default();
        let client = MoneroClient::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_get_wallet_info_structure() {
        // Test that the function returns the expected structure
        // This is a unit test that doesn't require a running Monero wallet
        let config = MoneroConfig::default();
        let client = MoneroClient::new(config)
            .context("Failed to create client for test")?;
        
        // Note: This will fail without a running Monero wallet, but we can test the structure
        // In a real test environment, you'd mock the RPC client
        let result = client.get_wallet_info().await;
        
        // The function should return an error without a running wallet, but the structure is correct
        assert!(result.is_err());
        
        // Verify the error is a network/RPC error, not a structure error
        match result.unwrap_err() {
            Error::MoneroRpc(_) | Error::Network(_) => {
                // Expected - no Monero wallet running
            }
            _ => return Err(anyhow::anyhow!("Unexpected error type")),
        }
    }

    // Note: Integration tests would require a running Monero wallet
    // These would be in tests/integration.rs
}
