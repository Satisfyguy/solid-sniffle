//! Order model and related database operations
//!
//! Represents a purchase order in the marketplace escrow system.

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::schema::orders;

/// Order status enum tracking the lifecycle of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    /// Order created, awaiting escrow funding
    Pending,
    /// Funds locked in escrow, awaiting vendor action
    Funded,
    /// Vendor marked as shipped
    Shipped,
    /// Buyer confirmed receipt, funds released to vendor
    Completed,
    /// Order cancelled, funds refunded
    Cancelled,
    /// Dispute raised, arbiter involved
    Disputed,
    /// Refund processed
    Refunded,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "pending",
            OrderStatus::Funded => "funded",
            OrderStatus::Shipped => "shipped",
            OrderStatus::Completed => "completed",
            OrderStatus::Cancelled => "cancelled",
            OrderStatus::Disputed => "disputed",
            OrderStatus::Refunded => "refunded",
        }
    }

    /// Check if status transition is valid
    pub fn can_transition_to(&self, target: &OrderStatus) -> bool {
        match (self, target) {
            // Pending can go to funded or cancelled
            (OrderStatus::Pending, OrderStatus::Funded) => true,
            (OrderStatus::Pending, OrderStatus::Cancelled) => true,

            // Funded can go to shipped, disputed, or cancelled
            (OrderStatus::Funded, OrderStatus::Shipped) => true,
            (OrderStatus::Funded, OrderStatus::Disputed) => true,
            (OrderStatus::Funded, OrderStatus::Cancelled) => true,

            // Shipped can go to completed or disputed
            (OrderStatus::Shipped, OrderStatus::Completed) => true,
            (OrderStatus::Shipped, OrderStatus::Disputed) => true,

            // Disputed can go to completed or refunded
            (OrderStatus::Disputed, OrderStatus::Completed) => true,
            (OrderStatus::Disputed, OrderStatus::Refunded) => true,

            // Terminal states can't transition
            (OrderStatus::Completed, _) => false,
            (OrderStatus::Cancelled, _) => false,
            (OrderStatus::Refunded, _) => false,

            _ => false,
        }
    }
}

impl FromStr for OrderStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(OrderStatus::Pending),
            "funded" => Ok(OrderStatus::Funded),
            "shipped" => Ok(OrderStatus::Shipped),
            "completed" => Ok(OrderStatus::Completed),
            "cancelled" => Ok(OrderStatus::Cancelled),
            "disputed" => Ok(OrderStatus::Disputed),
            "refunded" => Ok(OrderStatus::Refunded),
            _ => anyhow::bail!("Invalid order status: {}", s),
        }
    }
}

/// Order database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = orders)]
pub struct Order {
    pub id: String,
    pub buyer_id: String,
    pub vendor_id: String,
    pub listing_id: String,
    pub escrow_id: Option<String>,
    pub status: String,
    /// Total amount in atomic units (piconeros)
    pub total_xmr: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    /// Shipping address (encrypted in DB via SQLCipher)
    pub shipping_address: Option<String>,
    /// Optional shipping notes from buyer (e.g., delivery instructions)
    pub shipping_notes: Option<String>,
}

/// New order for insertion
#[derive(Insertable)]
#[diesel(table_name = orders)]
pub struct NewOrder {
    pub id: String,
    pub buyer_id: String,
    pub vendor_id: String,
    pub listing_id: String,
    pub escrow_id: Option<String>,
    pub status: String,
    pub total_xmr: i64,
    pub shipping_address: Option<String>,
    pub shipping_notes: Option<String>,
}

impl Order {
    /// Create a new order in the database
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `new_order` - New order data
    ///
    /// # Returns
    ///
    /// The created order with timestamps populated
    pub fn create(conn: &mut SqliteConnection, new_order: NewOrder) -> Result<Order> {
        diesel::insert_into(orders::table)
            .values(&new_order)
            .execute(conn)
            .context("Failed to insert order")?;

        orders::table
            .filter(orders::id.eq(new_order.id))
            .first(conn)
            .context("Failed to retrieve created order")
    }

    /// Find order by ID
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `order_id` - Order UUID
    pub fn find_by_id(conn: &mut SqliteConnection, order_id: String) -> Result<Order> {
        orders::table
            .filter(orders::id.eq(order_id.clone()))
            .first(conn)
            .context(format!("Order with ID {} not found", order_id))
    }

