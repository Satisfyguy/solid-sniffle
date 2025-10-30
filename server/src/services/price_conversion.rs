//! XMR/USD Price Conversion Service
//!
//! Fetches real-time Monero to USD exchange rate from CoinGecko API via Tor proxy.
//! Implements caching with 5-minute TTL to reduce API calls and improve performance.

use anyhow::{Context, Result};
use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Cache entry for XMR/USD rate with timestamp
#[derive(Clone, Debug)]
struct CachedRate {
    rate: f64,
    timestamp: Instant,
}

/// Global cache for XMR/USD rate (thread-safe)
static RATE_CACHE: Mutex<Option<CachedRate>> = Mutex::new(None);

/// Cache TTL duration (5 minutes)
const CACHE_TTL: Duration = Duration::from_secs(300);

/// Fallback static rate if API is unreachable (150 USD/XMR)
const FALLBACK_RATE: f64 = 150.0;

/// CoinGecko API response structure
#[derive(Deserialize, Serialize, Debug)]
struct CoinGeckoResponse {
    monero: MoneroPrice,
}

#[derive(Deserialize, Serialize, Debug)]
struct MoneroPrice {
    usd: f64,
}

/// Fetch XMR/USD exchange rate from CoinGecko API via Tor
///
/// Returns cached rate if available and fresh (<5 min old), otherwise fetches from API.
/// Falls back to static rate (150 USD/XMR) if API call fails.
///
/// # Returns
/// - `Ok(rate)`: Exchange rate as f64 (e.g., 155.43 means 1 XMR = $155.43 USD)
///
/// # Errors
/// - Never returns error, always provides fallback rate
pub async fn get_xmr_usd_rate() -> f64 {
    // Check cache first
    if let Some(cached_rate) = get_cached_rate() {
        debug!("Using cached XMR/USD rate: ${:.2}", cached_rate);
        return cached_rate;
    }

    // Fetch from API
    match fetch_rate_from_api().await {
        Ok(rate) => {
            info!("Fetched fresh XMR/USD rate from CoinGecko: ${:.2}", rate);
            cache_rate(rate);
            rate
        }
        Err(e) => {
            warn!(
                "Failed to fetch XMR/USD rate from CoinGecko ({}), using fallback: ${:.2}",
                e, FALLBACK_RATE
            );
            FALLBACK_RATE
        }
    }
}

/// Get cached rate if available and not expired
fn get_cached_rate() -> Option<f64> {
    let cache = RATE_CACHE.lock().ok()?;

    if let Some(cached) = cache.as_ref() {
        let age = cached.timestamp.elapsed();
        if age < CACHE_TTL {
            debug!("Cache hit (age: {:?})", age);
            return Some(cached.rate);
        } else {
            debug!("Cache expired (age: {:?})", age);
        }
    }

    None
}

/// Cache the rate with current timestamp
fn cache_rate(rate: f64) {
    if let Ok(mut cache) = RATE_CACHE.lock() {
        *cache = Some(CachedRate {
            rate,
            timestamp: Instant::now(),
        });
        debug!("Cached XMR/USD rate: ${:.2}", rate);
    }
}

/// Fetch XMR/USD rate from CoinGecko API via Tor proxy
async fn fetch_rate_from_api() -> Result<f64> {
    // Configure Tor SOCKS5 proxy
    let proxy = Proxy::all("socks5h://127.0.0.1:9050")
        .context("Failed to configure Tor proxy for price API")?;

    // Build HTTP client with Tor proxy and timeout
    let client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(30)) // Tor can be slow
        .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0")
        .build()
        .context("Failed to build HTTP client")?;

    // CoinGecko API endpoint (free tier, no API key required)
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=monero&vs_currencies=usd";

    debug!("Fetching XMR/USD rate from CoinGecko via Tor...");

    // Make HTTP request
    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to send request to CoinGecko API")?;

    // Check HTTP status
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "CoinGecko API returned non-success status: {}",
            response.status()
        ));
    }

    // Parse JSON response
    let data: CoinGeckoResponse = response
        .json()
        .await
        .context("Failed to parse CoinGecko API response")?;

    // Validate rate is reasonable (between $50 and $1000)
    let rate = data.monero.usd;
    if rate < 50.0 || rate > 1000.0 {
        return Err(anyhow::anyhow!(
            "Unreasonable XMR/USD rate from API: ${:.2}",
            rate
        ));
    }

    Ok(rate)
}

/// Convert XMR atomic units (piconeros) to USD
///
/// # Arguments
/// * `atomic_units` - Price in piconeros (1 XMR = 1,000,000,000,000 piconeros)
///
/// # Returns
/// - USD amount as f64
///
/// # Example
/// ```
/// let atomic_price: i64 = 240_000_000_000; // 0.24 XMR
/// let usd_price = atomic_to_usd(atomic_price).await;
/// // If rate is $155/XMR, returns ~$37.20
/// ```
pub async fn atomic_to_usd(atomic_units: i64) -> f64 {
    const XMR_TO_ATOMIC: f64 = 1_000_000_000_000.0;

    let xmr = atomic_units as f64 / XMR_TO_ATOMIC;
    let rate = get_xmr_usd_rate().await;

    xmr * rate
}

/// Convert XMR (as f64) to USD
///
/// # Arguments
/// * `xmr` - Amount in XMR as f64
///
/// # Returns
/// - USD amount as f64
pub async fn xmr_to_usd(xmr: f64) -> f64 {
    let rate = get_xmr_usd_rate().await;
    xmr * rate
}

/// Clear the rate cache (useful for testing)
#[allow(dead_code)]
pub fn clear_cache() {
    if let Ok(mut cache) = RATE_CACHE.lock() {
        *cache = None;
        debug!("Rate cache cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_xmr_usd_rate() {
        // This test will use fallback rate if Tor/API unavailable
        let rate = get_xmr_usd_rate().await;
        assert!(rate > 0.0, "Rate should be positive");
        assert!(
            rate >= 50.0 && rate <= 1000.0,
            "Rate should be reasonable"
        );
    }

    #[tokio::test]
    async fn test_atomic_to_usd() {
        // Test with 1 XMR (1,000,000,000,000 atomic units)
        let atomic = 1_000_000_000_000i64;
        let usd = atomic_to_usd(atomic).await;

        // Should be between $50 and $1000
        assert!(
            usd >= 50.0 && usd <= 1000.0,
            "USD conversion should be reasonable"
        );
    }

    #[tokio::test]
    async fn test_xmr_to_usd() {
        let xmr = 2.5;
        let usd = xmr_to_usd(xmr).await;

        // Should be between $125 and $2500
        assert!(
            usd >= 125.0 && usd <= 2500.0,
            "USD conversion should be reasonable"
        );
    }

    #[test]
    fn test_cache_rate() {
        clear_cache();

        cache_rate(155.0);

        let cached = get_cached_rate();
        assert!(cached.is_some(), "Rate should be cached");
        assert_eq!(cached.unwrap(), 155.0, "Cached rate should match");
    }
}
