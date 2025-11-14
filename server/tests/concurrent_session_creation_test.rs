//! Concurrent Session Creation Test
//!
//! Verifies that the WalletSessionManager correctly handles concurrent
//! session creation requests without creating duplicate sessions or
//! leaking resources.
//!
//! **Test Scenario:**
//! - 10 concurrent tasks request session for same escrow_id
//! - Expected: Only 1 session created (not 10)
//! - Expected: No resource leaks
//! - Expected: No race condition warnings

use anyhow::Result;
use monero_marketplace_server::services::wallet_session_manager::WalletSessionManager;
use monero_marketplace_server::wallet_pool::WalletPool;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio;
use uuid::Uuid;

/// Test concurrent session creation with same escrow_id
///
/// This test verifies the double-checked locking fix for the session
/// creation race condition documented in MULTISIG-ANALYSIS.md Section 5.3.
#[tokio::test]
#[ignore] // Requires Monero RPC to be running
async fn test_concurrent_session_creation_same_escrow() -> Result<()> {
    // Setup
    let wallet_dir = std::env::var("WALLET_DIR")
        .unwrap_or_else(|_| "test_wallets".to_string());

    // Use 10 RPC ports for this test (18082-18091)
    let rpc_ports: Vec<u16> = (18082..18092).collect();

    let wallet_pool = Arc::new(WalletPool::new(rpc_ports, PathBuf::from(wallet_dir)));
    let session_manager = WalletSessionManager::new(Arc::clone(&wallet_pool));

    // Test: 10 concurrent requests for same escrow_id
    let escrow_id = Uuid::new_v4();
    let mut tasks = vec![];

    for i in 0..10 {
        let session_manager_clone = session_manager.clone();
        let task = tokio::spawn(async move {
            println!("Task {} starting session creation request", i);
            let start = std::time::Instant::now();
            let result = session_manager_clone.get_or_create_session(escrow_id).await;
            let duration = start.elapsed();
            println!("Task {} completed in {:?}", i, duration);
            result
        });
        tasks.push(task);
    }

    // Wait for all tasks to complete
    let mut success_count = 0;
    let mut error_count = 0;

    for (i, task) in tasks.into_iter().enumerate() {
        match task.await {
            Ok(Ok(returned_escrow_id)) => {
                assert_eq!(
                    returned_escrow_id, escrow_id,
                    "Task {} returned wrong escrow_id",
                    i
                );
                success_count += 1;
            }
            Ok(Err(e)) => {
                eprintln!("Task {} failed: {:?}", i, e);
                error_count += 1;
            }
            Err(e) => {
                eprintln!("Task {} panicked: {:?}", i, e);
                error_count += 1;
            }
        }
    }

    // Verify results
    assert_eq!(
        success_count, 10,
        "Expected all 10 tasks to succeed, got {} successes and {} errors",
        success_count, error_count
    );

    // Verify only 1 session was created
    let stats = session_manager.get_stats().await;
    assert_eq!(
        stats.active_sessions, 1,
        "Expected exactly 1 session, found {}. Race condition detected!",
        stats.active_sessions
    );

    println!("✅ Test passed: 10 concurrent requests created only 1 session");

    // Cleanup
    session_manager.close_session(escrow_id).await?;

    Ok(())
}

/// Test concurrent session creation with different escrow_ids
///
/// Verifies that concurrent sessions for different escrows can be created
/// in parallel without blocking each other.
#[tokio::test]
#[ignore] // Requires Monero RPC to be running
async fn test_concurrent_session_creation_different_escrows() -> Result<()> {
    // Setup
    let wallet_dir = std::env::var("WALLET_DIR")
        .unwrap_or_else(|_| "test_wallets".to_string());

    // Use 20 RPC ports (5 escrows × 3 wallets each + buffer)
    let rpc_ports: Vec<u16> = (18082..18102).collect();

    let wallet_pool = Arc::new(WalletPool::new(rpc_ports, PathBuf::from(wallet_dir)));
    let session_manager = WalletSessionManager::new_with_config(
        Arc::clone(&wallet_pool),
        10, // max_active_sessions
        Duration::from_secs(7200),
    );

    // Test: Create 5 concurrent sessions for different escrows
    let mut tasks = vec![];
    let mut escrow_ids = vec![];

    for i in 0..5 {
        let escrow_id = Uuid::new_v4();
        escrow_ids.push(escrow_id);

        let session_manager_clone = session_manager.clone();
        let task = tokio::spawn(async move {
            println!("Creating session for escrow {} (task {})", escrow_id, i);
            let start = std::time::Instant::now();
            let result = session_manager_clone.get_or_create_session(escrow_id).await;
            let duration = start.elapsed();
            println!("Session for escrow {} created in {:?}", escrow_id, duration);
            (escrow_id, result)
        });
        tasks.push(task);
    }

    // Wait for all tasks to complete
    let mut created_sessions = vec![];

    for task in tasks {
        match task.await {
            Ok((escrow_id, Ok(returned_escrow_id))) => {
                assert_eq!(returned_escrow_id, escrow_id, "Escrow ID mismatch");
                created_sessions.push(escrow_id);
            }
            Ok((escrow_id, Err(e))) => {
                panic!("Failed to create session for escrow {}: {:?}", escrow_id, e);
            }
            Err(e) => {
                panic!("Task panicked: {:?}", e);
            }
        }
    }

    // Verify 5 distinct sessions were created
    assert_eq!(
        created_sessions.len(),
        5,
        "Expected 5 sessions to be created"
    );

    let stats = session_manager.get_stats().await;
    assert_eq!(
        stats.active_sessions, 5,
        "Expected exactly 5 active sessions, found {}",
        stats.active_sessions
    );

    println!("✅ Test passed: 5 concurrent sessions created successfully");

    // Cleanup
    for escrow_id in escrow_ids {
        session_manager.close_session(escrow_id).await?;
    }

    Ok(())
}

