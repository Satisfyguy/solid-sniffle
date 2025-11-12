use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, CustomizeConnection};
use diesel::sql_query;
use uuid::Uuid;

use crate::models::escrow::{Escrow, NewEscrow};
use crate::models::transaction::{NewTransaction, Transaction};
use crate::schema::escrows;
use monero_marketplace_common::types::MultisigInfo;

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

/// Custom connection customizer that sets the SQLCipher encryption key
#[derive(Debug, Clone)]
struct SqlCipherConnectionCustomizer {
    encryption_key: String,
}

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for SqlCipherConnectionCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        // Set SQLCipher key using raw SQL
        sql_query(format!("PRAGMA key = '{}';", self.encryption_key))
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        // Configure SQLite for concurrent access and corruption prevention
        // CRITICAL: These settings prevent database corruption
        sql_query("PRAGMA journal_mode = WAL;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        sql_query("PRAGMA busy_timeout = 5000;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        sql_query("PRAGMA synchronous = NORMAL;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        sql_query("PRAGMA cache_size = -64000;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        sql_query("PRAGMA temp_store = MEMORY;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        // Verify encryption is working by checking we can read from sqlite_master
        sql_query("SELECT count(*) FROM sqlite_master;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        Ok(())
    }
}

/// Create a database connection pool with SQLCipher encryption
///
/// # Arguments
/// * `database_url` - Path to the SQLite database file
/// * `encryption_key` - Encryption key for SQLCipher (must be non-empty for production)
///
/// # Security
/// - Uses SQLCipher for at-rest encryption
/// - Key is applied to every connection in the pool
/// - Empty keys are rejected in production builds
pub fn create_pool(database_url: &str, encryption_key: &str) -> Result<DbPool> {
    // In production, require non-empty encryption key
    #[cfg(not(debug_assertions))]
    {
        if encryption_key.is_empty() {
            anyhow::bail!("Encryption key cannot be empty in production mode");
        }
    }

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let customizer = SqlCipherConnectionCustomizer {
        encryption_key: encryption_key.to_string(),
    };

    let pool = r2d2::Pool::builder()
        .max_size(10)
        .connection_customizer(Box::new(customizer))
        .build(manager)
        .context("Failed to create database connection pool")?;

    Ok(pool)
}

pub async fn db_insert_escrow(pool: &DbPool, new_escrow: NewEscrow) -> Result<Escrow> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let escrow_id = new_escrow.id.to_string();
    tokio::task::spawn_blocking(move || {
        diesel::insert_into(escrows::table)
            .values(&new_escrow)
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("Database insert error for escrow {}: {:?}", escrow_id, e);
                anyhow::anyhow!("Failed to insert escrow: {}", e)
            })?;

        escrows::table
            .filter(escrows::id.eq(escrow_id.clone()))
            .first(&mut conn)
            .map_err(|e| {
                tracing::error!("Failed to retrieve escrow {} after insert: {:?}", escrow_id, e);
                anyhow::anyhow!("Failed to retrieve created escrow: {}", e)
            })
    })
    .await?
}

pub async fn db_load_escrow(pool: &DbPool, escrow_id: Uuid) -> Result<Escrow> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    tokio::task::spawn_blocking(move || {
        escrows::table
            .filter(escrows::id.eq(escrow_id.to_string()))
            .first(&mut conn)
            .context(format!("Escrow with ID {} not found", escrow_id))
    })
    .await?
}

pub async fn db_update_escrow_address(pool: &DbPool, escrow_id: Uuid, address: &str) -> Result<()> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let address_clone = address.to_string();
    let _ = tokio::task::spawn_blocking(move || {
        diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.to_string())))
            .set(escrows::multisig_address.eq(address_clone))
            .execute(&mut conn)
            .context(format!("Failed to update escrow {} address", escrow_id))
    })
    .await?;
    Ok(())
}

