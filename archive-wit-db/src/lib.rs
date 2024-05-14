pub mod cumulus;
pub mod error;
pub mod helpers;
pub mod models;

use crate::error::{Error, Result};
use crate::models::{
    Category, Content, EventTimestamp, EventType, Image, MasterVideo, NewsAffiliate, NewsBroadcast,
    NewsNetwork, NistTape, NistVideo, Person, PersonType, Photographer, Release, ReleaseFile, Tag,
    Video,
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

/// ***********************
/// Read-based queries
/// ***********************

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
            .or_default()
            .push(PathBuf::from(row.path));
    }
    Ok(map)
}

pub async fn get_master_videos() -> Result<Vec<MasterVideo>> {
    let pool = establish_connection().await?;
    let rows = sqlx::query!("SELECT id FROM master_videos")
        .fetch_all(&pool)
        .await?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row.id);
    }

    let mut masters = Vec::new();
    for id in ids.iter() {
        masters.push(get_master_video(*id, Some(pool.clone())).await?);
    }

    Ok(masters)
}

pub async fn get_master_video(id: i32, pool: Option<Pool<Postgres>>) -> Result<MasterVideo> {
    let pool = if let Some(p) = pool {
        p
    } else {
        establish_connection().await?
    };

    let row = sqlx::query!(
        r#"
            SELECT id, categories as "categories: Vec<Category>", title, date, description, links,
            nist_notes FROM master_videos
            WHERE id = $1
        "#,
        id
    )
    .fetch_one(&pool)
    .await?;

    let mut master = MasterVideo {
        id: row.id,
        categories: row.categories,
        date: row.date,
        description: row.description,
        links: if let Some(links) = row.links {
            links
        } else {
            Vec::new()
        },
        people: Vec::new(),
        news_broadcasts: Vec::new(),
        nist_files: Vec::new(),
        nist_notes: row.nist_notes,
        timestamps: Vec::new(),
        title: row.title,
    };

    let rows = sqlx::query!(
        r#"
            SELECT id, description, timestamp, event_type as "event_type: EventType", time_of_day
            FROM event_timestamps
            WHERE master_video_id = $1
        "#,
        id
    )
    .fetch_all(&pool)
    .await?;
    for row in rows {
        master.timestamps.push(EventTimestamp {
            id: row.id,
            description: row.description,
            timestamp: row.timestamp,
            event_type: row.event_type,
            time_of_day: row.time_of_day,
        })
    }

    let news_networks = get_news_networks(Some(pool.clone())).await?;
    let news_affiliates = get_news_affiliates(Some(pool.clone())).await?;
    let rows = sqlx::query!(
        r#"
            SELECT nb.*
            FROM news_broadcasts nb
            JOIN master_videos_news_broadcasts mvnb ON nb.id = mvnb.news_broadcast_id
            WHERE mvnb.master_video_id = $1;
        "#,
        id
    )
    .fetch_all(&pool)
    .await?;
    for row in rows {
        if let Some(network_id) = row.news_network_id {
            let network = news_networks.iter().find(|n| n.id == network_id).unwrap();
            master.news_broadcasts.push(NewsBroadcast {
                id: row.id,
                date: row.date,
                description: row.description,
                news_network: Some(network.clone()),
                news_affiliate: None,
            });
        } else if let Some(affiliate_id) = row.news_affiliate_id {
            let affiliate = news_affiliates
                .iter()
                .find(|n| n.id == affiliate_id)
                .unwrap();
            master.news_broadcasts.push(NewsBroadcast {
                id: row.id,
                date: row.date,
                description: row.description,
                news_network: None,
                news_affiliate: Some(affiliate.clone()),
            });
        }
    }

    let rows = sqlx::query!(
        r#"
            SELECT
                p.id,
                p.name,
                p.description,
                p.historical_title,
                p.types as "types: Vec<PersonType>"
            FROM people p
            JOIN master_videos_people mvp ON p.id = mvp.person_id
            WHERE mvp.master_video_id = $1;
        "#,
        id
    )
    .fetch_all(&pool)
    .await?;
    for row in rows {
        master.people.push(Person {
            id: row.id,
            name: row.name,
            description: row.description,
            historical_title: row.historical_title,
            types: row.types,
        });
    }

    let rows = sqlx::query!(
        r#"
            SELECT rf.path, rf.size
            FROM release_files rf
            JOIN master_videos_release_files mvrf ON rf.id = mvrf.release_file_id
            WHERE mvrf.master_video_id = $1;
        "#,
        id
    )
    .fetch_all(&pool)
    .await?;
    for row in rows {
        master
            .nist_files
            .push((PathBuf::from(row.path), row.size as u64));
    }

    Ok(master)
}

