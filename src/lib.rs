pub mod cumulus;
pub mod error;
pub mod models;
pub mod releases;
pub mod schema;
pub mod static_data;

use crate::error::Result;
use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::path::PathBuf;

pub struct Content {}

pub struct ImageContent {
    pub album: String,
    pub caption: String,
    pub date_recorded: NaiveDateTime,
    pub file_size: u64,
    pub horizontal_pixels: u16,
    pub name: String,
    pub notes: String,
    pub photographers: Vec<String>,
    pub received_from: String,
    pub release_path: PathBuf,
    pub shot_from: String,
    pub tags: Vec<String>,
    pub vertical_pixels: u16,
}

pub fn establish_connection() -> Result<PgConnection> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")?;
    let connection = PgConnection::establish(&database_url)?;
    Ok(connection)
}