    /// Find all orders by buyer ID
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `buyer_id` - Buyer user ID
    pub fn find_by_buyer(conn: &mut SqliteConnection, buyer_id: String) -> Result<Vec<Order>> {
        orders::table
            .filter(orders::buyer_id.eq(buyer_id))
            .order(orders::created_at.desc())
            .load(conn)
            .context("Failed to load buyer orders")
    }

    /// Find all orders by vendor ID
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `vendor_id` - Vendor user ID
    pub fn find_by_vendor(conn: &mut SqliteConnection, vendor_id: String) -> Result<Vec<Order>> {
        orders::table
            .filter(orders::vendor_id.eq(vendor_id))
            .order(orders::created_at.desc())
            .load(conn)
            .context("Failed to load vendor orders")
    }

    /// Find all orders for a user (as buyer OR vendor)
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `user_id` - User ID (will match against both buyer_id and vendor_id)
    pub fn find_by_user(conn: &mut SqliteConnection, user_id: String) -> Result<Vec<Order>> {
        orders::table
            .filter(
                orders::buyer_id
                    .eq(&user_id)
                    .or(orders::vendor_id.eq(&user_id)),
            )
            .order(orders::created_at.desc())
            .load(conn)
            .context("Failed to load user orders")
    }

    /// Find orders by status
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `status_str` - Status string
    pub fn find_by_status(conn: &mut SqliteConnection, status_str: &str) -> Result<Vec<Order>> {
        orders::table
            .filter(orders::status.eq(status_str))
            .order(orders::created_at.desc())
            .load(conn)
            .context("Failed to load orders by status")
    }

    /// Update order status with validation
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `order_id` - Order UUID
    /// * `new_status` - New status
    ///
    /// # Returns
    ///
    /// Updated order
    ///
    /// # Errors
    ///
    /// Returns error if status transition is invalid
    pub fn update_status(
        conn: &mut SqliteConnection,
        order_id: String,
        new_status: OrderStatus,
    ) -> Result<Order> {
        let order = Self::find_by_id(conn, order_id.clone())?;
        let current_status = order.status.parse::<OrderStatus>()?;

        if !current_status.can_transition_to(&new_status) {
            anyhow::bail!(
                "Invalid status transition: {} -> {}",
                current_status.as_str(),
                new_status.as_str()
            );
        }

        diesel::update(orders::table.filter(orders::id.eq(order_id.clone())))
            .set(orders::status.eq(new_status.as_str()))
            .execute(conn)
            .context("Failed to update order status")?;

        Self::find_by_id(conn, order_id)
    }

    /// Associate escrow with order
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `order_id` - Order UUID
    /// * `escrow_id` - Escrow UUID
    pub fn set_escrow(
        conn: &mut SqliteConnection,
        order_id: String,
        escrow_id: String,
    ) -> Result<Order> {
        diesel::update(orders::table.filter(orders::id.eq(order_id.clone())))
            .set(orders::escrow_id.eq(escrow_id))
            .execute(conn)
            .context("Failed to set escrow ID")?;

        Self::find_by_id(conn, order_id)
    }

    /// Find order by escrow ID
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `escrow_id` - Escrow UUID
    pub fn find_by_escrow(conn: &mut SqliteConnection, escrow_id: String) -> Result<Order> {
        orders::table
            .filter(orders::escrow_id.eq(escrow_id.clone()))
            .first(conn)
            .context(format!("Order with escrow ID {} not found", escrow_id))
    }

    /// Convert total from atomic units to XMR
    pub fn total_as_xmr(&self) -> f64 {
        self.total_xmr as f64 / 1_000_000_000_000.0
    }

    /// Get parsed status enum
    pub fn get_status(&self) -> Result<OrderStatus> {
        self.status.parse::<OrderStatus>()
    }

    /// Check if buyer can cancel this order
    pub fn can_buyer_cancel(&self) -> bool {
        matches!(
            self.get_status(),
            Ok(OrderStatus::Pending) | Ok(OrderStatus::Funded)
        )
    }

    /// Check if vendor can mark as shipped
    pub fn can_mark_shipped(&self) -> bool {
        matches!(self.get_status(), Ok(OrderStatus::Funded))
    }

    /// Check if buyer can confirm receipt
    pub fn can_confirm_receipt(&self) -> bool {
        matches!(self.get_status(), Ok(OrderStatus::Shipped))
    }

