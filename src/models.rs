use chrono::NaiveDate;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct Release {
    pub id: i32,
    pub date: NaiveDate,
    pub name: String,
    pub directory_name: Option<String>,
    pub file_count: Option<i16>,
    pub size: Option<i64>,
    pub torrent_url: Option<String>,
}