pub async fn get_video(id: i32, pool: Option<Pool<Postgres>>) -> Result<Video> {
    let pool = if let Some(p) = pool {
        p
    } else {
        establish_connection().await?
    };

    let row = sqlx::query!(
        r#"
            SELECT id, description, duration, is_primary, link, master_id, title FROM videos
            WHERE id = $1
        "#,
        id
    )
    .fetch_one(&pool)
    .await?;

    let master = get_master_video(row.id, Some(pool.clone())).await?;
    let video = Video {
        description: row.description,
        duration: row.duration,
        id: row.id,
        is_primary: row.is_primary,
        link: row.link,
        master: master.clone(),
        title: row.title,
    };
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

pub async fn get_news_networks(pool: Option<Pool<Postgres>>) -> Result<Vec<NewsNetwork>> {
    let pool = if let Some(p) = pool {
        p
    } else {
        establish_connection().await?
    };
    let news_networks = sqlx::query_as!(
        NewsNetwork,
        "SELECT id, name, description FROM news_networks"
    )
    .fetch_all(&pool)
    .await?;
    Ok(news_networks)
}

pub async fn get_news_affiliates(pool: Option<Pool<Postgres>>) -> Result<Vec<NewsAffiliate>> {
    let pool = if let Some(p) = pool {
        p
    } else {
        establish_connection().await?
    };

    let news_networks = get_news_networks(Some(pool.clone())).await?;
    let mut news_affiliates = Vec::new();
    let rows =
        sqlx::query!("SELECT id, name, description, region, news_network_id FROM news_affiliates")
            .fetch_all(&pool)
            .await?;
    for row in rows {
        let network = news_networks
            .iter()
            .find(|n| n.id == row.news_network_id)
            .unwrap();
        news_affiliates.push(NewsAffiliate {
            id: row.id,
            name: row.name,
            description: row.description,
            region: row.region,
            network: network.clone(),
        });
    }
    Ok(news_affiliates)
}

pub async fn get_news_broadcasts() -> Result<Vec<NewsBroadcast>> {
    let pool = establish_connection().await?;

    let news_networks = sqlx::query_as!(
        NewsNetwork,
        "SELECT id, name, description FROM news_networks"
    )
    .fetch_all(&pool)
    .await?;

    let mut news_affiliates = Vec::new();
    let rows =
        sqlx::query!("SELECT id, name, description, region, news_network_id FROM news_affiliates")
            .fetch_all(&pool)
            .await?;
    for row in rows {
        let network = news_networks
            .iter()
            .find(|n| n.id == row.news_network_id)
            .unwrap();
        news_affiliates.push(NewsAffiliate {
            id: row.id,
            name: row.name,
            description: row.description,
            region: row.region,
            network: network.clone(),
        });
    }

    let mut news_broadcasts = Vec::new();
    let rows = sqlx::query!(
        "SELECT id, date, description, news_network_id, news_affiliate_id FROM news_broadcasts"
    )
    .fetch_all(&pool)
    .await?;
    for row in rows {
        if let Some(network_id) = row.news_network_id {
            let network = news_networks.iter().find(|n| n.id == network_id).unwrap();
            news_broadcasts.push(NewsBroadcast {
                id: row.id,
                date: row.date,
                description: row.description.clone(),
                news_network: Some(network.clone()),
                news_affiliate: None,
            });
        }
        if let Some(affiliate_id) = row.news_affiliate_id {
            let affiliate = news_affiliates
                .iter()
                .find(|a| a.id == affiliate_id)
                .unwrap();
            news_broadcasts.push(NewsBroadcast {
                id: row.id,
                date: row.date,
                description: row.description.clone(),
                news_network: None,
                news_affiliate: Some(affiliate.clone()),
            });
        }
    }

    Ok(news_broadcasts)
}

pub async fn get_people() -> Result<Vec<Person>> {
    let pool = establish_connection().await?;

    let people = sqlx::query_as!(
        Person,
        "SELECT id, name, historical_title, types as \"types: _\", description FROM people"
    )
    .fetch_all(&pool)
    .await?;

    Ok(people)
}

/// ***********************
/// Insert-based queries
/// ***********************
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
        sqlx::query!(
            r#"INSERT INTO master_videos (categories, title, date, description, links, nist_notes)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id"#,
            video.categories as _,
            video.title,
            video.date,
            video.description,
            &video.links,
            video.nist_notes,
        )
        .fetch_one(&mut *tx)
        .await?
        .id
    } else {
        sqlx::query!(
            r#"INSERT INTO master_videos (
                    id, categories, title, date, description, links, nist_notes)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (id) DO UPDATE SET
                   categories = EXCLUDED.categories,
                   title = EXCLUDED.title,
                   date = EXCLUDED.date,
                   description = EXCLUDED.description,
                   links = EXCLUDED.links,
                   nist_notes = EXCLUDED.nist_notes
               RETURNING id"#,
            video.id,
            video.categories as _,
            video.title,
            video.date,
            video.description,
            &video.links,
            video.nist_notes,
        )
        .fetch_one(&mut *tx)
        .await?
        .id
    };

    let mut updated_video = video.clone();
    updated_video.id = video_id;

    for timestamp in updated_video.timestamps.iter_mut() {
        let id = if timestamp.id == 0 {
            sqlx::query!(
                r#"
                INSERT INTO event_timestamps (
                    description, timestamp, event_type, time_of_day, master_video_id
                ) VALUES ($1, $2, $3, $4, $5)
                RETURNING id"#,
                timestamp.description,
                timestamp.timestamp,
                timestamp.event_type as _,
                timestamp.time_of_day,
                video_id
            )
            .fetch_one(&mut *tx)
            .await?
            .id
        } else {
            sqlx::query!(
                r#"
                INSERT INTO event_timestamps (
                    id, description, timestamp, event_type, time_of_day, master_video_id
                ) VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (id) DO UPDATE SET
                   description = EXCLUDED.description,
                   timestamp = EXCLUDED.timestamp,
                   event_type = EXCLUDED.event_type,
                   time_of_day = EXCLUDED.time_of_day,
                   master_video_id = EXCLUDED.master_video_id
                RETURNING id"#,
                timestamp.id,
                timestamp.description,
                timestamp.timestamp,
                timestamp.event_type as _,
                timestamp.time_of_day,
                video_id
            )
            .fetch_one(&mut *tx)
            .await?
            .id
        };
        timestamp.id = id;
    }

    for broadcast in video.news_broadcasts.iter() {
        sqlx::query!(
            r#"INSERT INTO master_videos_news_broadcasts (master_video_id, news_broadcast_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
            broadcast.id,
            video_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    for person in updated_video.people.iter_mut() {
        let row = sqlx::query!("SELECT id FROM people WHERE name = $1", person.name)
            .fetch_optional(&mut *tx)
            .await?;
        let id = if let Some(row) = row {
            row.id
        } else {
            sqlx::query!(
                r#"INSERT INTO people (name, types) VALUES ($1, $2) RETURNING id"#,
                person.name,
                person.types as _,
            )
            .fetch_one(&mut *tx)
            .await?
            .id
        };
        person.id = id;

        sqlx::query!(
            r#"INSERT INTO master_videos_people (master_video_id, person_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING"#,
            video_id,
            id
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
            r#"INSERT INTO master_videos_release_files (master_video_id, release_file_id)
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

pub async fn save_video(video: Video) -> Result<Video> {
    let pool = establish_connection().await?;

    let video_id = if video.id == 0 {
        sqlx::query!(
            r#"INSERT INTO videos (description, duration, is_primary, link, master_id, title)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id"#,
            video.description,
            video.duration,
            video.is_primary,
            video.link,
            video.master.id,
            video.title,
        )
        .fetch_one(&pool)
        .await?
        .id
    } else {
        sqlx::query!(
            r#"INSERT INTO videos (id, description, duration, is_primary, link, master_id, title)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (id) DO UPDATE SET
                   description = EXCLUDED.description,
                   duration = EXCLUDED.duration,
                   is_primary = EXCLUDED.is_primary,
                   link = EXCLUDED.link,
                   master_id = EXCLUDED.master_id,
                   title = EXCLUDED.title
               RETURNING id"#,
            video.id,
            video.description,
            video.duration,
            video.is_primary,
            video.link,
            video.master.id,
            video.title,
        )
        .fetch_one(&pool)
        .await?
        .id
    };

    let mut updated_video = video.clone();
    updated_video.id = video_id;
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
