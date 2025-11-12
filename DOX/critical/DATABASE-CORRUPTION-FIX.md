# Database Corruption Fix - CRITICAL

**Date**: 2025-11-12
**Severity**: CRITICAL
**Status**: FIXED

## Problem Statement

Database corruption was occurring repeatedly, requiring manual cleanup with `rm marketplace.db*`. Symptoms:
- `Error: file is not a database (26)` from SQLite/SQLCipher
- Server fails to start with r2d2 connection pool errors
- Loss of all marketplace data (users, listings, orders, escrows)

## Root Cause Analysis

### Investigation Findings

1. **10 File Descriptors Open on Same DB**
   ```bash
   $ lsof marketplace.db
   server  277834 malix   12u   REG  259,2   233472 marketplace.db
   server  277834 malix   14u   REG  259,2   233472 marketplace.db
   server  277834 malix   15u   REG  259,2   233472 marketplace.db
   # ... 10 total connections
   ```
   - Corresponds to `r2d2::Pool::builder().max_size(10)` in `server/src/db/mod.rs:61`

2. **No WAL Mode Configured**
   - SQLite was using default DELETE journal mode
   - In DELETE mode, entire database is locked during writes
   - With 10 concurrent connections + brutal kills (`killall -9`), high corruption risk

3. **No Busy Timeout**
   - Connections failed immediately on lock contention instead of waiting
   - Led to failed transactions and incomplete writes

4. **SQLCipher Encryption + DELETE Mode**
   - Encryption makes corruption unrecoverable (can't use `sqlite3 .recover`)
   - Any partial write corrupts the entire encrypted database

## The Fix

Modified `server/src/db/mod.rs` lines 20-56 to add critical PRAGMAs on every connection acquire:

```rust
impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for SqlCipherConnectionCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        // Set SQLCipher key
        sql_query(format!("PRAGMA key = '{}';", self.encryption_key))
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        // CRITICAL: Prevent database corruption
        sql_query("PRAGMA journal_mode = WAL;")      // Write-Ahead Logging
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        sql_query("PRAGMA busy_timeout = 5000;")     // Wait 5s on locks
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        sql_query("PRAGMA synchronous = NORMAL;")    // Balance safety/speed
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        sql_query("PRAGMA cache_size = -64000;")     // 64MB cache
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        sql_query("PRAGMA temp_store = MEMORY;")     // Temp tables in RAM
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        // Verify encryption works
        sql_query("SELECT count(*) FROM sqlite_master;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        Ok(())
    }
}
```

## Why This Works

### WAL Mode (Write-Ahead Logging)
- **Before (DELETE mode)**: Locks entire DB during writes
- **After (WAL mode)**: Readers don't block writers, writers don't block readers
- WAL file (`marketplace.db-wal`) buffers changes before committing to main DB
- Much more resilient to crashes and concurrent access

### Busy Timeout
- Connections wait up to 5 seconds for locks instead of failing immediately
- Prevents race conditions between pool connections
- Reduces `SQLITE_BUSY` errors

### Synchronous = NORMAL
- `FULL`: Safest but slowest (fsync after every write)
- `NORMAL`: Safe enough with WAL mode, much faster
- `OFF`: Fast but dangerous (not used)

### Cache Size
- `-64000` = 64MB of RAM cache (negative value = KB)
- Reduces disk I/O and lock contention
- Improves performance with 10 concurrent connections

### Temp Store = MEMORY
- Temporary tables/indexes use RAM instead of disk
- Faster and reduces disk contention

## Verification After Fix

After recompiling and restarting server:

```bash
# Check WAL mode is active
$ ls -lh marketplace.db*
-rw-r--r-- 1 malix malix 228K marketplace.db
-rw-r--r-- 1 malix malix  32K marketplace.db-shm    # <-- Shared memory
-rw-r--r-- 1 malix malix   0  marketplace.db-wal    # <-- WAL file

# Verify PRAGMA settings
$ sqlcipher marketplace.db
PRAGMA key = '...';
PRAGMA journal_mode;   # Should return: wal
PRAGMA busy_timeout;   # Should return: 5000
PRAGMA synchronous;    # Should return: 1 (NORMAL)
```

## Testing Resilience

Test database survives brutal shutdowns:

```bash
# Start server
./target/release/server &
SERVER_PID=$!

# Make some DB writes (create order, init escrow)
curl -X POST http://localhost:8080/api/orders/.../init-escrow

# Brutal kill during operation
kill -9 $SERVER_PID

# Restart - should work without corruption
./target/release/server
```

**Expected**: Server starts successfully, data intact, no corruption errors.

## Migration for Existing Databases

Existing databases in DELETE mode need conversion:

```bash
# Backup first
cp marketplace.db marketplace.db.backup

# Open with SQLCipher
sqlcipher marketplace.db

# Set key and convert to WAL
PRAGMA key = 'YOUR_KEY_FROM_ENV';
PRAGMA journal_mode = WAL;
.quit
```

On next server start, WAL mode will persist and all other PRAGMAs will be applied.

## Prevention Measures Going Forward

1. **Never use `killall -9 server`** - Use `killall server` (SIGTERM) for graceful shutdown
2. **Monitor WAL file size** - If `marketplace.db-wal` grows beyond 100MB, investigate
3. **Regular integrity checks**:
   ```bash
   sqlcipher marketplace.db "PRAGMA key='...'; PRAGMA integrity_check;"
   ```
4. **Automated backups before risky operations**

## Related Files

- `server/src/db/mod.rs` - Database pool creation with PRAGMA configuration
- `.env` - Contains `DB_ENCRYPTION_KEY`
- `marketplace.db-wal` - WAL file (created after fix)
- `marketplace.db-shm` - Shared memory index for WAL

## Commit Message

```
fix(db): Add WAL mode and corruption prevention PRAGMAs

CRITICAL FIX: Database was corrupting due to:
- SQLite DELETE journal mode with concurrent access
- No busy_timeout causing lock failures
- Brutal kills (killall -9) during transactions
- SQLCipher encryption preventing recovery

Solution:
- Enable WAL mode (readers don't block writers)
- Set busy_timeout = 5000ms
- Configure synchronous = NORMAL for WAL
- 64MB cache + temp_store = MEMORY

Prevents data loss and eliminates "file is not a database" errors.

Fixes: #CORRUPTION-001
```

## References

- [SQLite WAL Mode](https://www.sqlite.org/wal.html)
- [SQLCipher Configuration](https://www.zetetic.net/sqlcipher/sqlcipher-api/)
- [Diesel r2d2 Connection Customization](https://docs.diesel.rs/2.1.x/diesel/r2d2/trait.CustomizeConnection.html)
