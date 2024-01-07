use crate::error::Result;
use crate::models::{Content, Image, MasterVideo, NistTape, NistVideo, Photographer, Release, Tag};
use csv::ReaderBuilder;
use dotenvy::dotenv;
use sqlx::pool::Pool;
use sqlx::postgres::PgPoolOptions;
use sqlx::Postgres;
use std::path::PathBuf;

pub async fn establish_connection() -> Result<Pool<Postgres>> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new().connect(&database_url).await?;
    Ok(pool)
}

pub async fn get_release(release_id: i32) -> Result<Release> {
    let pool = establish_connection().await?;
    let release = sqlx::query_as!(
        Release,
        r#"SELECT id, date, name, directory_name, file_count, size, torrent_url
           FROM releases
           WHERE id = $1
        "#,
        release_id
    )
    .fetch_one(&pool)
    .await?;
    Ok(release)
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

pub async fn get_master_videos() -> Result<Vec<MasterVideo>> {
    let pool = establish_connection().await?;
    let videos = sqlx::query_as!(
        MasterVideo,
        "SELECT id, title, date, description, format, network, source, notes FROM master_videos"
    )
    .fetch_all(&pool)
    .await?;
    Ok(videos)
}

pub async fn get_nist_videos() -> Result<Vec<NistVideo>> {
    let pool = establish_connection().await?;
    let videos = sqlx::query_as!(
        NistVideo,
        "SELECT video_id, video_title, network, broadcast_date, duration_min, subject, notes FROM nist_videos"
    )
    .fetch_all(&pool)
    .await?;
    Ok(videos)
}

pub async fn get_nist_tapes() -> Result<Vec<NistTape>> {
    let pool = establish_connection().await?;
    let videos = sqlx::query_as!(
        NistTape,
        "SELECT tape_id, video_id, tape_name, tape_source, copy, derived_from, format, duration_min, batch, clips, timecode FROM nist_tapes"
    )
    .fetch_all(&pool)
    .await?;
    Ok(videos)
}

pub async fn get_torrent_content(release_id: i32) -> Result<Option<Vec<u8>>> {
    let pool = establish_connection().await?;
    let row = sqlx::query!(
        "SELECT content FROM release_torrents WHERE release_id = $1",
        release_id
    )
    .fetch_optional(&pool)
    .await?;
    match row {
        Some(row) => Ok(Some(row.content)),
        None => Ok(None),
    }
}

pub async fn import_nist_video_table_from_csv(csv_path: PathBuf) -> color_eyre::Result<()> {
    let pool = establish_connection().await?;
    let mut tx = pool.begin().await?;

    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(csv_path)?;
    for result in rdr.deserialize() {
        let record: Vec<String> = result?;
        let video = NistVideo::try_from(record).map_err(|e| color_eyre::eyre::eyre!(e))?;
        sqlx::query_as!(
            NistVideo,
            r#"INSERT INTO nist_videos (
                video_id, 
                video_title, 
                network, 
                broadcast_date, 
                duration_min, 
                subject, 
                notes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            video.video_id,
            video.video_title,
            video.network,
            video.broadcast_date,
            video.duration_min,
            video.subject,
            video.notes,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn import_nist_tapes_table_from_csv(csv_path: PathBuf) -> color_eyre::Result<()> {
    let pool = establish_connection().await?;
    let mut tx = pool.begin().await?;

    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(csv_path)?;
    for result in rdr.deserialize() {
        let record: Vec<String> = result?;
        let tape = NistTape::try_from(record).map_err(|e| color_eyre::eyre::eyre!(e))?;
        sqlx::query_as!(
            NistTape,
            r#"INSERT INTO nist_tapes (
                tape_id, 
                video_id, 
                tape_name, 
                tape_source, 
                copy, 
                derived_from, 
                format, 
                duration_min, 
                batch, 
                clips, 
                timecode
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#,
            tape.tape_id,
            tape.video_id,
            tape.tape_name,
            tape.tape_source,
            tape.copy,
            tape.derived_from,
            tape.format,
            tape.duration_min,
            tape.batch,
            tape.clips,
            tape.timecode,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn save_master_video_list(master_videos: Vec<MasterVideo>) -> Result<()> {
    let pool = establish_connection().await?;
    let mut tx = pool.begin().await?;

    for video in master_videos.iter() {
        sqlx::query_as!(
            MasterVideo,
            r#"INSERT INTO master_videos (
                title, 
                date, 
                description, 
                format, 
                network, 
                source, 
                notes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            video.title,
            video.date,
            video.description,
            video.format,
            video.network,
            video.source,
            video.notes,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
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

pub async fn save_torrent(release_id: i32, torrent_path: &PathBuf) -> Result<()> {
    let pool = establish_connection().await?;
    let content = std::fs::read(torrent_path)?;
    let query = sqlx::query!(
        "INSERT INTO release_torrents (release_id, content) VALUES ($1, $2)",
        release_id,
        content
    );
    query.execute(&pool).await?;
    Ok(())
}
