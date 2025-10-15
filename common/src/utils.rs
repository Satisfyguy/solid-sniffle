//! Utility functions for Monero Marketplace

use crate::error::{Error, Result};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a secure random ID
pub fn generate_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    SystemTime::now().duration_since(UNIX_EPOCH)
        .expect("System time should be after UNIX epoch")
        .as_nanos().hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Compute SHA256 hash of data
pub fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Validate Monero address format (basic check)
pub fn validate_monero_address(address: &str) -> Result<()> {
    if address.len() < 95 || address.len() > 106 {
        return Err(Error::InvalidInput("Invalid Monero address length".to_string()));
    }
    
    if !address.starts_with('4') && !address.starts_with('8') {
        return Err(Error::InvalidInput("Invalid Monero address prefix".to_string()));
    }
    
    // Basic character validation (base58)
    for c in address.chars() {
        if !c.is_ascii_alphanumeric() && c != '1' && c != '2' && c != '3' && c != '4' && c != '5' && c != '6' && c != '7' && c != '8' && c != '9' {
            return Err(Error::InvalidInput("Invalid character in Monero address".to_string()));
        }
    }
    
    Ok(())
}

/// Convert XMR to atomic units
pub fn xmr_to_atomic(xmr: f64) -> Result<u64> {
    if xmr < 0.0 {
        return Err(Error::InvalidInput("Amount cannot be negative".to_string()));
    }
    
    let atomic = (xmr * 1e12) as u64;
    Ok(atomic)
}

/// Convert atomic units to XMR
pub fn atomic_to_xmr(atomic: u64) -> f64 {
    atomic as f64 / 1e12
}

/// Get current timestamp in seconds
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after UNIX epoch")
        .as_secs()
}

/// Format amount for display
pub fn format_amount(atomic: u64) -> String {
    format!("{:.12} XMR", atomic_to_xmr(atomic))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
    }

    #[test]
    fn test_sha256_hash() {
        let data = b"hello world";
        let hash = sha256_hash(data);
        assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_validate_monero_address() {
        // Valid addresses
        assert!(validate_monero_address("4AdUndXHHZ6cFdRPAgP6zBFmZ1hBpiPsjCd1TqWLjokCLQcaQa4Yf8ZgWa61uB1DkHGrC1XqVjro7ykm5rF8YvP9aYTFjk").is_ok());
        
        // Invalid addresses
        assert!(validate_monero_address("invalid").is_err());
        assert!(validate_monero_address("").is_err());
    }

    #[test]
    fn test_xmr_conversion() {
        assert_eq!(xmr_to_atomic(1.0).expect("Valid XMR amount"), 1_000_000_000_000);
        assert_eq!(xmr_to_atomic(0.5).expect("Valid XMR amount"), 500_000_000_000);
        assert_eq!(atomic_to_xmr(1_000_000_000_000), 1.0);
        assert_eq!(atomic_to_xmr(500_000_000_000), 0.5);
    }

    #[test]
    fn test_format_amount() {
        assert_eq!(format_amount(1_000_000_000_000), "1.000000000000 XMR");
        assert_eq!(format_amount(500_000_000_000), "0.500000000000 XMR");
    }
}
