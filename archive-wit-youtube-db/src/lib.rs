pub mod error;
pub mod models;

use crate::{error::Result, models::YouTubeVideo};
use dotenvy::dotenv;
use sqlx::{pool::Pool, sqlite::SqlitePoolOptions, Sqlite};
use std::path::PathBuf;

pub async fn establish_connection() -> Result<Pool<Sqlite>> {
    dotenv().ok();
    let database_url = std::env::var("YOUTUBE_DB_URL")?;
    let pool = SqlitePoolOptions::new().connect(&database_url).await?;
    Ok(pool)
}

pub async fn get_video(id: &str) -> Result<YouTubeVideo> {
    let pool = establish_connection().await?;
    let row = sqlx::query!(
        r#"
        SELECT 
            videos.id,
            videos.title,
            videos.duration,
            videos.saved_path,
            channels.title AS channel_title
        FROM videos
        JOIN channels ON videos.channel_id = channels.id
        WHERE videos.id = $1;
        "#,
        id
    )
    .fetch_one(&pool)
    .await?;

    let video = YouTubeVideo::new(
        &row.channel_title.unwrap(),
        row.duration,
        &row.id.unwrap(),
        row.saved_path.map(PathBuf::from),
        &row.title,
    )?;
    Ok(video)
}