    /// Check if order can be disputed
    pub fn can_dispute(&self) -> bool {
        matches!(
            self.get_status(),
            Ok(OrderStatus::Funded) | Ok(OrderStatus::Shipped)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_status_conversion() {
        assert_eq!(OrderStatus::Pending.as_str(), "pending");
        assert_eq!(OrderStatus::Funded.as_str(), "funded");
        assert_eq!(OrderStatus::Shipped.as_str(), "shipped");
        assert_eq!(OrderStatus::Completed.as_str(), "completed");
        assert_eq!(OrderStatus::Cancelled.as_str(), "cancelled");
        assert_eq!(OrderStatus::Disputed.as_str(), "disputed");
        assert_eq!(OrderStatus::Refunded.as_str(), "refunded");

        assert!(matches!(
            "pending".parse::<OrderStatus>(),
            Ok(OrderStatus::Pending)
        ));
        assert!(matches!(
            "completed".parse::<OrderStatus>(),
            Ok(OrderStatus::Completed)
        ));
        assert!("invalid".parse::<OrderStatus>().is_err());
    }

    #[test]
    fn test_status_transitions() {
        // Valid transitions
        assert!(OrderStatus::Pending.can_transition_to(&OrderStatus::Funded));
        assert!(OrderStatus::Pending.can_transition_to(&OrderStatus::Cancelled));
        assert!(OrderStatus::Funded.can_transition_to(&OrderStatus::Shipped));
        assert!(OrderStatus::Funded.can_transition_to(&OrderStatus::Disputed));
        assert!(OrderStatus::Shipped.can_transition_to(&OrderStatus::Completed));
        assert!(OrderStatus::Shipped.can_transition_to(&OrderStatus::Disputed));
        assert!(OrderStatus::Disputed.can_transition_to(&OrderStatus::Completed));
        assert!(OrderStatus::Disputed.can_transition_to(&OrderStatus::Refunded));

        // Invalid transitions
        assert!(!OrderStatus::Pending.can_transition_to(&OrderStatus::Shipped));
        assert!(!OrderStatus::Pending.can_transition_to(&OrderStatus::Completed));
        assert!(!OrderStatus::Completed.can_transition_to(&OrderStatus::Pending));
        assert!(!OrderStatus::Completed.can_transition_to(&OrderStatus::Disputed));
        assert!(!OrderStatus::Cancelled.can_transition_to(&OrderStatus::Funded));
        assert!(!OrderStatus::Refunded.can_transition_to(&OrderStatus::Completed));
    }

    #[test]
    fn test_total_conversion() {
        let order = Order {
            id: "test".to_string(),
            buyer_id: "buyer".to_string(),
            vendor_id: "vendor".to_string(),
            listing_id: "listing".to_string(),
            escrow_id: None,
            status: "pending".to_string(),
            total_xmr: 2_500_000_000_000, // 2.5 XMR
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        };

        assert_eq!(order.total_as_xmr(), 2.5);
    }

    #[test]
    fn test_can_confirm_receipt() {
        // Test shipped order can be confirmed
        let shipped_order = Order {
            id: "test".to_string(),
            buyer_id: "buyer".to_string(),
            vendor_id: "vendor".to_string(),
            listing_id: "listing".to_string(),
            escrow_id: Some("escrow123".to_string()),
            status: "shipped".to_string(),
            total_xmr: 1_000_000_000_000,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        };
        assert!(shipped_order.can_confirm_receipt());

        // Test funded order cannot be confirmed
        let funded_order = Order {
            status: "funded".to_string(),
            ..shipped_order.clone()
        };
        assert!(!funded_order.can_confirm_receipt());

        // Test pending order cannot be confirmed
        let pending_order = Order {
            status: "pending".to_string(),
            ..shipped_order.clone()
        };
        assert!(!pending_order.can_confirm_receipt());

        // Test completed order cannot be confirmed again
        let completed_order = Order {
            status: "completed".to_string(),
            ..shipped_order
        };
        assert!(!completed_order.can_confirm_receipt());
    }

    #[test]
    fn test_can_mark_shipped() {
        let funded_order = Order {
            id: "test".to_string(),
            buyer_id: "buyer".to_string(),
            vendor_id: "vendor".to_string(),
            listing_id: "listing".to_string(),
            escrow_id: Some("escrow123".to_string()),
            status: "funded".to_string(),
            total_xmr: 1_000_000_000_000,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        };

        // Only funded orders can be marked as shipped
        assert!(funded_order.can_mark_shipped());

        // Pending orders cannot be shipped
        let pending_order = Order {
            status: "pending".to_string(),
            ..funded_order.clone()
        };
        assert!(!pending_order.can_mark_shipped());

        // Shipped orders cannot be shipped again
        let shipped_order = Order {
            status: "shipped".to_string(),
            ..funded_order
        };
        assert!(!shipped_order.can_mark_shipped());
    }
}
