use crate::error::Result;
use crate::models::Release;
use dotenvy::dotenv;
use sqlx::pool::Pool;
use sqlx::postgres::{PgPoolOptions, Postgres};

pub async fn establish_connection() -> Result<Pool<Postgres>> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new().connect(&database_url).await?;
    Ok(pool)
}

pub async fn get_releases() -> Result<Vec<Release>> {
    let pool = establish_connection().await?;
    let releases = sqlx::query_as!(
        Release,
        "SELECT id, date, name, directory_name, file_count, size, torrent_url FROM releases"
    )
    .fetch_all(&pool)
    .await?;
    Ok(releases)
}

pub async fn save_release(release: Release) -> Result<Release> {
    let pool = establish_connection().await?;
    let new_release = sqlx::query_as!(
        Release,
        "INSERT INTO releases (date, name, directory_name, file_count, size, torrent_url)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, date, name, directory_name, file_count, size, torrent_url",
        release.date,
        release.name,
        release.directory_name,
        release.file_count,
        release.size,
        release.torrent_url
    )
    .fetch_one(&pool)
    .await?;
    Ok(new_release)
}
