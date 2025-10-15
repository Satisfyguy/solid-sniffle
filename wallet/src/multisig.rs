//! Multisig functionality for Monero escrow

use monero_marketplace_common::{
    error::{Error, Result},
    types::MultisigInfo,
};
use crate::rpc::MoneroRpcClient;

/// Multisig manager for handling escrow operations
pub struct MultisigManager {
    rpc_client: MoneroRpcClient,
}

impl MultisigManager {
    /// Create a new multisig manager
    pub fn new(rpc_client: MoneroRpcClient) -> Self {
        Self { rpc_client }
    }

    /// Prepare multisig (step 1 of multisig creation)
    pub async fn prepare_multisig(&self) -> Result<MultisigInfo> {
        #[derive(serde::Deserialize)]
        struct PrepareResponse {
            multisig_info: String,
        }

        let response: PrepareResponse = self
            .rpc_client
            .call("prepare_multisig", None)
            .await
            .context("Failed to prepare multisig")?;

        Ok(MultisigInfo {
            info: response.multisig_info,
        })
    }

    /// Make multisig (step 2 of multisig creation)
    pub async fn make_multisig(&self, multisig_infos: Vec<String>) -> Result<MultisigInfo> {
        if multisig_infos.len() != 2 {
            return Err(Error::Multisig(
                "make_multisig requires exactly 2 multisig infos".to_string(),
            ));
        }

        #[derive(serde::Deserialize)]
        struct MakeResponse {
            multisig_info: String,
        }

        let params = serde_json::json!({
            "multisig_info": multisig_infos
        });

        let response: MakeResponse = self
            .rpc_client
            .call("make_multisig", Some(params))
            .await
            .context("Failed to make multisig")?;

        Ok(MultisigInfo {
            info: response.multisig_info,
        })
    }

    /// Export multisig info
    pub async fn export_multisig_info(&self) -> Result<MultisigInfo> {
        #[derive(serde::Deserialize)]
        struct ExportResponse {
            info: String,
        }

        let response: ExportResponse = self
            .rpc_client
            .call("export_multisig_info", None)
            .await
            .context("Failed to export multisig info")?;

        Ok(MultisigInfo {
            info: response.info,
        })
    }

    /// Import multisig info
    pub async fn import_multisig_info(&self, multisig_infos: Vec<String>) -> Result<u32> {
        #[derive(serde::Deserialize)]
        struct ImportResponse {
            n_outputs: u32,
        }

        let params = serde_json::json!({
            "info": multisig_infos
        });

        let response: ImportResponse = self
            .rpc_client
            .call("import_multisig_info", Some(params))
            .await
            .context("Failed to import multisig info")?;

        Ok(response.n_outputs)
    }

    /// Check if wallet is multisig
    pub async fn is_multisig(&self) -> Result<bool> {
        self.rpc_client.is_multisig().await
    }

    /// Get multisig info
    pub async fn get_multisig_info(&self) -> Result<MultisigInfo> {
        #[derive(serde::Deserialize)]
        struct InfoResponse {
            info: String,
        }

        let response: InfoResponse = self
            .rpc_client
            .call("get_multisig_info", None)
            .await
            .context("Failed to get multisig info")?;

        Ok(MultisigInfo {
            info: response.info,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monero_marketplace_common::types::MoneroConfig;

    #[tokio::test]
    async fn test_multisig_manager_creation() {
        let config = MoneroConfig::default();
        let rpc_client = MoneroRpcClient::new(config)
            .expect("Failed to create RPC client for test");
        let manager = MultisigManager::new(rpc_client);
        // Manager created successfully
        assert!(true);
    }

    // Note: Integration tests would require a running Monero wallet
    // These would be in tests/integration.rs
}
