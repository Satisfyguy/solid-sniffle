//! Shopping cart models and session management
//!
//! Cart is stored in session as JSON for simplicity and speed.
//! For production with persistence, consider database storage.

use serde::{Deserialize, Serialize};

/// Item in shopping cart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    /// Listing ID
    pub listing_id: String,
    /// Product title (cached for display)
    pub title: String,
    /// Vendor ID (cached)
    pub vendor_id: String,
    /// Vendor username (cached for display)
    pub vendor_username: String,
    /// Unit price in atomic units (piconeros)
    pub unit_price_xmr: i64,
    /// Quantity
    pub quantity: i32,
    /// Image IPFS CID (cached, optional)
    pub image_cid: Option<String>,
}

impl CartItem {
    /// Calculate total price for this item (quantity * unit_price)
    pub fn total_price(&self) -> i64 {
        self.unit_price_xmr.saturating_mul(self.quantity as i64)
    }
}

/// Shopping cart
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cart {
    /// Items in cart
    pub items: Vec<CartItem>,
}

impl Cart {
    /// Create a new empty cart
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    /// Add item to cart or update quantity if already exists
    pub fn add_item(&mut self, item: CartItem) -> Result<(), String> {
        // Check if item already in cart
        if let Some(existing) = self.items.iter_mut().find(|i| i.listing_id == item.listing_id) {
            // Update quantity
            existing.quantity = existing.quantity.saturating_add(item.quantity);
            Ok(())
        } else {
            // Add new item
            if item.quantity <= 0 {
                return Err("Quantity must be positive".to_string());
            }
            self.items.push(item);
            Ok(())
        }
    }

    /// Remove item from cart
    pub fn remove_item(&mut self, listing_id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|item| item.listing_id != listing_id);
        self.items.len() < before
    }

    /// Update item quantity
    pub fn update_quantity(&mut self, listing_id: &str, quantity: i32) -> Result<(), String> {
        if quantity <= 0 {
            return Err("Quantity must be positive".to_string());
        }

        if let Some(item) = self.items.iter_mut().find(|i| i.listing_id == listing_id) {
            item.quantity = quantity;
            Ok(())
        } else {
            Err("Item not found in cart".to_string())
        }
    }

    /// Clear all items from cart
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Get total number of items in cart
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Get total quantity (sum of all quantities)
    pub fn total_quantity(&self) -> i32 {
        self.items.iter().map(|item| item.quantity).sum()
    }

    /// Calculate total cart value in atomic units
    pub fn total_price(&self) -> i64 {
        self.items.iter().map(|item| item.total_price()).sum()
    }

    /// Calculate total cart value in XMR (as f64 for display)
    pub fn total_price_xmr(&self) -> f64 {
        self.total_price() as f64 / 1_000_000_000_000.0
    }

    /// Check if cart is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get item by listing ID
    pub fn get_item(&self, listing_id: &str) -> Option<&CartItem> {
        self.items.iter().find(|item| item.listing_id == listing_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cart_add_item() {
        let mut cart = Cart::new();
        let item = CartItem {
            listing_id: "test123".to_string(),
            title: "Test Product".to_string(),
            vendor_id: "vendor1".to_string(),
            vendor_username: "VendorName".to_string(),
            unit_price_xmr: 1_000_000_000_000, // 1 XMR
            quantity: 2,
            image_cid: None,
        };

        cart.add_item(item.clone()).unwrap();
        assert_eq!(cart.item_count(), 1);
        assert_eq!(cart.total_quantity(), 2);
        assert_eq!(cart.total_price(), 2_000_000_000_000);
    }

    #[test]
    fn test_cart_update_quantity() {
        let mut cart = Cart::new();
        let item = CartItem {
            listing_id: "test123".to_string(),
            title: "Test Product".to_string(),
            vendor_id: "vendor1".to_string(),
            vendor_username: "VendorName".to_string(),
            unit_price_xmr: 1_000_000_000_000,
            quantity: 1,
            image_cid: None,
        };

        cart.add_item(item).unwrap();
        cart.update_quantity("test123", 5).unwrap();

        assert_eq!(cart.get_item("test123").unwrap().quantity, 5);
    }

    #[test]
    fn test_cart_remove_item() {
        let mut cart = Cart::new();
        let item = CartItem {
            listing_id: "test123".to_string(),
            title: "Test Product".to_string(),
            vendor_id: "vendor1".to_string(),
            vendor_username: "VendorName".to_string(),
            unit_price_xmr: 1_000_000_000_000,
            quantity: 1,
            image_cid: None,
        };

        cart.add_item(item).unwrap();
        assert_eq!(cart.item_count(), 1);

        cart.remove_item("test123");
        assert_eq!(cart.item_count(), 0);
    }
}
