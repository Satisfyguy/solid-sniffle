//! End-to-end integration tests for 2-of-3 multisig wallet setup
//!
//! These tests require:
//! 1. A running monerod testnet node
//! 2. Three monero-wallet-rpc instances on ports 18082, 18083, 18084
//!
//! Run: ./scripts/setup-3-wallets-testnet.sh before running these tests
//! Execute: cargo test --package wallet --test multisig_e2e -- --nocapture --test-threads=1

use monero_marketplace_common::types::MoneroConfig;
use monero_marketplace_wallet::{MoneroClient, MoneroRpcClient};

/// Configuration for the 3 test wallets
const WALLET1_RPC_URL: &str = "http://127.0.0.1:18082/json_rpc";
const WALLET2_RPC_URL: &str = "http://127.0.0.1:18083/json_rpc";
const WALLET3_RPC_URL: &str = "http://127.0.0.1:18084/json_rpc";

/// Create a Monero RPC client for a given wallet
fn create_rpc_client(rpc_url: &str) -> MoneroRpcClient {
    let config = MoneroConfig {
        rpc_url: rpc_url.to_string(),
        rpc_user: None,
        rpc_password: None,
        timeout_seconds: 30,
    };
    MoneroRpcClient::new(config).expect("Failed to create RPC client")
}

/// Create a Monero client for a given wallet
fn create_client(rpc_url: &str) -> MoneroClient {
    let config = MoneroConfig {
        rpc_url: rpc_url.to_string(),
        rpc_user: None,
        rpc_password: None,
        timeout_seconds: 30,
    };
    MoneroClient::new(config).expect("Failed to create client")
}

#[tokio::test]
#[ignore] // Requires running wallet RPC servers
async fn test_wallet_connections() {
    println!("\n🧪 Test 1: Verifying wallet connections\n");

    let client1 = create_rpc_client(WALLET1_RPC_URL);
    let client2 = create_rpc_client(WALLET2_RPC_URL);
    let client3 = create_rpc_client(WALLET3_RPC_URL);

    // Test connection to wallet 1
    println!("📡 Testing connection to wallet 1...");
    client1
        .check_connection()
        .await
        .expect("Wallet 1 connection failed");
    println!("✅ Wallet 1 connected");

    // Test connection to wallet 2
    println!("📡 Testing connection to wallet 2...");
    client2
        .check_connection()
        .await
        .expect("Wallet 2 connection failed");
    println!("✅ Wallet 2 connected");

    // Test connection to wallet 3
    println!("📡 Testing connection to wallet 3...");
    client3
        .check_connection()
        .await
        .expect("Wallet 3 connection failed");
    println!("✅ Wallet 3 connected");

    println!("\n✅ All wallet connections successful!\n");
}

#[tokio::test]
#[ignore] // Requires running wallet RPC servers
async fn test_multisig_prepare() {
    println!("\n🧪 Test 2: Multisig Prepare (Step 1/6)\n");

    let client1 = create_rpc_client(WALLET1_RPC_URL);
    let client2 = create_rpc_client(WALLET2_RPC_URL);
    let client3 = create_rpc_client(WALLET3_RPC_URL);

    // Wallet 1: prepare_multisig
    println!("📝 Wallet 1: Preparing multisig...");
    let info1 = client1
        .prepare_multisig()
        .await
        .expect("Wallet 1 prepare_multisig failed");
    println!(
        "✅ Wallet 1 multisig info: {}...",
        &info1.multisig_info[..50]
    );
    assert!(info1.multisig_info.starts_with("MultisigV1"));
    assert!(info1.multisig_info.len() > 100);

    // Wallet 2: prepare_multisig
    println!("📝 Wallet 2: Preparing multisig...");
    let info2 = client2
        .prepare_multisig()
        .await
        .expect("Wallet 2 prepare_multisig failed");
    println!(
        "✅ Wallet 2 multisig info: {}...",
        &info2.multisig_info[..50]
    );
    assert!(info2.multisig_info.starts_with("MultisigV1"));

    // Wallet 3: prepare_multisig
    println!("📝 Wallet 3: Preparing multisig...");
    let info3 = client3
        .prepare_multisig()
        .await
        .expect("Wallet 3 prepare_multisig failed");
    println!(
        "✅ Wallet 3 multisig info: {}...",
        &info3.multisig_info[..50]
    );
    assert!(info3.multisig_info.starts_with("MultisigV1"));

    println!("\n✅ All wallets prepared for multisig!\n");
}

