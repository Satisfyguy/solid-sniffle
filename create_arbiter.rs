// Quick script to create an arbiter user
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

fn main() {
    let password = "arbiter_secure_2024";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    
    println!("Password hash: {}", password_hash);
    println!("\nSQL to insert arbiter:");
    println!("INSERT INTO users (id, username, password_hash, role, created_at, updated_at) VALUES");
    println!("('00000000-0000-0000-0000-000000000001', 'arbiter_system', '{}', 'arbiter', datetime('now'), datetime('now'));", password_hash);
}
