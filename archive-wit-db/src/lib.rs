mod helpers;

pub mod cumulus;
pub mod error;
pub mod models;

use crate::error::{Error, Result};
use crate::models::{
    Category, Content, Image, JumperTimestamp, MasterVideo, Network, NistTape, NistVideo, Person,
    Photographer, Release, ReleaseFile, Reporter, Tag, Video, Videographer,
};
use csv::ReaderBuilder;
use dotenvy::dotenv;
use sqlx::pool::Pool;
use sqlx::postgres::PgPoolOptions;
use sqlx::Postgres;
use std::{collections::HashMap, path::PathBuf};

pub async fn establish_connection() -> Result<Pool<Postgres>> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new().connect(&database_url).await?;
    Ok(pool)
}

pub async fn get_release(id: i32) -> Result<Release> {
    let pool = establish_connection().await?;
    let rows = sqlx::query!(
        r#"
        SELECT r.id AS release_id, r.date, r.name, r.directory_name, r.file_count,
               r.size AS release_size, r.torrent_url,
               f.id AS file_id, f.path, f.size AS file_size
        FROM releases r
        LEFT JOIN release_files f ON r.id = f.release_id
        WHERE r.id = $1;
        "#,
        id
    )
    .fetch_all(&pool)
    .await?;

    if rows.is_empty() {
        return Err(Error::ReleaseNotFound(id as u32));
    }

    let mut release = Release {
        id: rows[0].release_id,
        date: rows[0].date,
        name: rows[0].name.clone(),
        directory_name: rows[0].directory_name.clone(),
        file_count: rows[0].file_count,
        size: rows[0].release_size,
        torrent_url: rows[0].torrent_url.clone(),
        files: Vec::new(),
    };

    for row in rows {
        release.files.push(ReleaseFile {
            id: row.file_id,
            path: PathBuf::from(row.path),
            size: row.file_size,
        });
    }

    Ok(release)
}

/// Get all releases from the database.
///
/// The files for the releases are not included.
pub async fn get_releases() -> Result<Vec<Release>> {
    let pool = establish_connection().await?;
    let rows = sqlx::query!(
        "SELECT id, date, name, directory_name, file_count, size, torrent_url FROM releases"
    )
    .fetch_all(&pool)
    .await?;

    let mut releases = Vec::new();
    for row in rows {
        releases.push(Release {
            id: row.id,
            date: row.date,
            name: row.name,
            directory_name: row.directory_name,
            file_count: row.file_count,
            size: row.size,
            torrent_url: row.torrent_url,
            files: Vec::new(),
        });
    }
    Ok(releases)
}

pub async fn find_release_files(search_string: &str) -> Result<HashMap<String, Vec<PathBuf>>> {
    let pool = establish_connection().await?;
    let rows = sqlx::query!(
        r#"
        SELECT r.name AS release_name, rf.path, rf.size
        FROM release_files rf
        JOIN releases r ON r.id = rf.release_id
        WHERE rf.path LIKE $1;
        "#,
        format!("%{}%", search_string)
    )
    .fetch_all(&pool)
    .await?;

    if rows.is_empty() {
        return Ok(HashMap::new());
    }

    let mut map: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for row in rows {
        map.entry(row.release_name)
            .or_insert_with(Vec::new)
            .push(PathBuf::from(row.path));
    }
    Ok(map)
}

#[derive(sqlx::FromRow)]
struct VideoQueryResult {
    id: i32,
    title: String,
    date: Option<chrono::NaiveDate>,
    description: Option<String>,
    network_id: Option<i32>,
    network_name: Option<String>,
    category_id: Option<i32>,
    category_name: Option<String>,
    url: Option<String>,
}

