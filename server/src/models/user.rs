use crate::schema::users;
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::OptionalExtension;
use serde::Serialize;

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: &'a str,
    pub username: &'a str,
    pub password_hash: &'a str,
    pub role: &'a str,
}

#[derive(AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUser<'a> {
    pub username: Option<&'a str>,
    pub password_hash: Option<&'a str>,
    pub role: Option<&'a str>,
}

use crate::db::DbPool;
use actix_web::web;
use anyhow::Result; // For web::block

impl User {
    pub async fn create(db: &DbPool, new_user: NewUser<'_>) -> Result<Self> {
        let user_id = new_user.id.to_string(); // Store ID before move
        web::block(move || {
            let mut conn = db.get().map_err(anyhow::Error::from)?;
            diesel::insert_into(users::table)
                .values(new_user)
                .execute(&mut conn)?;
            Ok(())
        })
        .await??;
        // Fetch the created user by ID
        User::find_by_id(db, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to find created user"))
    }

    pub async fn find_by_id(db: &DbPool, user_id: String) -> Result<Option<Self>> {
        let user = web::block(move || {
            let mut conn = db.get().map_err(anyhow::Error::from)?;
            users::table.find(user_id).first(&mut conn).optional()
        })
        .await??;
        Ok(user)
    }

    pub async fn find_by_username(db: &DbPool, user_username: String) -> Result<Option<Self>> {
        let user = web::block(move || {
            let mut conn = db.get().map_err(anyhow::Error::from)?;
            users::table
                .filter(users::username.eq(user_username))
                .first(&mut conn)
                .optional()
        })
        .await??;
        Ok(user)
    }

    pub async fn update(db: &DbPool, user_id: String, update_user: UpdateUser<'_>) -> Result<Self> {
        web::block(move || {
            let mut conn = db.get().map_err(anyhow::Error::from)?;
            diesel::update(users::table.find(user_id.clone()))
                .set(update_user)
                .execute(&mut conn)?;
            Ok(())
        })
        .await??;
        // Fetch the updated user by ID
        User::find_by_id(db, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to find updated user"))
    }

    pub async fn delete(db: &DbPool, user_id: String) -> Result<usize> {
        let count = web::block(move || {
            let mut conn = db.get().map_err(anyhow::Error::from)?;
            diesel::delete(users::table.find(user_id)).execute(&mut conn)
        })
        .await??;
        Ok(count)
    }
}
