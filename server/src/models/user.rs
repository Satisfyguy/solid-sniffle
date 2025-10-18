//! User model and related database operations

use diesel::prelude::*;
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use anyhow::{Context, Result};

use crate::schema::users;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = users)]
pub struct User {
    #[diesel(column_name = "id")]
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: String,
}

impl User {
    /// Create a new user in the database
    pub fn create(conn: &mut SqliteConnection, new_user: NewUser) -> Result<User> {
        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(conn)
            .context("Failed to insert user")?;

        users::table
            .filter(users::id.eq(new_user.id))
            .first(conn)
            .context("Failed to retrieve created user")
    }

    /// Find user by ID
    pub fn find_by_id(conn: &mut SqliteConnection, user_id: Uuid) -> Result<User> {
        users::table
            .filter(users::id.eq(user_id))
            .first(conn)
            .context(format!("User with ID {} not found", user_id))
    }

    /// Find user by username
    pub fn find_by_username(conn: &mut SqliteConnection, username_str: &str) -> Result<User> {
        users::table
            .filter(users::username.eq(username_str))
            .first(conn)
            .context(format!("User with username '{}' not found", username_str))
    }

    /// Check if username already exists
    pub fn username_exists(conn: &mut SqliteConnection, username_str: &str) -> Result<bool> {
        let count: i64 = users::table
            .filter(users::username.eq(username_str))
            .count()
            .get_result(conn)
            .context("Failed to check username existence")?;
        Ok(count > 0)
    }

    /// Update user's updated_at timestamp
    pub fn touch(conn: &mut SqliteConnection, user_id: Uuid) -> Result<()> {
        diesel::update(users::table.filter(users::id.eq(user_id)))
            .set(users::updated_at.eq(diesel::dsl::now))
            .execute(conn)
            .context("Failed to update user timestamp")?;
        Ok(())
    }

    /// Delete user by ID
    pub fn delete(conn: &mut SqliteConnection, user_id: Uuid) -> Result<()> {
        diesel::delete(users::table.filter(users::id.eq(user_id)))
            .execute(conn)
            .context(format!("Failed to delete user {}", user_id))?;
        Ok(())
    }

    /// List all users with a specific role
    pub fn find_by_role(conn: &mut SqliteConnection, role_str: &str) -> Result<Vec<User>> {
        users::table
            .filter(users::role.eq(role_str))
            .load(conn)
            .context(format!("Failed to load users with role '{}'", role_str))
    }
}