pub async fn get_master_video(id: i32, pool: Option<Pool<Postgres>>) -> Result<MasterVideo> {
    let pool = if let Some(p) = pool {
        p
    } else {
        establish_connection().await?
    };

    let rows = sqlx::query_as!(
        VideoQueryResult,
        "SELECT 
            mv.id, mv.title, mv.date, mv.description,
            n.id as network_id, n.name as network_name,
            c.id as category_id, c.name as category_name,
            COALESCE(vu.url, '') as url
        FROM master_videos mv
        LEFT JOIN networks_master_videos nmv ON mv.id = nmv.master_video_id
        LEFT JOIN networks n ON nmv.network_id = n.id
        LEFT JOIN categories_master_videos cmv ON mv.id = cmv.master_video_id
        LEFT JOIN categories c ON cmv.category_id = c.id
        LEFT JOIN master_videos_links vu ON mv.id = vu.master_video_id
        WHERE mv.id = $1",
        &id,
    )
    .fetch_all(&pool)
    .await?;

    if rows.is_empty() {
        return Err(Error::MasterVideoNotFound(id as u32));
    }

    let mut video = MasterVideo {
        id: rows[0].id,
        title: rows[0].title.clone(),
        date: rows[0].date,
        description: rows[0].description.clone(),
        categories: Vec::new(),
        networks: Vec::new(),
        links: Vec::new(),
    };

    // This loop runs for each network *and* category returned, so you need to make sure you don't
    // add duplicates.
    for row in &rows {
        if let (Some(network_id), Some(network_name)) = (row.network_id, &row.network_name) {
            if let None = video.networks.iter().find(|n| n.name == *network_name) {
                video.networks.push(Network {
                    id: network_id,
                    name: network_name.to_string(),
                });
            }
        }
        if let (Some(category_id), Some(category_name)) = (row.category_id, &row.category_name) {
            if let None = video.categories.iter().find(|c| c.name == *category_name) {
                video.categories.push(Category {
                    id: category_id,
                    name: category_name.to_string(),
                });
            }
        }
        if let Some(url) = &row.url {
            if !url.is_empty() {
                video.links.push(url.to_string());
            }
        }
    }

    Ok(video)
}

pub async fn get_master_videos() -> Result<Vec<MasterVideo>> {
    let pool = establish_connection().await?;
    let mut videos = Vec::new();

    let rows = sqlx::query!(
        "SELECT 
            mv.id, mv.title, mv.date, mv.description,
            n.id as network_id, n.name as network_name,
            c.id as category_id, c.name as category_name,
            vu.url
        FROM master_videos mv
        LEFT JOIN networks_master_videos nmv ON mv.id = nmv.master_video_id
        LEFT JOIN networks n ON nmv.network_id = n.id
        LEFT JOIN categories_master_videos cmv ON mv.id = cmv.master_video_id
        LEFT JOIN categories c ON cmv.category_id = c.id
        LEFT JOIN master_videos_links vu ON mv.id = vu.master_video_id"
    )
    .fetch_all(&pool)
    .await?;

    let mut video_map = std::collections::HashMap::new();
    for row in rows {
        let entry = video_map.entry(row.id).or_insert_with(|| MasterVideo {
            categories: vec![],
            date: row.date,
            description: row.description,
            id: row.id.unwrap(),
            links: vec![],
            networks: vec![],
            title: row.title.unwrap(),
        });

        if let Some(network_id) = row.network_id {
            entry.networks.push(Network {
                id: network_id,
                name: row.network_name.unwrap(),
            });
        }
        if let Some(category_id) = row.category_id {
            entry.categories.push(Category {
                id: category_id,
                name: row.category_name.unwrap(),
            });
        }

        if let Some(url) = row.url {
            entry.links.push(url);
        }
    }
    videos.extend(video_map.into_values());
    videos.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(videos)
}

