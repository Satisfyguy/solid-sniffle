//! Escrow model and related database operations

use diesel::prelude::*;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use anyhow::{Context, Result};

use crate::schema::escrows;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
#[diesel(table_name = escrows)]
pub struct Escrow {
    pub id: String,
    pub order_id: String,
    pub buyer_id: String,
    pub vendor_id: String,
    pub arbiter_id: String,
    pub amount: i64,
    pub multisig_address: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub buyer_wallet_info: Option<Vec<u8>>,
    pub vendor_wallet_info: Option<Vec<u8>>,
    pub arbiter_wallet_info: Option<Vec<u8>>,
}

#[derive(Insertable)]
#[diesel(table_name = escrows)]
pub struct NewEscrow {
    pub id: String,
    pub order_id: String,
    pub buyer_id: String,
    pub vendor_id: String,
    pub arbiter_id: String,
    pub amount: i64,
    pub status: String,
}

impl Escrow {
    /// Create a new escrow in the database
    pub fn create(conn: &mut SqliteConnection, new_escrow: NewEscrow) -> Result<Escrow> {
        diesel::insert_into(escrows::table)
            .values(&new_escrow)
            .execute(conn)
            .context("Failed to insert escrow")?;

        escrows::table
            .filter(escrows::id.eq(new_escrow.id))
            .first(conn)
            .context("Failed to retrieve created escrow")
    }

    /// Find escrow by ID
    pub fn find_by_id(conn: &mut SqliteConnection, escrow_id: String) -> Result<Escrow> {
        escrows::table
            .filter(escrows::id.eq(escrow_id.clone()))
            .first(conn)
            .context(format!("Escrow with ID {} not found", escrow_id))
    }

    /// Find escrows by buyer ID
    pub fn find_by_buyer(conn: &mut SqliteConnection, buyer_id: String) -> Result<Vec<Escrow>> {
        escrows::table
            .filter(escrows::buyer_id.eq(buyer_id.clone()))
            .load(conn)
            .context(format!("Failed to load escrows for buyer {}", buyer_id))
    }

    /// Find escrows by vendor ID
    pub fn find_by_vendor(conn: &mut SqliteConnection, vendor_id: String) -> Result<Vec<Escrow>> {
        escrows::table
            .filter(escrows::vendor_id.eq(vendor_id.clone()))
            .load(conn)
            .context(format!("Failed to load escrows for vendor {}", vendor_id))
    }

    /// Find escrows by arbiter ID
    pub fn find_by_arbiter(conn: &mut SqliteConnection, arbiter_id: String) -> Result<Vec<Escrow>> {
        escrows::table
            .filter(escrows::arbiter_id.eq(arbiter_id.clone()))
            .load(conn)
            .context(format!("Failed to load escrows for arbiter {}", arbiter_id))
    }

    /// Update escrow status
    pub fn update_status(conn: &mut SqliteConnection, escrow_id: String, new_status: &str) -> Result<()> {
        diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.clone())))
            .set((
                escrows::status.eq(new_status),
                escrows::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)
            .context(format!("Failed to update status for escrow {}", escrow_id))?;
        Ok(())
    }

    /// Update multisig address
    pub fn update_multisig_address(conn: &mut SqliteConnection, escrow_id: String, address: &str) -> Result<()> {
        diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.clone())))
            .set((
                escrows::multisig_address.eq(address),
                escrows::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)
            .context(format!("Failed to update multisig address for escrow {}", escrow_id))?;
        Ok(())
    }

    /// Store encrypted wallet info for a party
    pub fn store_wallet_info(
        conn: &mut SqliteConnection,
        escrow_id: String,
        party: &str,
        encrypted_info: Vec<u8>
    ) -> Result<()> {
        let update_result = match party {
            "buyer" => diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.clone())))
                .set(escrows::buyer_wallet_info.eq(Some(encrypted_info)))
                .execute(conn),
            "vendor" => diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.clone())))
                .set(escrows::vendor_wallet_info.eq(Some(encrypted_info)))
                .execute(conn),
            "arbiter" => diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.clone())))
                .set(escrows::arbiter_wallet_info.eq(Some(encrypted_info)))
                .execute(conn),
            _ => return Err(anyhow::anyhow!("Invalid party: {}", party)),
        };

        update_result.context(format!("Failed to store wallet info for {} in escrow {}", party, escrow_id))?;
        Ok(())
    }

    /// Count how many parties have submitted wallet info
    pub fn count_wallet_infos(conn: &mut SqliteConnection, escrow_id: String) -> Result<usize> {
        let escrow = Self::find_by_id(conn, escrow_id)?;
        let mut count = 0;
        if escrow.buyer_wallet_info.is_some() { count += 1; }
        if escrow.vendor_wallet_info.is_some() { count += 1; }
        if escrow.arbiter_wallet_info.is_some() { count += 1; }
        Ok(count)
    }

    /// Get all wallet infos (returns vec of encrypted data)
    pub fn get_all_wallet_infos(conn: &mut SqliteConnection, escrow_id: String) -> Result<Vec<Vec<u8>>> {
        let escrow = Self::find_by_id(conn, escrow_id)?;
        let mut infos = Vec::new();
        if let Some(buyer_info) = escrow.buyer_wallet_info {
            infos.push(buyer_info);
        }
        if let Some(vendor_info) = escrow.vendor_wallet_info {
            infos.push(vendor_info);
        }
        if let Some(arbiter_info) = escrow.arbiter_wallet_info {
            infos.push(arbiter_info);
        }
        Ok(infos)
    }
}
