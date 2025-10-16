//! CLI Test Tool pour tests manuels
//!
//! Outil simple pour tester les fonctionnalités refactorées

use anyhow::Result;
use monero_marketplace_common::{error::MoneroError, types::MoneroConfig, MONERO_RPC_URL};
use monero_marketplace_wallet::rpc::MoneroRpcClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt().init();

    println!("🧅 Monero Marketplace - CLI Test Tool v2.0");
    println!("==========================================\n");

    // Test 1: Création Client RPC
    println!("1️⃣ Testing RPC Client creation...");
    let config = MoneroConfig {
        rpc_url: MONERO_RPC_URL.to_string(),
        rpc_user: None,
        rpc_password: None,
        timeout_seconds: 30,
    };

    let client = match MoneroRpcClient::new(config) {
        Ok(client) => {
            println!("   ✅ RPC Client created successfully");
            client
        }
        Err(e) => {
            println!("   ❌ RPC Client creation failed: {}", e);
            return Ok(());
        }
    };

    println!();

    // Test 2: Vérification Connexion
    println!("2️⃣ Testing RPC connection...");
    match client.check_connection().await {
        Ok(_) => {
            println!("   ✅ RPC connection successful");
        }
        Err(e) => {
            println!("   ❌ RPC connection failed: {}", e);
            println!("   💡 Launch wallet RPC: monero-wallet-rpc --testnet ...");
            return Ok(());
        }
    }

    println!();

    // Test 3: prepare_multisig
    println!("3️⃣ Testing prepare_multisig...");

    match client.prepare_multisig().await {
        Ok(info) => {
            println!("   ✅ prepare_multisig succeeded");
            println!("   Info: {}...", &info.multisig_info[..50]);
            println!("   Length: {} chars", info.multisig_info.len());

            // Validation
            if info.multisig_info.starts_with("MultisigV1") {
                println!("   ✅ Validation passed (prefix OK)");
            } else {
                println!("   ⚠️ Validation warning: Invalid prefix");
            }
        }
        Err(MoneroError::AlreadyMultisig) => {
            println!("   ⚠️ Wallet already in multisig mode (normal if test replayed)");
            println!("   💡 To reset: close RPC, delete wallet, recreate");
        }
        Err(e) => {
            println!("   ❌ prepare_multisig failed: {}", e);
        }
    }

    println!();

    // Test 4: Appels Concurrents
    println!("4️⃣ Testing concurrent calls...");
    let client_arc = std::sync::Arc::new(client);
    let handles: Vec<_> = (0..3)
        .map(|i| {
            let client = std::sync::Arc::clone(&client_arc);
            tokio::spawn(async move {
                match client.check_connection().await {
                    Ok(_) => format!("Task {}: ✅ Success", i + 1),
                    Err(e) => format!("Task {}: ❌ Failed: {}", i + 1, e),
                }
            })
        })
        .collect();

    for handle in handles {
        match handle.await {
            Ok(result) => println!("   {}", result),
            Err(e) => println!("   ❌ Task failed: {}", e),
        }
    }

    println!();
    println!("✅ All tests completed");
    println!();
    println!("📊 Summary:");
    println!("   - RPC Client: Thread-safe with Mutex + Semaphore");
    println!("   - Retry Logic: Backoff exponential implemented");
    println!("   - Validation: Stricte multisig_info validation");
    println!("   - Timeouts: Configurable via MONERO_RPC_TIMEOUT_SECS");
    println!("   - Logging: Structured with tracing");

    Ok(())
}
