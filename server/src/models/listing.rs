//! Listing model and related database operations
//!
//! Represents a product or service listed for sale on the marketplace.

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::schema::listings;

/// Listing status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListingStatus {
    /// Listing is active and available for purchase
    Active,
    /// Listing is temporarily inactive (vendor can reactivate)
    Inactive,
    /// Listing is sold out (stock = 0)
    SoldOut,
    /// Listing was deleted/removed
    Deleted,
}

impl ListingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ListingStatus::Active => "active",
            ListingStatus::Inactive => "inactive",
            ListingStatus::SoldOut => "sold_out",
            ListingStatus::Deleted => "deleted",
        }
    }
}

impl FromStr for ListingStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(ListingStatus::Active),
            "inactive" => Ok(ListingStatus::Inactive),
            "sold_out" => Ok(ListingStatus::SoldOut),
            "deleted" => Ok(ListingStatus::Deleted),
            _ => anyhow::bail!("Invalid listing status: {}", s),
        }
    }
}

/// Listing database model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = listings)]
pub struct Listing {
    pub id: String,
    pub vendor_id: String,
    pub title: String,
    pub description: String,
    /// Price in atomic units (piconeros): 1 XMR = 1,000,000,000,000 piconeros
    pub price_xmr: i64,
    pub stock: i32,
    pub status: String,
    /// IPFS CIDs for product images stored as JSON array: ["Qm...", "Qm..."]
    pub images_ipfs_cids: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// New listing for insertion
#[derive(Insertable)]
#[diesel(table_name = listings)]
pub struct NewListing {
    pub id: String,
    pub vendor_id: String,
    pub title: String,
    pub description: String,
    pub price_xmr: i64,
    pub stock: i32,
    pub status: String,
    pub images_ipfs_cids: Option<String>,
}

/// Listing update data
#[derive(AsChangeset)]
#[diesel(table_name = listings)]
pub struct UpdateListing {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price_xmr: Option<i64>,
    pub stock: Option<i32>,
    pub status: Option<String>,
}

impl Listing {
    /// Create a new listing in the database
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `new_listing` - New listing data
    ///
    /// # Returns
    ///
    /// The created listing with timestamps populated by the database
    pub fn create(conn: &mut SqliteConnection, new_listing: NewListing) -> Result<Listing> {
        let listing_id = new_listing.id.clone();

        diesel::insert_into(listings::table)
            .values(&new_listing)
            .execute(conn)
            .context("Failed to insert listing")?;

        listings::table
            .find(listing_id)
            .first(conn)
            .context("Failed to retrieve created listing")
    }

    /// Find listing by ID
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `listing_id` - Listing UUID
    pub fn find_by_id(conn: &mut SqliteConnection, listing_id: String) -> Result<Listing> {
        listings::table
            .filter(listings::id.eq(listing_id.clone()))
            .first(conn)
            .context(format!("Listing with ID {} not found", listing_id))
    }

    /// Find all listings by vendor ID
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `vendor_id` - Vendor user ID
    pub fn find_by_vendor(conn: &mut SqliteConnection, vendor_id: String) -> Result<Vec<Listing>> {
        listings::table
            .filter(listings::vendor_id.eq(vendor_id))
            .order(listings::created_at.desc())
            .load(conn)
            .context("Failed to load vendor listings")
    }

