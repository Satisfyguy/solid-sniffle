//! Database operations for the reputation system
//!
//! This module handles storage and retrieval of cryptographically-signed reviews.
//! All reviews are verified before storage and can be exported for IPFS portability.

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use reputation_common::types::SignedReview;

use crate::db::DbPool;
use crate::schema::reviews;

/// Database model for a review
///
/// Represents the storage format in SQLite. Maps to `SignedReview` from reputation-common.
#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = reviews)]
pub struct Review {
    pub id: String,
    pub txid: String,
    pub reviewer_id: String,
    pub vendor_id: String,
    pub rating: i32,
    pub comment: Option<String>,
    pub buyer_pubkey: String,
    pub signature: String,
    pub timestamp: NaiveDateTime,
    pub verified: bool,
    pub created_at: NaiveDateTime,
}

/// Insert a cryptographically-signed review into the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `review` - Signed review (signature already verified by caller)
/// * `reviewer_id` - UUID of the buyer submitting the review
/// * `vendor_id` - UUID of the vendor being reviewed
///
/// # Security
/// - Caller MUST verify signature before calling this function
/// - Duplicate reviews (same txid + reviewer) are rejected by UNIQUE constraint
/// - Foreign keys ensure reviewer and vendor exist in users table
///
/// # Errors
/// - Database connection failure
/// - Constraint violation (duplicate review, invalid user IDs)
/// - Serialization errors
///
/// # Example
/// ```no_run
/// let review = sign_review(...);
/// if verify_review_signature(&review)? {
///     db_insert_review(&pool, &review, &buyer_id, &vendor_id).await?;
/// }
/// ```
pub async fn db_insert_review(
    pool: &DbPool,
    review: &SignedReview,
    reviewer_id: &str,
    vendor_id: &str,
) -> Result<Review> {
    let mut conn = pool.get().context("Failed to get DB connection")?;

    let new_review = Review {
        id: Uuid::new_v4().to_string(),
        txid: review.txid.clone(),
        reviewer_id: reviewer_id.to_string(),
        vendor_id: vendor_id.to_string(),
        rating: review.rating as i32,
        comment: review.comment.clone(),
        buyer_pubkey: review.buyer_pubkey.clone(),
        signature: review.signature.clone(),
        timestamp: review.timestamp.naive_utc(),
        verified: false, // On-chain verification happens separately
        created_at: Utc::now().naive_utc(),
    };

    tokio::task::spawn_blocking(move || {
        diesel::insert_into(reviews::table)
            .values(&new_review)
            .execute(&mut conn)
            .context("Failed to insert review into database")?;

        reviews::table
            .filter(reviews::id.eq(&new_review.id))
            .first::<Review>(&mut conn)
            .context("Failed to retrieve created review")
    })
    .await
    .context("Database task panicked")?
}

/// Retrieve all reviews for a specific vendor
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `vendor_uuid` - UUID of the vendor
///
/// # Returns
/// Vector of `SignedReview` objects, ordered by timestamp (newest first)
///
/// # Performance
/// - Uses index `idx_reviews_vendor` for fast lookup
/// - Returns empty vec if vendor has no reviews (not an error)
///
/// # Errors
/// - Database connection failure
/// - Deserialization errors (corrupted data)
pub async fn db_get_vendor_reviews(
    pool: &DbPool,
    vendor_uuid: Uuid,
) -> Result<Vec<SignedReview>> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let vendor_id_str = vendor_uuid.to_string();

    tokio::task::spawn_blocking(move || {
        let db_reviews = reviews::table
            .filter(reviews::vendor_id.eq(vendor_id_str))
            .order(reviews::timestamp.desc())
            .load::<Review>(&mut conn)
            .context("Failed to load reviews from database")?;

        // Convert Review (DB model) → SignedReview (domain model)
        let signed_reviews: Vec<SignedReview> = db_reviews
            .into_iter()
            .map(|r| SignedReview {
                txid: r.txid,
                rating: r.rating as u8,
                comment: r.comment,
                timestamp: DateTime::from_naive_utc_and_offset(r.timestamp, Utc),
                buyer_pubkey: r.buyer_pubkey,
                signature: r.signature,
            })
            .collect();

        Ok(signed_reviews)
    })
    .await
    .context("Database task panicked")?
}

