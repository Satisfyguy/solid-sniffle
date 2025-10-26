//! Escrow model and related database operations

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub transaction_hash: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub last_activity_at: NaiveDateTime,
    pub multisig_phase: String,
    pub multisig_state_json: Option<String>,
    pub multisig_updated_at: i32,
    pub recovery_mode: String,
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
    pub fn update_status(
        conn: &mut SqliteConnection,
        escrow_id: String,
        new_status: &str,
    ) -> Result<()> {
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
    pub fn update_multisig_address(
        conn: &mut SqliteConnection,
        escrow_id: String,
        address: &str,
    ) -> Result<()> {
        diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.clone())))
            .set((
                escrows::multisig_address.eq(address),
                escrows::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)
            .context(format!(
                "Failed to update multisig address for escrow {}",
                escrow_id
            ))?;
        Ok(())
    }

    /// Store encrypted wallet info for a party
    pub fn store_wallet_info(
        conn: &mut SqliteConnection,
        escrow_id: String,
        party: &str,
        encrypted_info: Vec<u8>,
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

        update_result.context(format!(
            "Failed to store wallet info for {} in escrow {}",
            party, escrow_id
        ))?;
        Ok(())
    }

    /// Count how many parties have submitted wallet info
    pub fn count_wallet_infos(conn: &mut SqliteConnection, escrow_id: String) -> Result<usize> {
        let escrow = Self::find_by_id(conn, escrow_id)?;
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
    }

    /// Get all wallet infos (returns vec of encrypted data)
    pub fn get_all_wallet_infos(
        conn: &mut SqliteConnection,
        escrow_id: String,
    ) -> Result<Vec<Vec<u8>>> {
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

    /// Update transaction hash for release/refund transaction
    ///
    /// This is called when funds are released to vendor or refunded to buyer.
    /// The transaction_hash is used by the blockchain monitor to track confirmations.
    pub fn update_transaction_hash(
        conn: &mut SqliteConnection,
        escrow_id: String,
        tx_hash: &str,
    ) -> Result<()> {
        diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.clone())))
            .set((
                escrows::transaction_hash.eq(tx_hash),
                escrows::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)
            .context(format!(
                "Failed to update transaction_hash for escrow {}",
                escrow_id
            ))?;
        Ok(())
    }

    /// Update last_activity_at timestamp to current time
    ///
    /// Should be called whenever there's a significant action on an escrow:
    /// - Status change
    /// - Multisig setup step completed
    /// - Funds deposited
    /// - Dispute initiated/resolved
    ///
    /// This resets the timeout clock for the current status.
    pub fn update_activity(
        conn: &mut SqliteConnection,
        escrow_id: String,
    ) -> Result<()> {
        diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.clone())))
            .set((
                escrows::last_activity_at.eq(diesel::dsl::now),
                escrows::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)
            .context(format!(
                "Failed to update last_activity_at for escrow {}",
                escrow_id
            ))?;
        Ok(())
    }

    /// Update expires_at deadline for the current escrow status
    ///
    /// Called after status changes or activity updates to set the new deadline.
    /// Pass None to clear expiration (for terminal states like completed/refunded).
    ///
    /// # Arguments
    /// * `conn` - Database connection
    /// * `escrow_id` - Escrow ID to update
    /// * `new_expires_at` - New expiration timestamp, or None for no expiration
    pub fn update_expiration(
        conn: &mut SqliteConnection,
        escrow_id: String,
        new_expires_at: Option<NaiveDateTime>,
    ) -> Result<()> {
        diesel::update(escrows::table.filter(escrows::id.eq(escrow_id.clone())))
            .set((
                escrows::expires_at.eq(new_expires_at),
                escrows::updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)
            .context(format!(
                "Failed to update expires_at for escrow {}",
                escrow_id
            ))?;
        Ok(())
    }

    /// Check if escrow has expired (deadline passed)
    ///
    /// Returns true if expires_at is set and is in the past.
    /// Returns false if expires_at is None (terminal states) or in the future.
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < chrono::Utc::now().naive_utc()
        } else {
            false
        }
    }

    /// Get seconds remaining until expiration
    ///
    /// Returns None if expires_at is not set (terminal states).
    /// Returns Some(0) if already expired.
    /// Returns Some(n) with seconds remaining otherwise.
    pub fn seconds_until_expiration(&self) -> Option<i64> {
        self.expires_at.map(|expires_at| {
            let now = chrono::Utc::now().naive_utc();
            let duration = expires_at.signed_duration_since(now);
            duration.num_seconds().max(0)
        })
    }

    /// Check if escrow is approaching expiration (within warning threshold)
    ///
    /// # Arguments
    /// * `warning_threshold_secs` - How many seconds before expiration to warn (default 3600 = 1h)
    ///
    /// Returns true if expiration is within the threshold but not yet expired.
    pub fn is_expiring_soon(&self, warning_threshold_secs: i64) -> bool {
        if let Some(secs_remaining) = self.seconds_until_expiration() {
            secs_remaining > 0 && secs_remaining <= warning_threshold_secs
        } else {
            false
        }
    }

    /// Get all escrows that have expired (past their deadline)
    ///
    /// Returns escrows where:
    /// - expires_at IS NOT NULL
    /// - expires_at < NOW()
    /// - status is not a terminal state
    ///
    /// Used by TimeoutMonitor to find escrows needing timeout handling.
    pub fn find_expired(conn: &mut SqliteConnection) -> Result<Vec<Escrow>> {
        let now = chrono::Utc::now().naive_utc();

        escrows::table
            .filter(escrows::expires_at.is_not_null())
            .filter(escrows::expires_at.lt(now))
            .filter(escrows::status.ne("completed"))
            .filter(escrows::status.ne("refunded"))
            .filter(escrows::status.ne("cancelled"))
            .filter(escrows::status.ne("expired"))
            .load(conn)
            .context("Failed to load expired escrows")
    }

    /// Get all escrows approaching expiration (within warning threshold)
    ///
    /// Returns escrows where:
    /// - expires_at IS NOT NULL
    /// - expires_at is between NOW() and NOW() + warning_threshold
    /// - status is not a terminal state
    ///
    /// Used by TimeoutMonitor to send warning notifications.
    pub fn find_expiring_soon(
        conn: &mut SqliteConnection,
        warning_threshold_secs: i64,
    ) -> Result<Vec<Escrow>> {
        let now = chrono::Utc::now().naive_utc();
        let warning_time = now + chrono::Duration::seconds(warning_threshold_secs);

        escrows::table
            .filter(escrows::expires_at.is_not_null())
            .filter(escrows::expires_at.gt(now))
            .filter(escrows::expires_at.le(warning_time))
            .filter(escrows::status.ne("completed"))
            .filter(escrows::status.ne("refunded"))
            .filter(escrows::status.ne("cancelled"))
            .filter(escrows::status.ne("expired"))
            .load(conn)
            .context("Failed to load expiring escrows")
    }
}