    /// List all active listings (paginated)
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `limit` - Maximum number of results
    /// * `offset` - Number of results to skip
    ///
    /// # Returns
    ///
    /// Vector of active listings, ordered by creation date (newest first)
    pub fn list_active(
        conn: &mut SqliteConnection,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Listing>> {
        listings::table
            .filter(listings::status.eq("active"))
            .order(listings::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(conn)
            .context("Failed to load active listings")
    }

    /// Search listings by title (case-insensitive)
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `query` - Search query string
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// Vector of matching listings
    pub fn search_by_title(
        conn: &mut SqliteConnection,
        query: &str,
        limit: i64,
    ) -> Result<Vec<Listing>> {
        let search_pattern = format!("%{}%", query);
        listings::table
            .filter(listings::title.like(search_pattern))
            .filter(listings::status.eq("active"))
            .limit(limit)
            .load(conn)
            .context("Failed to search listings")
    }

    /// Update listing fields
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `listing_id` - Listing UUID
    /// * `update_data` - Fields to update
    ///
    /// # Returns
    ///
    /// Updated listing
    pub fn update(
        conn: &mut SqliteConnection,
        listing_id: String,
        update_data: UpdateListing,
    ) -> Result<Listing> {
        diesel::update(listings::table.filter(listings::id.eq(listing_id.clone())))
            .set(&update_data)
            .execute(conn)
            .context("Failed to update listing")?;

        Self::find_by_id(conn, listing_id)
    }

    /// Decrease listing stock by quantity
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `listing_id` - Listing UUID
    /// * `quantity` - Amount to decrease stock by
    ///
    /// # Returns
    ///
    /// Updated listing with decreased stock
    ///
    /// # Errors
    ///
    /// Returns error if insufficient stock available
    pub fn decrease_stock(
        conn: &mut SqliteConnection,
        listing_id: String,
        quantity: i32,
    ) -> Result<Listing> {
        let listing = Self::find_by_id(conn, listing_id.clone())?;

        if listing.stock < quantity {
            anyhow::bail!(
                "Insufficient stock: available={}, requested={}",
                listing.stock,
                quantity
            );
        }

        let new_stock = listing.stock - quantity;
        let new_status = if new_stock == 0 {
            "sold_out".to_string()
        } else {
            listing.status.clone()
        };

        diesel::update(listings::table.filter(listings::id.eq(listing_id.clone())))
            .set((
                listings::stock.eq(new_stock),
                listings::status.eq(new_status),
            ))
            .execute(conn)
            .context("Failed to decrease stock")?;

        Self::find_by_id(conn, listing_id)
    }

    /// Increase listing stock by quantity (e.g., after order cancellation)
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `listing_id` - Listing UUID
    /// * `quantity` - Amount to increase stock by
    pub fn increase_stock(
        conn: &mut SqliteConnection,
        listing_id: String,
        quantity: i32,
    ) -> Result<Listing> {
        let listing = Self::find_by_id(conn, listing_id.clone())?;
        let new_stock = listing.stock + quantity;

        // If was sold out and now has stock, reactivate
        let new_status = if listing.status == "sold_out" && new_stock > 0 {
            "active".to_string()
        } else {
            listing.status.clone()
        };

        diesel::update(listings::table.filter(listings::id.eq(listing_id.clone())))
            .set((
                listings::stock.eq(new_stock),
                listings::status.eq(new_status),
            ))
            .execute(conn)
            .context("Failed to increase stock")?;

        Self::find_by_id(conn, listing_id)
    }

    /// Delete listing (soft delete by setting status to 'deleted')
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection
    /// * `listing_id` - Listing UUID
    pub fn delete(conn: &mut SqliteConnection, listing_id: String) -> Result<()> {
        diesel::update(listings::table.filter(listings::id.eq(listing_id)))
            .set(listings::status.eq("deleted"))
            .execute(conn)
            .context("Failed to delete listing")?;
        Ok(())
    }

    /// Convert price from atomic units to XMR
    pub fn price_as_xmr(&self) -> f64 {
        self.price_xmr as f64 / 1_000_000_000_000.0
    }

    /// Get parsed status enum
    pub fn get_status(&self) -> Result<ListingStatus> {
        self.status.parse::<ListingStatus>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listing_status_conversion() {
        assert_eq!(ListingStatus::Active.as_str(), "active");
        assert_eq!(ListingStatus::Inactive.as_str(), "inactive");
        assert_eq!(ListingStatus::SoldOut.as_str(), "sold_out");
        assert_eq!(ListingStatus::Deleted.as_str(), "deleted");

        assert!(matches!(
            "active".parse::<ListingStatus>(),
            Ok(ListingStatus::Active)
        ));
        assert!(matches!(
            "inactive".parse::<ListingStatus>(),
            Ok(ListingStatus::Inactive)
        ));
        assert!(matches!(
            "sold_out".parse::<ListingStatus>(),
            Ok(ListingStatus::SoldOut)
        ));
        assert!(matches!(
            "deleted".parse::<ListingStatus>(),
            Ok(ListingStatus::Deleted)
        ));
        assert!("invalid".parse::<ListingStatus>().is_err());
    }

    #[test]
    fn test_price_conversion() {
        let listing = Listing {
            id: "test".to_string(),
            vendor_id: "vendor".to_string(),
            title: "Test".to_string(),
            description: "Desc".to_string(),
            price_xmr: 1_500_000_000_000, // 1.5 XMR
            stock: 10,
            status: "active".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            images_ipfs_cids: None,
        };

        assert_eq!(listing.price_as_xmr(), 1.5);
    }
}