pub async fn db_update_escrow_status(pool: &DbPool, escrow_id: Uuid, status: &str) -> Result<()> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let status_clone = status.to_string();
    let _ = tokio::task::spawn_blocking(move || {
        diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.to_string())))
            .set(escrows::status.eq(status_clone))
            .execute(&mut conn)
            .context(format!("Failed to update escrow {} status", escrow_id))
    })
    .await?;
    Ok(())
}

pub async fn db_update_escrow_transaction_hash(
    pool: &DbPool,
    escrow_id: Uuid,
    tx_hash: &str,
) -> Result<()> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let tx_hash_clone = tx_hash.to_string();
    let _ = tokio::task::spawn_blocking(move || {
        diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.to_string())))
            .set(escrows::transaction_hash.eq(tx_hash_clone))
            .execute(&mut conn)
            .context(format!(
                "Failed to update escrow {} transaction_hash",
                escrow_id
            ))
    })
    .await?;
    Ok(())
}

pub async fn db_store_multisig_info(
    pool: &DbPool,
    escrow_id: Uuid,
    party: &str,
    info: Vec<u8>,
) -> Result<()> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let info_clone = info.clone();
    let party_clone = party.to_string();
    tokio::task::spawn_blocking(move || {
        match party_clone.as_str() {
            "buyer" => diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.to_string())))
                .set(escrows::buyer_wallet_info.eq(info_clone))
                .execute(&mut conn),
            "vendor" => {
                diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.to_string())))
                    .set(escrows::vendor_wallet_info.eq(info_clone))
                    .execute(&mut conn)
            }
            "arbiter" => {
                diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.to_string())))
                    .set(escrows::arbiter_wallet_info.eq(info_clone))
                    .execute(&mut conn)
            }
            _ => return Err(anyhow::anyhow!("Invalid party for multisig info")),
        }
        .context(format!(
            "Failed to store multisig info for escrow {} party {}",
            escrow_id, party_clone
        ))
    })
    .await??;
    Ok(())
}

pub async fn db_count_multisig_infos(pool: &DbPool, escrow_id: Uuid) -> Result<i64> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    tokio::task::spawn_blocking(move || {
        let escrow = escrows::table
            .filter(escrows::id.eq(escrow_id.to_string()))
            .first::<Escrow>(&mut conn)
            .context(format!("Escrow with ID {} not found", escrow_id))?;

        let mut count = 0;
        if escrow.buyer_wallet_info.is_some() {
            count += 1;
        }
        if escrow.vendor_wallet_info.is_some() {
            count += 1;
        }
        if escrow.arbiter_wallet_info.is_some() {
            count += 1;
        }
        Ok(count)
    })
    .await?
}

pub async fn db_load_multisig_infos(pool: &DbPool, escrow_id: Uuid) -> Result<Vec<MultisigInfo>> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    tokio::task::spawn_blocking(move || {
        let escrow = escrows::table
            .filter(escrows::id.eq(escrow_id.to_string()))
            .first::<Escrow>(&mut conn)
            .context(format!("Escrow with ID {} not found", escrow_id))?;

        let mut infos = Vec::new();
        if let Some(info) = escrow.buyer_wallet_info {
            infos.push(MultisigInfo {
                multisig_info: String::from_utf8(info)?,
            });
        }
        if let Some(info) = escrow.vendor_wallet_info {
            infos.push(MultisigInfo {
                multisig_info: String::from_utf8(info)?,
            });
        }
        if let Some(info) = escrow.arbiter_wallet_info {
            infos.push(MultisigInfo {
                multisig_info: String::from_utf8(info)?,
            });
        }
        Ok(infos)
    })
    .await?
}

// ============================================================================
// Transaction Database Operations
// ============================================================================

/// Create a new transaction record in the database
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `new_transaction` - New transaction data
///
/// # Returns
///
/// The created transaction with timestamp populated
///
/// # Errors
///
/// Returns error if database insertion or retrieval fails
pub async fn db_create_transaction(
    pool: &DbPool,
    new_transaction: NewTransaction,
) -> Result<Transaction> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    tokio::task::spawn_blocking(move || Transaction::create(&mut conn, new_transaction)).await?
}