/// Retrieve all verified reviews for a vendor
///
/// Same as `db_get_vendor_reviews` but filters for `verified = true`.
/// Verified reviews have been confirmed on the Monero blockchain.
///
/// # Performance
/// - Uses composite index `idx_reviews_vendor_verified` for optimal speed
pub async fn db_get_verified_vendor_reviews(
    pool: &DbPool,
    vendor_uuid: Uuid,
) -> Result<Vec<SignedReview>> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let vendor_id_str = vendor_uuid.to_string();

    tokio::task::spawn_blocking(move || {
        let db_reviews = reviews::table
            .filter(reviews::vendor_id.eq(vendor_id_str))
            .filter(reviews::verified.eq(true))
            .order(reviews::timestamp.desc())
            .load::<Review>(&mut conn)
            .context("Failed to load verified reviews")?;

        let signed_reviews: Vec<SignedReview> = db_reviews
            .into_iter()
            .map(|r| SignedReview {
                txid: r.txid,
                rating: r.rating as u8,
                comment: r.comment,
                timestamp: DateTime::from_naive_utc_and_offset(r.timestamp, Utc),
                buyer_pubkey: r.buyer_pubkey,
                signature: r.signature,
            })
            .collect();

        Ok(signed_reviews)
    })
    .await
    .context("Database task panicked")?
}

/// Mark a review as verified (after blockchain confirmation)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `txid` - Transaction hash to verify
///
/// # Security
/// - Only call this after confirming the transaction exists on-chain
/// - Transaction must have sufficient confirmations (10+ recommended)
///
/// # Returns
/// Number of reviews updated (should be 1, or 0 if txid not found)
pub async fn db_mark_review_verified(pool: &DbPool, txid: &str) -> Result<usize> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let txid_clone = txid.to_string();

    tokio::task::spawn_blocking(move || {
        let updated = diesel::update(reviews::table.filter(reviews::txid.eq(txid_clone)))
            .set(reviews::verified.eq(true))
            .execute(&mut conn)
            .context("Failed to mark review as verified")?;

        Ok(updated)
    })
    .await
    .context("Database task panicked")?
}

/// Check if a review already exists for a transaction
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `txid` - Transaction hash
/// * `reviewer_id` - UUID of the reviewer
///
/// # Returns
/// `true` if review exists, `false` otherwise
///
/// # Use Case
/// Prevent duplicate reviews from the same buyer for the same transaction
pub async fn db_review_exists(pool: &DbPool, txid: &str, reviewer_id: &str) -> Result<bool> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let txid_clone = txid.to_string();
    let reviewer_clone = reviewer_id.to_string();

    tokio::task::spawn_blocking(move || {
        let count: i64 = reviews::table
            .filter(reviews::txid.eq(txid_clone))
            .filter(reviews::reviewer_id.eq(reviewer_clone))
            .count()
            .get_result(&mut conn)
            .context("Failed to check review existence")?;

        Ok(count > 0)
    })
    .await
    .context("Database task panicked")?
}

/// Get reputation statistics for a vendor
///
/// # Returns
/// Tuple of (total_reviews, average_rating)
///
/// # Performance
/// - Uses database aggregation for efficiency
/// - O(1) complexity with proper indexes
pub async fn db_get_vendor_stats(pool: &DbPool, vendor_uuid: Uuid) -> Result<(i64, f64)> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let vendor_id_str = vendor_uuid.to_string();

    tokio::task::spawn_blocking(move || {
        use diesel::dsl::count_star;

        // Get count of verified reviews
        let total: i64 = reviews::table
            .filter(reviews::vendor_id.eq(&vendor_id_str))
            .filter(reviews::verified.eq(true))
            .select(count_star())
            .first(&mut conn)
            .context("Failed to count vendor reviews")?;

        // Calculate average manually (Diesel's avg() returns Numeric which can't deserialize as f64 in SQLite)
        let ratings: Vec<i32> = reviews::table
            .filter(reviews::vendor_id.eq(&vendor_id_str))
            .filter(reviews::verified.eq(true))
            .select(reviews::rating)
            .load(&mut conn)
            .context("Failed to load ratings")?;

        let average = if ratings.is_empty() {
            0.0
        } else {
            let sum: i32 = ratings.iter().sum();
            sum as f64 / ratings.len() as f64
        };

        Ok((total, average))
    })
    .await
    .context("Database task panicked")?
}

#[cfg(test)]
mod tests {
    

    // Note: Integration tests require a real database
    // See server/tests/reputation_integration.rs for full tests
}
