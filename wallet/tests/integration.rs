//! Integration tests for Monero wallet functionality
//! 
//! These tests require a running Monero testnet environment.

use monero_marketplace_common::{
    error::Result,
    types::MoneroConfig,
    TEST_RPC_URL,
};
use monero_marketplace_wallet::MoneroClient;

/// Test basic RPC connectivity
#[tokio::test]
async fn test_rpc_connectivity() -> Result<()> {
    let config = MoneroConfig::default();
    let client = MoneroClient::new(config)?;
    
    // Test get_version
    let version = client.rpc().get_version().await?;
    assert!(!version.is_empty());
    
    Ok(())
}

/// Test wallet status retrieval
#[tokio::test]
async fn test_get_wallet_status() -> Result<()> {
    let config = MoneroConfig::default();
    let client = MoneroClient::new(config)?;
    
    let status = client.get_wallet_status().await?;
    
    // Basic assertions
    assert!(status.balance >= 0);
    assert!(status.unlocked_balance >= 0);
    assert!(status.unlocked_balance <= status.balance);
    
    Ok(())
}

/// Test complete wallet info retrieval
#[tokio::test]
async fn test_get_wallet_info() -> Result<()> {
    let config = MoneroConfig::default();
    let client = MoneroClient::new(config)?;
    
    let info = client.get_wallet_info().await?;
    
    // Basic assertions
    assert!(!info.version.is_empty());
    assert!(info.balance >= 0);
    assert!(info.unlocked_balance >= 0);
    assert!(info.unlocked_balance <= info.balance);
    assert!(info.block_height >= 0);
    assert!(info.daemon_block_height >= 0);
    
    Ok(())
}

/// Test multisig operations (requires proper setup)
#[tokio::test]
async fn test_multisig_operations() -> Result<()> {
    let config = MoneroConfig::default();
    let client = MoneroClient::new(config)?;
    
    // Test is_multisig
    let is_multisig = client.multisig().is_multisig().await?;
    
    // If not multisig, test prepare_multisig
    if !is_multisig {
        let info = client.multisig().prepare_multisig().await?;
        assert!(!info.info.is_empty());
    }
    
    Ok(())
}

/// Test error handling with invalid configuration
#[tokio::test]
async fn test_error_handling() {
    let config = MoneroConfig {
        rpc_url: TEST_RPC_URL.to_string(),
        timeout_seconds: 1,
        ..Default::default()
    };
    
    let client = MoneroClient::new(config)
        .expect("Failed to create client for integration test");
    let result = client.get_wallet_info().await;
    
    // Expected to fail with network error (no RPC running)
    assert!(result.is_err());
}

/// Test CLI integration
#[tokio::test]
async fn test_cli_integration() -> Result<()> {
    // This test would run the CLI and verify output
    // For now, just test that the client can be created
    let config = MoneroConfig::default();
    let _client = MoneroClient::new(config)?;
    
    Ok(())
}
