use super::{db, models::user::*, crypto::encryption::*};
use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use diesel::Connection;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");

async fn setup_test_db() -> Result<db::DbPool> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "test_database.sqlite".to_string());
    
    // Ensure a clean database for each test
    if std::path::Path::new(&database_url).exists() {
        std::fs::remove_file(&database_url)?;
    }

    let pool = db::create_pool(&database_url)?;
    
    // Run migrations
    let mut conn = pool.get()?;
    conn.run_pending_migrations(MIGRATIONS).unwrap();

    Ok(pool)
}

#[tokio::test]
async fn test_user_crud() -> Result<()> {
    let pool = setup_test_db().await?;

    // Test Create
    let new_user_data = NewUser {
        id: "test_user_id_1",
        username: "testuser",
        password_hash: "hashed_password",
        role: "buyer",
    };
    let user = User::create(&pool, new_user_data).await?;
    assert_eq!(user.username, "testuser");
    assert_eq!(user.role, "buyer");

    // Test Read by ID
    let fetched_user_by_id = User::find_by_id(&pool, user.id.clone()).await?;
    assert!(fetched_user_by_id.is_some());
    assert_eq!(fetched_user_by_id.unwrap().username, "testuser");

    // Test Read by Username
    let fetched_user_by_username = User::find_by_username(&pool, "testuser".to_string()).await?;
    assert!(fetched_user_by_username.is_some());
    assert_eq!(fetched_user_by_username.unwrap().id, user.id);

    // Test Update
    let updated_user_data = UpdateUser {
        username: Some("updated_testuser"),
        password_hash: None,
        role: None,
    };
    let updated_user = User::update(&pool, user.id.clone(), updated_user_data).await?;
    assert_eq!(updated_user.username, "updated_testuser");

    // Test Delete
    let deleted_count = User::delete(&pool, user.id.clone()).await?;
    assert_eq!(deleted_count, 1);
    let deleted_user = User::find_by_id(&pool, user.id).await?;
    assert!(deleted_user.is_none());

    Ok(())
}

#[tokio::test]
async fn test_encryption_decryption() -> Result<()> {
    let key = generate_key();
    let plaintext = "This is a secret message.";

    let encrypted = encrypt_field(plaintext, &key)?;
    assert_ne!(encrypted, plaintext); // Should not be the same

    let decrypted = decrypt_field(&encrypted, &key)?;
    assert_eq!(decrypted, plaintext);

    // Test with a different key (should fail)
    let wrong_key = generate_key();
    let decrypt_fail_result = decrypt_field(&encrypted, &wrong_key);
    assert!(decrypt_fail_result.is_err());

    Ok(())
}
