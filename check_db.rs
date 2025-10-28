// Quick DB inspection script
use std::process::Command;

fn main() {
    println!("=== Checking marketplace.db ===\n");

    // Check if DB file exists
    if !std::path::Path::new("marketplace.db").exists() {
        eprintln!("ERROR: marketplace.db not found!");
        return;
    }

    // Use rusqlite to query the database
    let conn = rusqlite::Connection::open("marketplace.db").expect("Failed to open DB");

    // Check listings table structure
    println!("📋 LISTINGS TABLE STRUCTURE:");
    let mut stmt = conn.prepare("PRAGMA table_info(listings)").unwrap();
    let rows = stmt.query_map([], |row| {
        Ok(format!("  - {} | {} | {}",
            row.get::<_, String>(1)?,  // name
            row.get::<_, String>(2)?,  // type
            row.get::<_, i32>(3)?       // notnull
        ))
    }).unwrap();

    for row in rows {
        println!("{}", row.unwrap());
    }

    // Count users by role
    println!("\n👥 USERS COUNT BY ROLE:");
    let mut stmt = conn.prepare("SELECT role, COUNT(*) FROM users GROUP BY role").unwrap();
    let rows = stmt.query_map([], |row| {
        Ok(format!("  - {}: {}",
            row.get::<_, String>(0)?,
            row.get::<_, i32>(1)?
        ))
    }).unwrap();

    for row in rows {
        println!("{}", row.unwrap());
    }

    // List vendor users
    println!("\n🛒 VENDOR USERS:");
    let mut stmt = conn.prepare("SELECT id, username FROM users WHERE role='vendor'").unwrap();
    let rows = stmt.query_map([], |row| {
        Ok(format!("  - {} ({})",
            row.get::<_, String>(1)?,  // username
            row.get::<_, String>(0)?   // id (truncated)
        ))
    }).unwrap();

    for row in rows {
        println!("{}", row.unwrap());
    }

    // Count listings
    println!("\n📦 LISTINGS COUNT:");
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM listings", [], |row| row.get(0)).unwrap();
    println!("  Total: {}", count);

    println!("\n✅ Database inspection complete");
}