#[tokio::test]
#[ignore] // Requires running wallet RPC servers AND completed prepare step
async fn test_multisig_make() {
    println!("\n🧪 Test 3: Multisig Make (Step 2/6)\n");

    let client1 = create_rpc_client(WALLET1_RPC_URL);
    let client2 = create_rpc_client(WALLET2_RPC_URL);
    let client3 = create_rpc_client(WALLET3_RPC_URL);

    // Step 1: Prepare all wallets
    println!("📝 Preparing all wallets...");
    let info1 = client1
        .prepare_multisig()
        .await
        .expect("Wallet 1 prepare failed");
    let info2 = client2
        .prepare_multisig()
        .await
        .expect("Wallet 2 prepare failed");
    let info3 = client3
        .prepare_multisig()
        .await
        .expect("Wallet 3 prepare failed");
    println!("✅ All wallets prepared\n");

    // Step 2: Make multisig (2-of-3)
    // Wallet 1: receives info from wallet 2 and 3
    println!("🔐 Wallet 1: Making 2-of-3 multisig...");
    let make1 = client1
        .make_multisig(
            2,
            vec![info2.multisig_info.clone(), info3.multisig_info.clone()],
        )
        .await
        .expect("Wallet 1 make_multisig failed");
    println!("✅ Wallet 1 multisig address: {}", make1.address);
    assert!(make1.address.starts_with("5") || make1.address.starts_with("9")); // Testnet addresses

    // Wallet 2: receives info from wallet 1 and 3
    println!("🔐 Wallet 2: Making 2-of-3 multisig...");
    let make2 = client2
        .make_multisig(
            2,
            vec![info1.multisig_info.clone(), info3.multisig_info.clone()],
        )
        .await
        .expect("Wallet 2 make_multisig failed");
    println!("✅ Wallet 2 multisig address: {}", make2.address);

    // Wallet 3: receives info from wallet 1 and 2
    println!("🔐 Wallet 3: Making 2-of-3 multisig...");
    let make3 = client3
        .make_multisig(2, vec![info1.multisig_info, info2.multisig_info])
        .await
        .expect("Wallet 3 make_multisig failed");
    println!("✅ Wallet 3 multisig address: {}", make3.address);

    // Verify all wallets have the same multisig address
    assert_eq!(make1.address, make2.address);
    assert_eq!(make2.address, make3.address);
    println!(
        "\n✅ All wallets share the same multisig address: {}\n",
        make1.address
    );
}

