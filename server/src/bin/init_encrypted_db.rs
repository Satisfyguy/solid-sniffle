//! Initialize encrypted SQLCipher database and run all migrations
//!
//! This utility creates a fresh encrypted database with SQLCipher
//! and applies all Diesel migrations in the correct order.

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, CustomizeConnection};
use diesel::sql_query;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Debug, Clone)]
struct SqlCipherConnectionCustomizer {
    encryption_key: String,
}

impl CustomizeConnection<SqliteConnection, r2d2::Error> for SqlCipherConnectionCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), r2d2::Error> {
        sql_query(format!("PRAGMA key = '{}';", self.encryption_key))
            .execute(conn)
            .map_err(r2d2::Error::QueryError)?;
        
        // Verify encryption is working
        sql_query("SELECT count(*) FROM sqlite_master;")
            .execute(conn)
            .map_err(r2d2::Error::QueryError)?;
        
        Ok(())
    }
}

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 SQLCipher Database Initialization Utility");
    println!("{}", "=".repeat(70));
    
    // Load environment variables
    dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "marketplace.db".to_string());
    
    let encryption_key = env::var("DB_ENCRYPTION_KEY")
        .expect("❌ DB_ENCRYPTION_KEY not set! Please set it in .env");
    
    println!("📂 Database: {}", database_url);
    println!("🔑 Using encryption key: {}...", &encryption_key[..16]);
    
    // Check if database already exists
    if std::path::Path::new(&database_url).exists() {
        println!("⚠️  Database file already exists!");
        println!("   Delete it first if you want a fresh start:");
        println!("   rm {}", database_url);
        return Err("Database file already exists".into());
    }
    
    println!("🔨 Creating new encrypted SQLCipher database...");
    
    // Create connection pool with SQLCipher encryption
    let manager = ConnectionManager::<SqliteConnection>::new(&database_url);
    let customizer = SqlCipherConnectionCustomizer {
        encryption_key: encryption_key.clone(),
    };
    
    let pool = r2d2::Pool::builder()
        .max_size(1)
        .connection_customizer(Box::new(customizer))
        .build(manager)?;
    
    let mut conn = pool.get()?;
    
    println!("✅ Encrypted database created successfully!");
    println!();
    
    // Run all migrations
    println!("📝 Running Diesel migrations...");
    println!();
    
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    
    println!();
    println!("{}", "=".repeat(70));
    println!("🎉 DATABASE INITIALIZATION COMPLETED!");
    println!();
    println!("✅ Encrypted SQLCipher database created");
    println!("✅ All migrations applied successfully");
    println!();
    println!("🚀 You can now start the server:");
    println!("   ./target/release/server");
    println!("{}", "=".repeat(70));
    
    Ok(())
}