pub async fn get_video(id: i32) -> Result<Video> {
    let pool = establish_connection().await?;
    let row = sqlx::query!(
        r#"SELECT id, title, description, timestamps, duration, link, nist_notes, master_id
         FROM videos WHERE id = $1"#,
        &id,
    )
    .fetch_one(&pool)
    .await?;

    let master = get_master_video(row.master_id, Some(pool.clone())).await?;
    let mut video = Video {
        id: row.id,
        title: row.title,
        description: row.description,
        timestamps: row.timestamps,
        duration: row.duration,
        link: row.link,
        nist_notes: row.nist_notes,
        master,
        videographers: Vec::new(),
        reporters: Vec::new(),
        people: Vec::new(),
        jumper_timestamps: Vec::new(),
        nist_files: Vec::new(),
    };

    let rows = sqlx::query_as!(
        Videographer,
        r#"SELECT v.id, v.name FROM videographers v
        JOIN videos_videographers vv ON v.id = vv.videographer_id
        WHERE vv.video_id = $1;"#,
        &id,
    )
    .fetch_all(&pool)
    .await?;
    for row in rows {
        video.videographers.push(row)
    }

    let rows = sqlx::query_as!(
        Videographer,
        r#"SELECT v.id, v.name FROM videographers v
        JOIN videos_videographers vv ON v.id = vv.videographer_id
        WHERE vv.video_id = $1;"#,
        &id,
    )
    .fetch_all(&pool)
    .await?;
    for row in rows {
        video.videographers.push(row)
    }

    let rows = sqlx::query_as!(
        Reporter,
        r#"SELECT r.id, r.name FROM reporters r
        JOIN videos_reporters rr ON r.id = rr.reporter_id
        WHERE rr.video_id = $1;"#,
        &id,
    )
    .fetch_all(&pool)
    .await?;
    for row in rows {
        video.reporters.push(row)
    }

    let rows = sqlx::query_as!(
        Person,
        r#"SELECT p.id, p.name, p.historical_title FROM people p
        JOIN videos_people pp ON p.id = pp.person_id
        WHERE pp.video_id = $1;"#,
        &id,
    )
    .fetch_all(&pool)
    .await?;
    for row in rows {
        video.people.push(row)
    }

    let rows = sqlx::query_as!(
        JumperTimestamp,
        r#"SELECT jt.id, jt.timestamp FROM jumper_timestamps jt
        JOIN videos_jumper_timestamps vjt ON jt.id = vjt.jumper_timestamp_id
        WHERE vjt.video_id = $1;"#,
        &id,
    )
    .fetch_all(&pool)
    .await?;
    for row in rows {
        video.jumper_timestamps.push(row)
    }

    let rows = sqlx::query_as!(
        ReleaseFile,
        r#"
        SELECT rf.id, rf.path, rf.size
        FROM release_files rf
        JOIN videos_release_files vrf ON rf.id = vrf.release_file_id
        WHERE vrf.video_id = $1;
        "#,
        &id,
    )
    .fetch_all(&pool)
    .await?;
    for row in rows {
        video.nist_files.push((row.path, row.size as u64))
    }

    Ok(video)
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

pub async fn import_nist_video_table_from_csv(csv_path: PathBuf) -> Result<()> {
    let pool = establish_connection().await?;
    let mut tx = pool.begin().await?;

    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(csv_path)?;
    for result in rdr.deserialize() {
        let record: Vec<String> = result?;
        let video = NistVideo::try_from(record)
            .map_err(|e| Error::NistVideoConversionError(e.to_string()))?;
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

pub async fn import_nist_tapes_table_from_csv(csv_path: PathBuf) -> Result<()> {
    let pool = establish_connection().await?;
    let mut tx = pool.begin().await?;

    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(csv_path)?;
    for result in rdr.deserialize() {
        let record: Vec<String> = result?;
        let tape = NistTape::try_from(record)
            .map_err(|e| Error::NistTapeConversionError(e.to_string()))?;
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

pub async fn save_master_video(video: MasterVideo) -> Result<MasterVideo> {
    let pool = establish_connection().await?;
    let mut tx = pool.begin().await?;

    // Unfortunately, you need to handle the special case where the ID is zero, which is for a new
    // record. Postgres allows the insertion of 0, despite the fact that the ID column is defined
    // as `SERIAL`.
    let video_id = if video.id == 0 {
        let video_id = sqlx::query!(
            r#"INSERT INTO master_videos (title, date, description)
               VALUES ($1, $2, $3)
               RETURNING id"#,
            video.title,
            video.date,
            video.description,
        )
        .fetch_one(&mut *tx)
        .await?
        .id;
        video_id
    } else {
        let video_id = sqlx::query!(
            r#"INSERT INTO master_videos (id, title, date, description)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (id) DO UPDATE SET
               title = EXCLUDED.title, date = EXCLUDED.date, description = EXCLUDED.description
               RETURNING id"#,
            video.id,
            video.title,
            video.date,
            video.description,
        )
        .fetch_one(&mut *tx)
        .await?
        .id;
        video_id
    };

    let mut updated_video = video.clone();
    updated_video.id = video_id;

    for network in video.networks.iter() {
        // When a video is being updated, it could have had a new network added; however, it was
        // added using its name, and at that point, we didn't have access to the ID, so we need to
        // get it now. The list of networks is static, so the name must refer to a network that
        // already exists. If it doesn't, we can't insert the new video.
        let row = sqlx::query_as!(
            Network,
            "SELECT * FROM networks WHERE name = $1",
            network.name
        )
        .fetch_one(&mut *tx)
        .await?;

        let updated_network = updated_video
            .networks
            .iter_mut()
            .find(|n| n.name == network.name)
            .unwrap();
        updated_network.id = row.id;

        sqlx::query!(
            r#"INSERT INTO networks_master_videos (network_id, master_video_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
            row.id,
            video_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    for category in video.categories.iter() {
        // As for networks, the same applies for categories.
        let row = sqlx::query_as!(
            Category,
            "SELECT * FROM categories WHERE name = $1",
            category.name
        )
        .fetch_one(&mut *tx)
        .await?;

        let updated_category = updated_video
            .categories
            .iter_mut()
            .find(|c| c.name == category.name)
            .unwrap();
        updated_category.id = row.id;

        sqlx::query!(
            r#"INSERT INTO categories_master_videos (category_id, master_video_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
            row.id,
            video_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    for url in video.links.iter() {
        sqlx::query!(
            r#"INSERT INTO master_videos_links (master_video_id, url)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
            video_id,
            url
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(updated_video)
}

pub async fn save_video(video: Video) -> Result<Video> {
    let pool = establish_connection().await?;
    let mut tx = pool.begin().await?;

    // Unfortunately, you need to handle the special case where the ID is zero, which is for a new
    // record. Postgres allows the insertion of 0, despite the fact that the ID column is defined
    // as `SERIAL`.
    let video_id = if video.id == 0 {
        let video_id = sqlx::query!(
            r#"INSERT INTO videos (
                    title, description, timestamps, duration, link, nist_notes, master_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id"#,
            video.title,
            video.description,
            video.timestamps,
            video.duration,
            video.link,
            video.nist_notes,
            video.master.id,
        )
        .fetch_one(&mut *tx)
        .await?
        .id;
        video_id
    } else {
        let video_id = sqlx::query!(
            r#"INSERT INTO videos (
                    id, title, description, timestamps, duration, link, nist_notes, master_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (id) DO UPDATE SET
                   title = EXCLUDED.title,
                   description = EXCLUDED.description,
                   timestamps = EXCLUDED.timestamps,
                   duration = EXCLUDED.duration,
                   link = EXCLUDED.link,
                   nist_notes = EXCLUDED.nist_notes,
                   master_id = EXCLUDED.master_id
               RETURNING id"#,
            video.id,
            video.title,
            video.description,
            video.timestamps,
            video.duration,
            video.link,
            video.nist_notes,
            video.master.id,
        )
        .fetch_one(&mut *tx)
        .await?
        .id;
        video_id
    };

    let mut updated_video = video.clone();
    updated_video.id = video_id;

    for videographer in video.videographers.iter() {
        let row = sqlx::query!(
            "SELECT id FROM videographers WHERE name = $1",
            videographer.name
        )
        .fetch_optional(&mut *tx)
        .await?;

        let videographer_id = if let Some(row) = row {
            row.id
        } else {
            sqlx::query!(
                r#"INSERT INTO videographers (name) VALUES ($1) RETURNING id"#,
                videographer.name
            )
            .fetch_one(&mut *tx)
            .await?
            .id
        };

        let updated_videographer = updated_video
            .videographers
            .iter_mut()
            .find(|n| n.name == videographer.name)
            .unwrap();
        updated_videographer.id = videographer_id;

        sqlx::query!(
            r#"INSERT INTO videos_videographers (video_id, videographer_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
            video_id,
            videographer_id
        )
        .execute(&mut *tx)
        .await?;
    }

    for reporter in video.reporters.iter() {
        let row = sqlx::query!("SELECT id FROM reporters WHERE name = $1", reporter.name)
            .fetch_optional(&mut *tx)
            .await?;

        let reporter_id = if let Some(row) = row {
            row.id
        } else {
            sqlx::query!(
                "INSERT INTO reporters (name) VALUES ($1) RETURNING id",
                reporter.name
            )
            .fetch_one(&mut *tx)
            .await?
            .id
        };

        let updated_reporter = updated_video
            .reporters
            .iter_mut()
            .find(|r| r.name == reporter.name)
            .unwrap();
        updated_reporter.id = reporter_id;

        sqlx::query!(
            r#"INSERT INTO videos_reporters (video_id, reporter_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
            video_id,
            reporter_id
        )
        .execute(&mut *tx)
        .await?;
    }

    for person in video.people.iter() {
        let row = sqlx::query!("SELECT id FROM people WHERE name = $1", person.name)
            .fetch_optional(&mut *tx)
            .await?;

        let person_id = if let Some(row) = row {
            row.id
        } else {
            sqlx::query!(
                r#"INSERT INTO people (name) VALUES ($1) RETURNING id"#,
                person.name
            )
            .fetch_one(&mut *tx)
            .await?
            .id
        };

        let updated_person = updated_video
            .people
            .iter_mut()
            .find(|p| p.name == person.name)
            .unwrap();
        updated_person.id = person_id;

        sqlx::query!(
            r#"INSERT INTO videos_people (video_id, person_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
            video_id,
            person_id
        )
        .execute(&mut *tx)
        .await?;
    }

    for jt in video.jumper_timestamps.iter() {
        let jt_id = if jt.id == 0 {
            sqlx::query!(
                r#"INSERT INTO jumper_timestamps (timestamp)
                    VALUES ($1)
                    RETURNING id"#,
                jt.timestamp
            )
            .fetch_one(&mut *tx)
            .await?
            .id
        } else {
            jt.id
        };

        sqlx::query!(
            r#"INSERT INTO videos_jumper_timestamps (video_id, jumper_timestamp_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
            video_id,
            jt_id
        )
        .execute(&mut *tx)
        .await?;
    }

    for i in 0..video.nist_files.len() {
        let path = video.nist_files[i].0.clone();
        let row = sqlx::query!(
            "SELECT id, path, size FROM release_files WHERE path = $1",
            &path.to_string_lossy()
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"INSERT INTO videos_release_files (video_id, release_file_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
            video_id,
            row.id
        )
        .execute(&mut *tx)
        .await?;

        updated_video.nist_files[i] = (path, row.size as u64);
    }

    tx.commit().await?;

    Ok(updated_video)
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

/// Saves a NIST release in the database.
///
/// Saving a release does not need to be an 'upsert' operation because release content is static.
/// They should only be initialised once.
pub async fn save_release(release: Release) -> Result<Release> {
    let pool = establish_connection().await?;
    let mut tx = pool.begin().await?;

    let release_id = sqlx::query!(
        "INSERT INTO releases (date, name, directory_name, file_count, size, torrent_url)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id",
        release.date,
        release.name,
        release.directory_name,
        release.file_count,
        release.size,
        release.torrent_url
    )
    .fetch_one(&mut *tx)
    .await?
    .id;

    let mut updated_release = release.clone();
    updated_release.id = release_id;

    for i in 0..release.files.len() {
        let id = sqlx::query!(
            "INSERT INTO release_files (path, size, release_id)
             VALUES ($1, $2, $3)
             RETURNING id",
            &release.files[i].path.to_string_lossy(),
            release.files[i].size,
            release_id
        )
        .fetch_one(&mut *tx)
        .await?
        .id;
        updated_release.files[i].id = id;
    }

    tx.commit().await?;

    Ok(updated_release)
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