#[tokio::test]
#[ignore] // Requires completed make_multisig step
async fn test_multisig_export_import() {
    println!("\n🧪 Test 4: Multisig Export/Import (Steps 3-6)\n");

    let client1 = create_rpc_client(WALLET1_RPC_URL);
    let client2 = create_rpc_client(WALLET2_RPC_URL);
    let client3 = create_rpc_client(WALLET3_RPC_URL);

    // Round 1: Export
    println!("📤 Round 1: Exporting multisig info from all wallets...");
    let export1_r1 = client1
        .export_multisig_info()
        .await
        .expect("Wallet 1 export failed");
    let export2_r1 = client2
        .export_multisig_info()
        .await
        .expect("Wallet 2 export failed");
    let export3_r1 = client3
        .export_multisig_info()
        .await
        .expect("Wallet 3 export failed");
    println!("✅ Round 1 exports complete\n");

    // Round 1: Import
    println!("📥 Round 1: Importing multisig info...");
    let import1_r1 = client1
        .import_multisig_info(vec![export2_r1.info.clone(), export3_r1.info.clone()])
        .await
        .expect("Wallet 1 import failed");
    println!("✅ Wallet 1: Imported {} outputs", import1_r1.n_outputs);

    let import2_r1 = client2
        .import_multisig_info(vec![export1_r1.info.clone(), export3_r1.info.clone()])
        .await
        .expect("Wallet 2 import failed");
    println!("✅ Wallet 2: Imported {} outputs", import2_r1.n_outputs);

    let import3_r1 = client3
        .import_multisig_info(vec![export1_r1.info, export2_r1.info])
        .await
        .expect("Wallet 3 import failed");
    println!("✅ Wallet 3: Imported {} outputs\n", import3_r1.n_outputs);

    // Round 2: Export
    println!("📤 Round 2: Exporting multisig info from all wallets...");
    let export1_r2 = client1
        .export_multisig_info()
        .await
        .expect("Wallet 1 export R2 failed");
    let export2_r2 = client2
        .export_multisig_info()
        .await
        .expect("Wallet 2 export R2 failed");
    let export3_r2 = client3
        .export_multisig_info()
        .await
        .expect("Wallet 3 export R2 failed");
    println!("✅ Round 2 exports complete\n");

    // Round 2: Import
    println!("📥 Round 2: Importing multisig info...");
    let import1_r2 = client1
        .import_multisig_info(vec![export2_r2.info.clone(), export3_r2.info.clone()])
        .await
        .expect("Wallet 1 import R2 failed");
    println!("✅ Wallet 1: Imported {} outputs", import1_r2.n_outputs);

    let import2_r2 = client2
        .import_multisig_info(vec![export1_r2.info.clone(), export3_r2.info.clone()])
        .await
        .expect("Wallet 2 import R2 failed");
    println!("✅ Wallet 2: Imported {} outputs", import2_r2.n_outputs);

    let import3_r2 = client3
        .import_multisig_info(vec![export1_r2.info, export2_r2.info])
        .await
        .expect("Wallet 3 import R2 failed");
    println!("✅ Wallet 3: Imported {} outputs", import3_r2.n_outputs);

    println!("\n✅ 2-of-3 multisig wallet fully synchronized!\n");
}

