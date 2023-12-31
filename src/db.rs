use crate::error::Result;
use crate::models::{Content, Image, Photographer, Release, Tag};
use dotenvy::dotenv;
use sqlx::pool::Pool;
use sqlx::postgres::PgPoolOptions;
use sqlx::Postgres;

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

pub async fn save_image(content: Content, image: Image) -> Result<Image> {
    let pool = establish_connection().await?;
    let mut tx = pool.begin().await?;

    let content_id = sqlx::query_as::<_, Content>(
        r#"INSERT INTO content (content_type, file_path, release_id)
        VALUES ($1, $2, $3) RETURNING id, content_type, file_path, release_id"#,
    )
    .bind(content.content_type)
    .bind(content.file_path)
    .bind(content.release_id)
    .fetch_one(&mut *tx)
    .await?
    .id;

    sqlx::query!(
        r#"INSERT INTO images (id, caption, date_recorded, file_metadata, file_size,
        horizontal_pixels, name, notes, received_from, shot_from, vertical_pixels)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#,
        content_id,
        image.caption,
        image.date_recorded,
        image.file_metadata,
        image.file_size,
        image.horizontal_pixels,
        image.name,
        image.notes,
        image.received_from,
        image.shot_from,
        image.vertical_pixels,
    )
    .execute(&mut *tx)
    .await?;

    let mut new_image = Image {
        id: content_id,
        caption: image.caption,
        date_recorded: image.date_recorded,
        file_metadata: image.file_metadata,
        file_size: image.file_size,
        horizontal_pixels: image.horizontal_pixels,
        name: image.name,
        notes: image.notes,
        photographers: Some(Vec::new()),
        received_from: image.received_from,
        shot_from: image.shot_from,
        tags: Some(Vec::new()),
        vertical_pixels: image.vertical_pixels,
    };

    if let Some(tags) = &image.tags {
        for tag in tags {
            let row = sqlx::query!(r#"SELECT id FROM tags WHERE name = $1"#, tag.name)
                .fetch_optional(&mut *tx)
                .await?;
            let tag_id = if let Some(row) = row {
                row.id
            } else {
                sqlx::query!(
                    r#"INSERT INTO tags (name) VALUES ($1) RETURNING id"#,
                    tag.name
                )
                .fetch_one(&mut *tx)
                .await?
                .id
            };

            sqlx::query!(
                r#"INSERT INTO images_tags (image_id, tag_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
                content_id,
                tag_id
            )
            .execute(&mut *tx)
            .await?;

            new_image.add_tag(Tag {
                id: tag_id,
                name: tag.name.clone(),
            });
        }
    }

    if let Some(photographers) = &image.photographers {
        for photographer in photographers {
            let row = sqlx::query!(
                r#"SELECT id FROM photographers WHERE name = $1"#,
                photographer.name
            )
            .fetch_optional(&mut *tx)
            .await?;
            let photographer_id = if let Some(row) = row {
                row.id
            } else {
                sqlx::query!(
                    r#"INSERT INTO photographers (name) VALUES ($1) RETURNING id"#,
                    photographer.name
                )
                .fetch_one(&mut *tx)
                .await?
                .id
            };

            sqlx::query!(
                r#"INSERT INTO images_photographers (image_id, photographer_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
                content_id,
                photographer_id
            )
            .execute(&mut *tx)
            .await?;

            new_image.add_photographer(Photographer {
                id: photographer_id,
                name: photographer.name.clone(),
            });
        }
    }

    tx.commit().await?;

    Ok(new_image)
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
