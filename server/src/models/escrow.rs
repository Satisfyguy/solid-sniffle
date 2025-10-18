//! Escrow model and related database operations

use diesel::prelude::*;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};

use super::super::schema::escrows;
use monero_marketplace_common::{EscrowStatus, UserId, Amount, MoneroAddress};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = escrows)]
pub struct Escrow {
    #[diesel(column_name = "id")]
    pub id: Uuid,
    pub order_id: Uuid,
    pub buyer_id: UserId,
    pub vendor_id: UserId,
    pub arbiter_id: UserId,
    pub amount: Amount,
    pub multisig_address: Option<MoneroAddress>,
    pub status: EscrowStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub buyer_wallet_info: Option<Vec<u8>>,
    pub vendor_wallet_info: Option<Vec<u8>>,
    pub arbiter_wallet_info: Option<Vec<u8>>,
}

#[derive(Insertable)]
#[diesel(table_name = escrows)]
pub struct NewEscrow {
    pub id: Uuid,
    pub order_id: Uuid,
    pub buyer_id: UserId,
    pub vendor_id: UserId,
    pub arbiter_id: UserId,
    pub amount: Amount,
    pub status: EscrowStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Placeholder for CRUD operations
impl Escrow {
    pub async fn create(_conn: &mut SqliteConnection, _new_escrow: NewEscrow) -> QueryResult<Escrow> {
        // Placeholder
        Err(diesel::result::Error::NotFound)
    }

    pub async fn find_by_id(_conn: &mut SqliteConnection, _id: Uuid) -> QueryResult<Escrow> {
        // Placeholder
        Err(diesel::result::Error::NotFound)
    }
}
