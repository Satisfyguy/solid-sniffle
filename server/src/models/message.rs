// ! Order message model for vendor-buyer communication
//!
//! Represents a message sent between buyer and vendor for a specific order.

use anyhow::Result;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::order_messages;

/// Order message database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = order_messages)]
pub struct OrderMessage {
    pub id: String,
    pub order_id: String,
    pub sender_id: String,
    pub message: String,
    pub created_at: i32, // Unix timestamp
}

/// New order message for insertion
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = order_messages)]
pub struct NewOrderMessage {
    pub id: String,
    pub order_id: String,
    pub sender_id: String,
    pub message: String,
    pub created_at: i32,
}

impl NewOrderMessage {
    /// Create a new order message
    pub fn new(order_id: String, sender_id: String, message: String) -> Self {
        let now = chrono::Utc::now().timestamp() as i32;

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            order_id,
            sender_id,
            message,
            created_at: now,
        }
    }
}

/// Message for API responses with sender username
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderMessageWithSender {
    pub id: String,
    pub order_id: String,
    pub sender_id: String,
    pub sender_username: String,
    pub message: String,
    pub created_at: i32,
    pub is_current_user: bool,
}

impl OrderMessage {
    /// Get all messages for an order
    pub fn get_by_order_id(
        order_id: &str,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<OrderMessage>> {
        use crate::schema::order_messages::dsl;

        let messages = dsl::order_messages
            .filter(dsl::order_id.eq(order_id))
            .order(dsl::created_at.asc())
            .load::<OrderMessage>(conn)?;

        Ok(messages)
    }

    /// Create a new message
    pub fn create(
        new_message: NewOrderMessage,
        conn: &mut SqliteConnection,
    ) -> Result<OrderMessage> {
        use crate::schema::order_messages::dsl;

        diesel::insert_into(dsl::order_messages)
            .values(&new_message)
            .execute(conn)?;

        let message = dsl::order_messages
            .find(&new_message.id)
            .first::<OrderMessage>(conn)?;

        Ok(message)
    }

    /// Get message count for an order
    pub fn count_by_order_id(
        order_id: &str,
        conn: &mut SqliteConnection,
    ) -> Result<i64> {
        use crate::schema::order_messages::dsl;

        let count = dsl::order_messages
            .filter(dsl::order_id.eq(order_id))
            .count()
            .get_result(conn)?;

        Ok(count)
    }
}