#[tokio::test]
#[ignore] // Full end-to-end test
async fn test_complete_multisig_setup() {
    println!("\n🧪 Test 5: Complete 2-of-3 Multisig Setup (Full E2E)\n");
    println!("═══════════════════════════════════════════════════════\n");

    let client1 = create_client(WALLET1_RPC_URL);
    let client2 = create_client(WALLET2_RPC_URL);
    let client3 = create_client(WALLET3_RPC_URL);

    // ━━━ STEP 1: Prepare Multisig ━━━
    println!("📋 STEP 1/6: Prepare Multisig\n");
    let info1 = client1
        .multisig()
        .prepare_multisig()
        .await
        .expect("Wallet 1 prepare failed");
    println!("  ✅ Wallet 1 prepared");

    let info2 = client2
        .multisig()
        .prepare_multisig()
        .await
        .expect("Wallet 2 prepare failed");
    println!("  ✅ Wallet 2 prepared");

    let info3 = client3
        .multisig()
        .prepare_multisig()
        .await
        .expect("Wallet 3 prepare failed");
    println!("  ✅ Wallet 3 prepared\n");

    // ━━━ STEP 2: Make Multisig ━━━
    println!("📋 STEP 2/6: Make Multisig (2-of-3)\n");
    let make1 = client1
        .multisig()
        .make_multisig(
            2,
            vec![info2.multisig_info.clone(), info3.multisig_info.clone()],
        )
        .await
        .expect("Wallet 1 make failed");
    println!("  ✅ Wallet 1: {}", make1.address);

    let make2 = client2
        .multisig()
        .make_multisig(
            2,
            vec![info1.multisig_info.clone(), info3.multisig_info.clone()],
        )
        .await
        .expect("Wallet 2 make failed");
    println!("  ✅ Wallet 2: {}", make2.address);

    let make3 = client3
        .multisig()
        .make_multisig(2, vec![info1.multisig_info, info2.multisig_info])
        .await
        .expect("Wallet 3 make failed");
    println!("  ✅ Wallet 3: {}\n", make3.address);

    assert_eq!(make1.address, make2.address);
    assert_eq!(make2.address, make3.address);
    println!("  🎯 Shared multisig address: {}\n", make1.address);

    // ━━━ ROUND 1: Export/Import ━━━
    println!("📋 ROUND 1: Synchronization (Export → Import)\n");

    println!("  📤 Exporting...");
    let export1_r1 = client1
        .multisig()
        .export_multisig_info()
        .await
        .expect("W1 export R1 failed");
    let export2_r1 = client2
        .multisig()
        .export_multisig_info()
        .await
        .expect("W2 export R1 failed");
    let export3_r1 = client3
        .multisig()
        .export_multisig_info()
        .await
        .expect("W3 export R1 failed");

    println!("  📥 Importing...");
    client1
        .multisig()
        .import_multisig_info(vec![export2_r1.info.clone(), export3_r1.info.clone()])
        .await
        .expect("W1 import R1 failed");
    client2
        .multisig()
        .import_multisig_info(vec![export1_r1.info.clone(), export3_r1.info.clone()])
        .await
        .expect("W2 import R1 failed");
    client3
        .multisig()
        .import_multisig_info(vec![export1_r1.info, export2_r1.info])
        .await
        .expect("W3 import R1 failed");
    println!("  ✅ Round 1 complete\n");

    // ━━━ ROUND 2: Export/Import ━━━
    println!("📋 ROUND 2: Final Synchronization (Export → Import)\n");

    println!("  📤 Exporting...");
    let export1_r2 = client1
        .multisig()
        .export_multisig_info()
        .await
        .expect("W1 export R2 failed");
    let export2_r2 = client2
        .multisig()
        .export_multisig_info()
        .await
        .expect("W2 export R2 failed");
    let export3_r2 = client3
        .multisig()
        .export_multisig_info()
        .await
        .expect("W3 export R2 failed");

    println!("  📥 Importing...");
    client1
        .multisig()
        .import_multisig_info(vec![export2_r2.info.clone(), export3_r2.info.clone()])
        .await
        .expect("W1 import R2 failed");
    client2
        .multisig()
        .import_multisig_info(vec![export1_r2.info.clone(), export3_r2.info.clone()])
        .await
        .expect("W2 import R2 failed");
    client3
        .multisig()
        .import_multisig_info(vec![export1_r2.info, export2_r2.info])
        .await
        .expect("W3 import R2 failed");
    println!("  ✅ Round 2 complete\n");

    // ━━━ VERIFICATION ━━━
    println!("📋 VERIFICATION: Checking multisig status\n");

    let is_multisig1 = client1
        .multisig()
        .is_multisig()
        .await
        .expect("W1 check failed");
    let is_multisig2 = client2
        .multisig()
        .is_multisig()
        .await
        .expect("W2 check failed");
    let is_multisig3 = client3
        .multisig()
        .is_multisig()
        .await
        .expect("W3 check failed");

    assert!(is_multisig1);
    assert!(is_multisig2);
    assert!(is_multisig3);

    println!("  ✅ Wallet 1: multisig = {}", is_multisig1);
    println!("  ✅ Wallet 2: multisig = {}", is_multisig2);
    println!("  ✅ Wallet 3: multisig = {}", is_multisig3);

    println!("\n═══════════════════════════════════════════════════════");
    println!("🎉 SUCCESS: 2-of-3 multisig wallet fully operational!");
    println!("═══════════════════════════════════════════════════════\n");
}
