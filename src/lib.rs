pub mod cumulus;
pub mod error;
pub mod static_data;

use chrono::{NaiveDate, NaiveDateTime};
use std::path::PathBuf;

pub struct Release {
    pub id: String,
    pub date: NaiveDate,
    pub name: String,
    pub file_count: u16,
    pub size: u64,
    pub torrent_url: Option<String>,
}

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
