pub mod cumulus;
pub mod error;
pub mod helpers;
pub mod models;
mod static_data;

use crate::error::{Error, Result};
use crate::models::{
    Category, EventTimestamp, EventType, MasterVideo, NewsAffiliate, NewsBroadcast, NewsNetwork,
    NistTape, NistVideo, Person, PersonType, Release, ReleaseFile, Video,
};
use csv::ReaderBuilder;
use dotenvy::dotenv;
use sqlx::pool::Pool;
use sqlx::postgres::PgPoolOptions;
use sqlx::Postgres;
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

pub async fn establish_connection() -> Result<Pool<Postgres>> {
    dotenv().ok();
    let database_url = std::env::var("AW_DB_URL")?;
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

pub async fn get_videos_for_master(master_id: i32) -> Result<Vec<Video>> {
    let pool = establish_connection().await?;
    let rows = sqlx::query!(
        r#"
            SELECT id, title, channel_username, description, duration, link, is_primary, master_id
            FROM videos
            WHERE master_id = $1
            ORDER BY id
        "#,
        master_id,
    )
    .fetch_all(&pool)
    .await?;

    let master = get_master_video(master_id, Some(pool.clone())).await?;
    let mut videos = Vec::new();
    for row in rows {
        videos.push(Video {
            channel_username: row.channel_username,
            description: row.description,
            duration: row.duration,
            id: row.id,
            is_primary: row.is_primary,
            link: row.link,
            master: master.clone(),
            title: row.title,
        });
    }

    Ok(videos)
}

pub async fn get_videos() -> Result<Vec<Video>> {
    let pool = establish_connection().await?;

    let rows = sqlx::query!(
        r#"
            SELECT id, channel_username, description, duration, is_primary, link, master_id, title
            FROM videos
            ORDER BY id
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let mut videos = Vec::new();
    for row in rows {
        let master = get_master_video(row.master_id, Some(pool.clone())).await?;
        videos.push(Video {
            channel_username: row.channel_username,
            description: row.description,
            duration: row.duration,
            id: row.id,
            is_primary: row.is_primary,
            link: row.link,
            master: master.clone(),
            title: row.title,
        })
    }

    Ok(videos)
}
pub async fn get_video(id: i32, pool: Option<Pool<Postgres>>) -> Result<Video> {
    let pool = if let Some(p) = pool {
        p
    } else {
        establish_connection().await?
    };

    let row = sqlx::query!(
        r#"
            SELECT id, channel_username, description, duration, is_primary, link, master_id, title
            FROM videos
            WHERE id = $1
        "#,
        id
    )
    .fetch_one(&pool)
    .await?;

    let master = get_master_video(row.id, Some(pool.clone())).await?;
    let video = Video {
        channel_username: row.channel_username,
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
        r#"
            SELECT
                video_id, video_title, network,
                broadcast_date, duration_min, subject,
                notes, is_missing, additional_notes
            FROM nist_videos
        "#
    )
    .fetch_all(&pool)
    .await?;
    Ok(videos)
}

pub async fn get_nist_tapes() -> Result<Vec<NistTape>> {
    let videos = get_nist_videos().await?;

    let pool = establish_connection().await?;
    let rows = sqlx::query!(
        r#"
            SELECT
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
                timecode,
                document_database_number
            FROM nist_tapes
            ORDER BY tape_id
        "#
    )
    .fetch_all(&pool)
    .await?;

    let mut tapes = Vec::new();
    for row in rows {
        let video = videos
            .iter()
            .find(|v| v.video_id == row.video_id)
            .ok_or_else(|| Error::NistVideoNotFound(row.video_id))?;
        tapes.push(NistTape {
            tape_id: row.tape_id,
            tape_name: row.tape_name,
            tape_source: row.tape_source,
            copy: row.copy,
            derived_from: row.derived_from,
            format: row.format,
            duration_min: row.duration_min,
            batch: row.batch,
            clips: row.clips,
            timecode: row.timecode,
            release_files: Vec::new(),
            video: video.clone(),
            document_database_number: row.document_database_number,
        })
    }

    for tape in tapes.iter_mut() {
        let rows = sqlx::query!(
            r#"
                SELECT rf.path, rf.size
                FROM release_files rf
                JOIN nist_tapes_release_files ntrf ON rf.id = ntrf.release_file_id
                WHERE ntrf.nist_tape_id = $1;
            "#,
            tape.tape_id
        )
        .fetch_all(&pool)
        .await?;
        for row in rows {
            tape.release_files
                .push((PathBuf::from(row.path), row.size as u64));
        }
    }

    Ok(tapes)
}

pub async fn get_nist_tapes_grouped_by_video() -> Result<BTreeMap<NistVideo, Vec<NistTape>>> {
    let videos = get_nist_videos().await?;
    let pool = establish_connection().await?;

    let rows = sqlx::query!(
        r#"
            SELECT
                video_id,
                array_agg(tape_id ORDER BY tape_id) AS tape_ids,
                array_agg(tape_name ORDER BY tape_id) AS tape_names,
                array_agg(tape_source ORDER BY tape_id) AS tape_sources,
                array_agg(copy ORDER BY tape_id) AS copies,
                array_agg(derived_from ORDER BY tape_id) AS derived_froms,
                array_agg(format ORDER BY tape_id) AS formats,
                array_agg(duration_min ORDER BY tape_id) AS durations,
                array_agg(batch ORDER BY tape_id) AS batches,
                array_agg(clips ORDER BY tape_id) AS clips,
                array_agg(timecode ORDER BY tape_id) AS timecodes,
                array_agg(document_database_number ORDER BY tape_id) AS "document_database_numbers: Vec<Option<String>>"
            FROM nist_tapes
            GROUP BY video_id
        "#
    )
    .fetch_all(&pool)
    .await?;

    let mut grouped_tapes: BTreeMap<NistVideo, Vec<NistTape>> = BTreeMap::new();

    for row in rows {
        let video = videos
            .iter()
            .find(|v| v.video_id == row.video_id)
            .ok_or_else(|| Error::NistVideoNotFound(row.video_id))?
            .clone();

        let tape_ids = row.tape_ids.unwrap_or_default();
        let tape_names = row.tape_names.unwrap_or_default();
        let tape_sources = row.tape_sources.unwrap_or_default();
        let copies = row.copies.unwrap_or_default();
        let derived_froms = row.derived_froms.unwrap_or_default();
        let formats = row.formats.unwrap_or_default();
        let durations = row.durations.unwrap_or_default();
        let batches = row.batches.unwrap_or_default();
        let clips = row.clips.unwrap_or_default();
        let timecodes = row.timecodes.unwrap_or_default();
        let document_database_numbers = row.document_database_numbers.unwrap_or_default();

        let mut tapes = Vec::new();
        for i in 0..tape_ids.len() {
            tapes.push(NistTape {
                tape_id: tape_ids[i],
                tape_name: tape_names[i].clone(),
                tape_source: tape_sources[i].clone(),
                copy: copies[i],
                derived_from: derived_froms[i],
                format: formats[i].clone(),
                duration_min: durations[i],
                batch: batches[i],
                clips: clips[i],
                timecode: timecodes[i],
                document_database_number: document_database_numbers[i].clone(),
                release_files: Vec::new(),
                video: video.clone(),
            });
        }

        grouped_tapes.insert(video, tapes);
    }

    for (_, tapes) in grouped_tapes.iter_mut() {
        for tape in tapes.iter_mut() {
            let rows = sqlx::query!(
                r#"
                    SELECT rf.path, rf.size
                    FROM release_files rf
                    JOIN nist_tapes_release_files ntrf ON rf.id = ntrf.release_file_id
                    WHERE ntrf.nist_tape_id = $1;
                "#,
                tape.tape_id
            )
            .fetch_all(&pool)
            .await?;

            for row in rows {
                tape.release_files
                    .push((PathBuf::from(row.path), row.size as u64));
            }
        }
    }

    Ok(grouped_tapes)
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

pub async fn get_news_network(id: i32, pool: Option<Pool<Postgres>>) -> Result<NewsNetwork> {
    let pool = if let Some(p) = pool {
        p
    } else {
        establish_connection().await?
    };
    let news_network = sqlx::query_as!(
        NewsNetwork,
        "SELECT id, name, description FROM news_networks WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await?;
    Ok(news_network)
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

pub async fn get_news_affiliate(id: i32, pool: Option<Pool<Postgres>>) -> Result<NewsAffiliate> {
    let pool = if let Some(p) = pool {
        p
    } else {
        establish_connection().await?
    };

    let news_networks = get_news_networks(Some(pool.clone())).await?;
    let row = sqlx::query!(
        r#"
            SELECT id, name, description, region, news_network_id FROM news_affiliates
            WHERE id = $1
        "#,
        id
    )
    .fetch_one(&pool)
    .await?;
    let network = news_networks
        .iter()
        .find(|n| n.id == row.news_network_id)
        .unwrap();

    Ok(NewsAffiliate {
        id: row.id,
        name: row.name,
        description: row.description,
        region: row.region,
        network: network.clone(),
    })
}

pub async fn get_news_broadcast(id: i32, pool: Option<Pool<Postgres>>) -> Result<NewsBroadcast> {
    let pool = if let Some(p) = pool {
        p
    } else {
        establish_connection().await?
    };

    let row = sqlx::query!(
        r#"
            SELECT id, date, description, news_network_id, news_affiliate_id FROM news_broadcasts
            WHERE id = $1
        "#,
        id
    )
    .fetch_one(&pool)
    .await?;

    let network = if let Some(network_id) = row.news_network_id {
        Some(get_news_network(network_id, Some(pool.clone())).await?)
    } else {
        None
    };

    let affiliate = if let Some(affiliate_id) = row.news_affiliate_id {
        Some(get_news_affiliate(affiliate_id, Some(pool.clone())).await?)
    } else {
        None
    };

    Ok(NewsBroadcast {
        date: row.date,
        description: row.description,
        id: row.id,
        news_network: network,
        news_affiliate: affiliate,
    })
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
        r#"
            SELECT id, date, description, news_network_id, news_affiliate_id FROM news_broadcasts
            ORDER BY id
        "#
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
pub async fn import_nist_videos_table_from_csv(csv_path: &Path) -> Result<()> {
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

pub async fn import_nist_tapes_table_from_csv(csv_path: &Path) -> Result<()> {
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
            tape.video.video_id,
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

pub async fn import_document_database_numbers() -> Result<()> {
    let pool = establish_connection().await?;
    let mut tx = pool.begin().await?;

    for (id, number) in crate::static_data::DOCUMENT_DATABASE_NUMBERS.iter() {
        println!("Assigning {} to tape {}", number, id);
        sqlx::query!(
            "UPDATE nist_tapes SET document_database_number = $1 WHERE tape_id = $2",
            number,
            id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn save_news_network(network: NewsNetwork) -> Result<NewsNetwork> {
    let pool = establish_connection().await?;

    let network_id = if network.id == 0 {
        sqlx::query!(
            r#"INSERT INTO news_networks (name, description) VALUES ($1, $2) RETURNING id"#,
            network.name,
            network.description,
        )
        .fetch_one(&pool)
        .await?
        .id
    } else {
        sqlx::query!(
            r#"INSERT INTO news_networks (id, name, description)
               VALUES ($1, $2, $3)
               ON CONFLICT (id) DO UPDATE SET
                   name = EXCLUDED.name,
                   description = EXCLUDED.description
               RETURNING id"#,
            network.id,
            network.name,
            network.description,
        )
        .fetch_one(&pool)
        .await?
        .id
    };

    let mut updated_network = network.clone();
    updated_network.id = network_id;
    Ok(updated_network)
}

pub async fn save_news_affiliate(affiliate: NewsAffiliate) -> Result<NewsAffiliate> {
    let pool = establish_connection().await?;

    let affiliate_id = if affiliate.id == 0 {
        sqlx::query!(
            r#"
                INSERT INTO news_affiliates (name, description, region, news_network_id)
                VALUES ($1, $2, $3, $4)
                RETURNING id
            "#,
            affiliate.name,
            affiliate.description,
            affiliate.region,
            affiliate.network.id,
        )
        .fetch_one(&pool)
        .await?
        .id
    } else {
        sqlx::query!(
            r#"INSERT INTO news_affiliates (id, name, description, region, news_network_id)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (id) DO UPDATE SET
                   name = EXCLUDED.name,
                   description = EXCLUDED.description,
                   region = EXCLUDED.region,
                   news_network_id = EXCLUDED.news_network_id
               RETURNING id"#,
            affiliate.id,
            affiliate.name,
            affiliate.description,
            affiliate.region,
            affiliate.network.id,
        )
        .fetch_one(&pool)
        .await?
        .id
    };

    let mut updated_affiliate = affiliate.clone();
    updated_affiliate.id = affiliate_id;
    Ok(updated_affiliate)
}

pub async fn save_news_broadcast(broadcast: NewsBroadcast) -> Result<NewsBroadcast> {
    if broadcast.news_network.is_some() && broadcast.news_affiliate.is_some() {
        return Err(Error::NewsBroadcastCannotHaveNetworkAndAffiliate);
    }

    let pool = establish_connection().await?;
    let broadcast_id = if let Some(network) = &broadcast.news_network {
        if broadcast.id == 0 {
            sqlx::query!(
                r#"
                    INSERT INTO news_broadcasts (date, description, news_network_id)
                    VALUES ($1, $2, $3)
                    RETURNING id
                "#,
                broadcast.date,
                broadcast.description,
                network.id
            )
            .fetch_one(&pool)
            .await?
            .id
        } else {
            sqlx::query!(
                r#"
                    INSERT INTO news_broadcasts (id, date, description, news_network_id)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (id) DO UPDATE SET
                       date = EXCLUDED.date,
                       description = EXCLUDED.description,
                       news_network_id = EXCLUDED.news_network_id
                    RETURNING id
               "#,
                broadcast.id,
                broadcast.date,
                broadcast.description,
                network.id
            )
            .fetch_one(&pool)
            .await?
            .id
        }
    } else if let Some(affiliate) = &broadcast.news_affiliate {
        if broadcast.id == 0 {
            sqlx::query!(
                r#"
                    INSERT INTO news_broadcasts (date, description, news_affiliate_id)
                    VALUES ($1, $2, $3)
                    RETURNING id
                "#,
                broadcast.date,
                broadcast.description,
                affiliate.id
            )
            .fetch_one(&pool)
            .await?
            .id
        } else {
            sqlx::query!(
                r#"
                    INSERT INTO news_broadcasts (id, date, description, news_affiliate_id)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (id) DO UPDATE SET
                       date = EXCLUDED.date,
                       description = EXCLUDED.description,
                       news_affiliate_id = EXCLUDED.news_affiliate_id
                    RETURNING id
               "#,
                broadcast.id,
                broadcast.date,
                broadcast.description,
                affiliate.id
            )
            .fetch_one(&pool)
            .await?
            .id
        }
    } else {
        return Err(Error::NewsBroadcastDoesNotHaveNetworkOrAffiliate);
    };

    let mut updated_broadcast = broadcast.clone();
    updated_broadcast.id = broadcast_id;
    Ok(updated_broadcast)
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
            r#"
                INSERT INTO videos (
                    channel_username, description, duration, is_primary, link, master_id, title)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id
            "#,
            video.channel_username,
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
            r#"
                INSERT INTO videos (
                    id, channel_username, description, duration, is_primary, link, master_id, title)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (id) DO UPDATE SET
                    description = EXCLUDED.description,
                    duration = EXCLUDED.duration,
                    is_primary = EXCLUDED.is_primary,
                    link = EXCLUDED.link,
                    master_id = EXCLUDED.master_id,
                    title = EXCLUDED.title
                RETURNING id
           "#,
            video.id,
            video.channel_username,
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

pub async fn save_nist_tape_files(tape_id: i32, files: Vec<(PathBuf, u64)>) -> Result<NistTape> {
    let pool = establish_connection().await?;
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "DELETE FROM nist_tapes_release_files WHERE nist_tape_id = $1",
        tape_id
    )
    .execute(&mut *tx)
    .await?;

    for file in files.iter() {
        let path = file.0.clone();
        let row = sqlx::query!(
            "SELECT id, path, size FROM release_files WHERE path = $1",
            &path.to_string_lossy()
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
                INSERT INTO nist_tapes_release_files (nist_tape_id, release_file_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
            "#,
            tape_id,
            row.id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let updated_tape = get_nist_tapes().await?;
    updated_tape
        .into_iter()
        .find(|t| t.tape_id == tape_id)
        .ok_or_else(|| Error::NistTapeNotFound(tape_id))
}

pub async fn save_nist_video(
    id: i32,
    is_missing: bool,
    additional_notes: &str,
) -> Result<NistVideo> {
    let pool = establish_connection().await?;
    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
            UPDATE nist_videos SET is_missing = $1, additional_notes = $2
            WHERE video_id = $3
        "#,
        is_missing,
        additional_notes,
        id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let updated_tape = get_nist_videos().await?;
    updated_tape
        .into_iter()
        .find(|v| v.video_id == id)
        .ok_or_else(|| Error::NistVideoNotFound(id))
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
