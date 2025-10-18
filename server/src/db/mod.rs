//! Database utilities and connection pooling

use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use uuid::Uuid;

use crate::models::escrow::{Escrow, NewEscrow};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

/// Creates a new database connection pool.
pub fn create_pool(database_url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(10) // Max 10 connections
        .build(manager)
        .context("Failed to create R2D2 pool")?;
    Ok(pool)
}

/// Runs database migrations.
pub fn run_migrations(connection: &mut impl MigrationHarness<diesel::sqlite::Sqlite>) -> Result<()> {
    connection.run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;
    Ok(())
}

/// Insert a new escrow into the database
pub async fn db_insert_escrow(pool: &DbPool, new_escrow: NewEscrow) -> Result<Escrow> {
    let mut conn = pool.get().context("Failed to get DB connection")?;

    tokio::task::spawn_blocking(move || {
        Escrow::create(&mut conn, new_escrow)
    })
    .await
    .context("Task join error")?
}

/// Load an escrow from the database by ID
pub async fn db_load_escrow(pool: &DbPool, escrow_id: Uuid) -> Result<Escrow> {
    let mut conn = pool.get().context("Failed to get DB connection")?;

    tokio::task::spawn_blocking(move || {
        Escrow::find_by_id(&mut conn, escrow_id)
    })
    .await
    .context("Task join error")?
}

/// Update escrow's multisig address
pub async fn db_update_escrow_address(pool: &DbPool, escrow_id: Uuid, address: &str) -> Result<()> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let address_owned = address.to_string();

    tokio::task::spawn_blocking(move || {
        Escrow::update_multisig_address(&mut conn, escrow_id, &address_owned)
    })
    .await
    .context("Task join error")?
}

/// Update escrow's status
pub async fn db_update_escrow_status(pool: &DbPool, escrow_id: Uuid, status: &str) -> Result<()> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let status_owned = status.to_string();

    tokio::task::spawn_blocking(move || {
        Escrow::update_status(&mut conn, escrow_id, &status_owned)
    })
    .await
    .context("Task join error")?
}

/// Store encrypted multisig info for a party
pub async fn db_store_multisig_info(
    pool: &DbPool,
    escrow_id: Uuid,
    party: &str,
    encrypted_info: Vec<u8>
) -> Result<()> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let party_owned = party.to_string();

    tokio::task::spawn_blocking(move || {
        Escrow::store_wallet_info(&mut conn, escrow_id, &party_owned, encrypted_info)
    })
    .await
    .context("Task join error")?
}

/// Count how many parties have submitted multisig info
pub async fn db_count_multisig_infos(pool: &DbPool, escrow_id: Uuid) -> Result<usize> {
    let mut conn = pool.get().context("Failed to get DB connection")?;

    tokio::task::spawn_blocking(move || {
        Escrow::count_wallet_infos(&mut conn, escrow_id)
    })
    .await
    .context("Task join error")?
}

/// Load all multisig infos for an escrow
pub async fn db_load_multisig_infos(pool: &DbPool, escrow_id: Uuid) -> Result<Vec<Vec<u8>>> {
    let mut conn = pool.get().context("Failed to get DB connection")?;

    tokio::task::spawn_blocking(move || {
        Escrow::get_all_wallet_infos(&mut conn, escrow_id)
    })
    .await
    .context("Task join error")?
}