/// Test session reuse (no duplicate creation on second request)
#[tokio::test]
#[ignore] // Requires Monero RPC to be running
async fn test_session_reuse_no_duplicate() -> Result<()> {
    // Setup
    let wallet_dir = std::env::var("WALLET_DIR")
        .unwrap_or_else(|_| "test_wallets".to_string());

    let rpc_ports: Vec<u16> = (18082..18092).collect();

    let wallet_pool = Arc::new(WalletPool::new(rpc_ports, PathBuf::from(wallet_dir)));
    let session_manager = WalletSessionManager::new(Arc::clone(&wallet_pool));

    let escrow_id = Uuid::new_v4();

    // First request: creates session
    let result1 = session_manager.get_or_create_session(escrow_id).await?;
    assert_eq!(result1, escrow_id);

    let stats = session_manager.get_stats().await;
    assert_eq!(stats.active_sessions, 1, "Expected 1 session after first request");

    // Second request: reuses existing session
    let result2 = session_manager.get_or_create_session(escrow_id).await?;
    assert_eq!(result2, escrow_id);

    let stats = session_manager.get_stats().await;
    assert_eq!(
        stats.active_sessions, 1,
        "Expected still 1 session after second request (reused, not duplicated)"
    );

    println!("✅ Test passed: Session reused correctly, no duplicate created");

    // Cleanup
    session_manager.close_session(escrow_id).await?;

    Ok(())
}

/// Test LRU eviction when session limit reached
#[tokio::test]
#[ignore] // Requires Monero RPC to be running
async fn test_lru_eviction_on_limit() -> Result<()> {
    // Setup with max 3 sessions
    let wallet_dir = std::env::var("WALLET_DIR")
        .unwrap_or_else(|_| "test_wallets".to_string());

    // Need at least 12 ports for 4 escrows × 3 wallets
    let rpc_ports: Vec<u16> = (18082..18094).collect();

    let wallet_pool = Arc::new(WalletPool::new(rpc_ports, PathBuf::from(wallet_dir)));
    let session_manager = WalletSessionManager::new_with_config(
        Arc::clone(&wallet_pool),
        3, // max_active_sessions
        Duration::from_secs(7200),
    );

    // Create 3 sessions (fill up to limit)
    let escrow_id_1 = Uuid::new_v4();
    let escrow_id_2 = Uuid::new_v4();
    let escrow_id_3 = Uuid::new_v4();

    session_manager.get_or_create_session(escrow_id_1).await?;
    tokio::time::sleep(Duration::from_millis(100)).await; // Ensure different timestamps

    session_manager.get_or_create_session(escrow_id_2).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;

    session_manager.get_or_create_session(escrow_id_3).await?;

    let stats = session_manager.get_stats().await;
    assert_eq!(stats.active_sessions, 3, "Expected 3 sessions (at limit)");

    // Create 4th session - should evict LRU (escrow_id_1)
    let escrow_id_4 = Uuid::new_v4();
    session_manager.get_or_create_session(escrow_id_4).await?;

    let stats = session_manager.get_stats().await;
    assert_eq!(
        stats.active_sessions, 3,
        "Expected still 3 sessions after eviction"
    );

    println!("✅ Test passed: LRU eviction works correctly when limit reached");

    // Cleanup
    session_manager.close_session(escrow_id_2).await?;
    session_manager.close_session(escrow_id_3).await?;
    session_manager.close_session(escrow_id_4).await?;

    Ok(())
}