/// Find transaction by ID
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `transaction_id` - Transaction UUID string
///
/// # Errors
///
/// Returns error if transaction not found or database query fails
pub async fn db_find_transaction(pool: &DbPool, transaction_id: String) -> Result<Transaction> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    tokio::task::spawn_blocking(move || Transaction::find_by_id(&mut conn, transaction_id)).await?
}

/// Find transaction by Monero transaction hash
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `tx_hash` - Monero transaction hash (64 hex characters)
///
/// # Errors
///
/// Returns error if transaction not found or database query fails
pub async fn db_find_transaction_by_hash(pool: &DbPool, tx_hash: String) -> Result<Transaction> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    tokio::task::spawn_blocking(move || Transaction::find_by_tx_hash(&mut conn, &tx_hash)).await?
}

/// Find all transactions for a specific escrow
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `escrow_id` - Escrow UUID string
///
/// # Returns
///
/// Vector of transactions ordered by creation time (oldest first)
///
/// # Errors
///
/// Returns error if database query fails
pub async fn db_find_transactions_by_escrow(
    pool: &DbPool,
    escrow_id: String,
) -> Result<Vec<Transaction>> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    tokio::task::spawn_blocking(move || Transaction::find_by_escrow(&mut conn, escrow_id)).await?
}

/// Update transaction confirmation count
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `transaction_id` - Transaction UUID string
/// * `confirmations` - New confirmation count
///
/// # Returns
///
/// Updated transaction
///
/// # Errors
///
/// Returns error if transaction not found or database update fails
pub async fn db_update_transaction_confirmations(
    pool: &DbPool,
    transaction_id: String,
    confirmations: i32,
) -> Result<Transaction> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    tokio::task::spawn_blocking(move || {
        Transaction::update_confirmations(&mut conn, transaction_id, confirmations)
    })
    .await?
}

/// Set transaction hash for a transaction
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `transaction_id` - Transaction UUID string
/// * `tx_hash` - Monero transaction hash
///
/// # Returns
///
/// Updated transaction
///
/// # Errors
///
/// Returns error if:
/// - Transaction not found
/// - Transaction already has a hash set
/// - Database update fails
pub async fn db_set_transaction_hash(
    pool: &DbPool,
    transaction_id: String,
    tx_hash: String,
) -> Result<Transaction> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    tokio::task::spawn_blocking(move || {
        Transaction::set_tx_hash(&mut conn, transaction_id, tx_hash)
    })
    .await?
}

/// Find all unconfirmed transactions (confirmations < 10)
///
/// # Arguments
///
/// * `pool` - Database connection pool
///
/// # Returns
///
/// Vector of unconfirmed transactions
///
/// # Errors
///
/// Returns error if database query fails
pub async fn db_find_unconfirmed_transactions(pool: &DbPool) -> Result<Vec<Transaction>> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    tokio::task::spawn_blocking(move || Transaction::find_unconfirmed(&mut conn)).await?
}

/// Find all confirmed transactions (confirmations >= 10)
///
/// # Arguments
///
/// * `pool` - Database connection pool
///
/// # Returns
///
/// Vector of confirmed transactions
///
/// # Errors
///
/// Returns error if database query fails
pub async fn db_find_confirmed_transactions(pool: &DbPool) -> Result<Vec<Transaction>> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    tokio::task::spawn_blocking(move || Transaction::find_confirmed(&mut conn)).await?
}

/// Calculate total transaction amount for an escrow
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `escrow_id` - Escrow UUID string
///
/// # Returns
///
/// Total amount in atomic units (piconeros)
///
/// # Errors
///
/// Returns error if database query fails
pub async fn db_transaction_total_for_escrow(pool: &DbPool, escrow_id: String) -> Result<i64> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    tokio::task::spawn_blocking(move || Transaction::total_amount_for_escrow(&mut conn, escrow_id))
        .await?
}

// ============================================================================
// Reputation Database Operations
// ============================================================================

pub mod reputation;
