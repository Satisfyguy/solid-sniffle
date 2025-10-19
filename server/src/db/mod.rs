use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, CustomizeConnection};
use diesel::sql_query;
use anyhow::{Context, Result};
use uuid::Uuid;

use crate::models::escrow::{Escrow, NewEscrow};
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
            .map_err(|e| diesel::r2d2::Error::QueryError(e))?;

        // Verify encryption is working by checking we can read from sqlite_master
        sql_query("SELECT count(*) FROM sqlite_master;")
            .execute(conn)
            .map_err(|e| diesel::r2d2::Error::QueryError(e))?;

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
    tokio::task::spawn_blocking(move || {
        diesel::insert_into(escrows::table)
            .values(&new_escrow)
            .execute(&mut conn)
            .context("Failed to insert escrow")?;

        escrows::table
            .filter(escrows::id.eq(new_escrow.id.to_string()))
            .first(&mut conn)
            .context("Failed to retrieve created escrow")
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
    tokio::task::spawn_blocking(move || {
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
    tokio::task::spawn_blocking(move || {
        diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.to_string())))
            .set(escrows::status.eq(status_clone))
            .execute(&mut conn)
            .context(format!("Failed to update escrow {} status", escrow_id))
    })
    .await?;
    Ok(())
}

pub async fn db_store_multisig_info(pool: &DbPool, escrow_id: Uuid, party: &str, info: Vec<u8>) -> Result<()> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let info_clone = info.clone();
    let party_clone = party.to_string();
    tokio::task::spawn_blocking(move || {
        match party_clone.as_str() {
            "buyer" => diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.to_string())))
                .set(escrows::buyer_wallet_info.eq(info_clone))
                .execute(&mut conn),
            "vendor" => diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.to_string())))
                .set(escrows::vendor_wallet_info.eq(info_clone))
                .execute(&mut conn),
            "arbiter" => diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.to_string())))
                .set(escrows::arbiter_wallet_info.eq(info_clone))
                .execute(&mut conn),
            _ => return Err(anyhow::anyhow!("Invalid party for multisig info")),
        }
        .context(format!("Failed to store multisig info for escrow {} party {}", escrow_id, party_clone))
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
            infos.push(MultisigInfo { multisig_info: String::from_utf8(info)? });
        }
        if let Some(info) = escrow.vendor_wallet_info {
            infos.push(MultisigInfo { multisig_info: String::from_utf8(info)? });
        }
        if let Some(info) = escrow.arbiter_wallet_info {
            infos.push(MultisigInfo { multisig_info: String::from_utf8(info)? });
        }
        Ok(infos)
    })
    .await?
}