# Database Optimizations

This document outlines strategies and configurations for optimizing database performance in the Monero Marketplace application.

## 1. Database Indexes

To improve query performance for frequently accessed columns, the following indexes should be created. These are crucial for speeding up lookups, sorting, and filtering operations.

```sql
CREATE INDEX IF NOT EXISTS idx_listings_vendor_id ON listings(vendor_id);
CREATE INDEX IF NOT EXISTS idx_listings_created_at ON listings(created_at);
CREATE INDEX IF NOT EXISTS idx_orders_buyer_id ON orders(buyer_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_escrows_state ON escrows(state);
CREATE INDEX IF NOT EXISTS idx_transactions_order_id ON transactions(order_id);
```

**Implementation Notes:**
- These `CREATE INDEX IF NOT EXISTS` statements can be included in your database migration scripts to ensure they are applied automatically during deployment.
- Regularly review query plans (`EXPLAIN QUERY PLAN`) to identify other potential indexing opportunities.

## 2. Connection Pooling with Diesel

Connection pooling is essential for managing database connections efficiently, reducing the overhead of establishing new connections for each request. For applications using Diesel, `r2d2` is the recommended connection pool.

Configure the connection pool with appropriate settings to match your application's load characteristics:

```rust
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;
use std::time::Duration;

// ... inside your application setup ...

let manager = ConnectionManager::<SqliteConnection>::new(database_url);
let pool = Pool::builder()
    .max_size(20) // Maximum number of connections in the pool
    .min_idle(Some(5)) // Minimum number of idle connections to maintain
    .connection_timeout(Duration::from_secs(10)) // How long to wait for a connection
    .build(manager)
    .expect("Failed to create database connection pool");

// ... pass the pool to your application state ...
```

**Configuration Considerations:**
- `max_size`: Adjust based on the number of concurrent requests your application handles. Too few can cause bottlenecks; too many can exhaust database resources.
- `min_idle`: Keeps a minimum number of connections open, reducing latency for new requests.
- `connection_timeout`: Prevents requests from hanging indefinitely if the database is unresponsive.

## 3. Redis Caching (Optional)

For read-heavy operations, especially for data that doesn't change frequently (e.g., listing details), implementing a Redis cache can significantly reduce database load and improve response times.

**Example: Caching Listings**

```rust
use redis::{Client, Commands, RedisResult};
use serde_json;

// ... inside your application logic ...

async fn get_listing_from_cache_or_db(listing_id: i32, redis_client: &Client) -> RedisResult<String> {
    let mut con = redis_client.get_connection()?;
    let cache_key = format!("listing:{}", listing_id);

    // Try to get from cache
    if let Ok(cached_json) = con.get::<String, String>(&cache_key) {
        if !cached_json.is_empty() {
            println!("Cache hit for listing {}", listing_id);
            return Ok(cached_json);
        }
    }

    // Cache miss, fetch from DB
    println!("Cache miss for listing {}, fetching from DB", listing_id);
    // In a real app, this would involve a DB query
    let listing_from_db = serde_json::json!({ "id": listing_id, "title": "Example Listing", "price": 0.5 });
    let json_string = serde_json::to_string(&listing_from_db).unwrap();

    // Store in cache with an expiration (e.g., 5 minutes)
    con.set_ex(&cache_key, &json_string, 300)?; // 300 seconds = 5 minutes

    Ok(json_string)
}

// Example usage:
// let redis_client = Client::open("redis://127.0.0.1/").unwrap();
// let listing_data = get_listing_from_cache_or_db(1, &redis_client).await.unwrap();
```

**Considerations for Caching:**
- **Cache Invalidation:** Develop a strategy for invalidating cached data when the underlying database records change.
- **Cache Aside Pattern:** The example above demonstrates the cache-aside pattern, where the application checks the cache first and falls back to the database on a miss.
- **Data Consistency:** Be mindful of eventual consistency when using caching. For highly critical data, direct database reads might still be preferred.
- **Redis Deployment:** Redis would need to be deployed as an additional service, potentially within your Docker Compose setup.
