//! Monero RPC client implementation

use common::{
    MoneroError, MultisigInfo, PrepareMultisigResult, RpcRequest, RpcResponse,
};
use reqwest::Client;
use std::time::Duration;
use anyhow::Context;

/// Client RPC Monero
///
/// HYPOTHÈSES (À VALIDER):
/// - RPC tourne sur localhost uniquement (pas exposé publiquement)
/// - Pas d'authentification (--disable-rpc-login en testnet)
/// - Timeout 30s acceptable
pub struct MoneroRpcClient {
    url: String,
    client: Client,
}

impl MoneroRpcClient {
    /// Crée nouveau client RPC
    ///
    /// # OPSEC Note
    /// RPC doit être sur localhost UNIQUEMENT.
    /// JAMAIS exposer sur 0.0.0.0 ou IP publique.
    pub fn new(url: String) -> Result<Self, MoneroError> {
        // OPSEC: Vérifier que URL est localhost
        if !url.contains("127.0.0.1") && !url.contains("localhost") {
            return Err(MoneroError::InvalidResponse(
                "RPC URL must be localhost only (OPSEC)".to_string(),
            ));
        }
        
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| MoneroError::NetworkError(format!("Client build: {}", e)))?;
        
        Ok(Self { url, client })
    }
    
    /// Vérifie que RPC est accessible
    pub async fn check_connection(&self) -> Result<(), MoneroError> {
        let request = RpcRequest::new("get_version");
        
        let response = self
            .client
            .post(&format!("{}/json_rpc", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    MoneroError::RpcUnreachable
                } else {
                    MoneroError::NetworkError(e.to_string())
                }
            })?;
        
        if !response.status().is_success() {
            return Err(MoneroError::RpcUnreachable);
        }
        
        Ok(())
    }
    
    /// Prépare wallet pour multisig (étape 1/6)
    ///
    /// # Errors
    /// - MoneroError::RpcUnreachable - RPC pas accessible
    /// - MoneroError::AlreadyMultisig - Wallet déjà en mode multisig
    /// - MoneroError::WalletLocked - Wallet verrouillé
    /// - MoneroError::InvalidResponse - Réponse invalide
    ///
    /// # Examples
    /// ```no_run
    /// # use wallet::MoneroRpcClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = MoneroRpcClient::new("http://127.0.0.1:18082".to_string())?;
    /// let info = client.prepare_multisig().await?;
    /// assert!(info.multisig_info.starts_with("MultisigV1"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn prepare_multisig(&self) -> Result<MultisigInfo, MoneroError> {
        let request = RpcRequest::new("prepare_multisig");
        
        // ERREUR POSSIBLE: RPC down
        let response = self
            .client
            .post(&format!("{}/json_rpc", self.url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    MoneroError::RpcUnreachable
                } else {
                    MoneroError::NetworkError(e.to_string())
                }
            })?;
        
        // ERREUR POSSIBLE: JSON invalide
        let rpc_response: RpcResponse<PrepareMultisigResult> = response
            .json()
            .await
            .map_err(|e| MoneroError::InvalidResponse(format!("JSON parse: {}", e)))?;
        
        // ERREUR POSSIBLE: RPC error (wallet déjà multisig, locked, etc.)
        if let Some(error) = rpc_response.error {
            return Err(match error.message.as_str() {
                msg if msg.contains("already") && msg.contains("multisig") => {
                    MoneroError::AlreadyMultisig
                }
                msg if msg.contains("locked") => MoneroError::WalletLocked,
                _ => MoneroError::RpcError(error.message),
            });
        }
        
        // ERREUR POSSIBLE: result absent
        let result = rpc_response
            .result
            .ok_or_else(|| MoneroError::InvalidResponse("Missing result field".to_string()))?;
        
        // VALIDATION: multisig_info doit commencer par "MultisigV1"
        if !result.multisig_info.starts_with("MultisigV1") {
            return Err(MoneroError::InvalidResponse(format!(
                "Invalid multisig_info format: {}",
                &result.multisig_info[..20]
            )));
        }
        
        Ok(MultisigInfo {
            multisig_info: result.multisig_info,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_monero_rpc_client_localhost_only() {
        // OPSEC: Vérifier que client rejette URLs publiques
        let result = MoneroRpcClient::new("http://0.0.0.0:18082".to_string());
        assert!(result.is_err());
        
        let result = MoneroRpcClient::new("http://192.168.1.10:18082".to_string());
        assert!(result.is_err());
        
        // Localhost OK
        let result = MoneroRpcClient::new("http://127.0.0.1:18082".to_string());
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_prepare_multisig() {
        // SETUP: monero-wallet-rpc doit tourner sur 18082
        // Voir docs/specs/prepare_multisig.md pour commandes
        
        let client = MoneroRpcClient::new("http://127.0.0.1:18082".to_string())
            .context("Failed to create client for test")?;
        
        // Vérifier connexion d'abord
        match client.check_connection().await {
            Ok(_) => tracing::info!("RPC accessible"),
            Err(e) => {
                tracing::warn!("RPC pas accessible: {}", e);
                tracing::info!("Lance: monero-wallet-rpc --testnet ...");
                return;
            }
        }
        
        // Tester prepare_multisig
        let result = client.prepare_multisig().await;
        
        match &result {
            Ok(info) => {
                tracing::info!("prepare_multisig OK");
                tracing::debug!("Info length: {}", info.multisig_info.len());
                assert!(info.multisig_info.starts_with("MultisigV1"));
                assert!(info.multisig_info.len() > 100);
            }
            Err(MoneroError::AlreadyMultisig) => {
                tracing::warn!("Wallet déjà en multisig (normal si test rejoué)");
                tracing::info!("Pour reset: fermer RPC, supprimer wallet, recréer");
                // Ce n'est pas un échec de test
                return;
            }
            Err(e) => {
                tracing::error!("prepare_multisig échoué: {}", e);
                return Err(anyhow::anyhow!("Test failed: {}", e));
            }
        }
    }
    
    #[tokio::test]
    async fn test_prepare_multisig_rpc_down() {
        // Test avec RPC pas lancé
        let client = MoneroRpcClient::new("http://127.0.0.1:19999".to_string())
            .context("Failed to create client for test")?;
        
        let result = client.prepare_multisig().await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MoneroError::RpcUnreachable));
    }
}