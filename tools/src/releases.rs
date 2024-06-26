use crate::{helpers::human_readable_size, static_data::RELEASE_DATA};
use chrono::NaiveDate;
use color_eyre::{eyre::eyre, Result};
use csv::Writer;
use db::models::{MasterVideo, Release, ReleaseFile};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lava_torrent::torrent::v1::Torrent;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use url::Url;

pub async fn download_torrents(target_path: &Path) -> Result<()> {
    println!(
        "Saving torrents to directory at {}",
        target_path.to_string_lossy()
    );
    std::fs::create_dir_all(target_path)?;

    let multi_progress = MultiProgress::new();
    let total_pb = multi_progress.add(ProgressBar::new(RELEASE_DATA.len() as u64));
    total_pb.set_style(
        ProgressStyle::default_bar()
            .template("Overall progress: [{bar:40.cyan/blue}] {pos}/{len} files")?
            .progress_chars("#>-"),
    );
    let file_pb = multi_progress.add(ProgressBar::new(0));
    file_pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{prefix:.bold.dim} [{bar:30.green/blue}] {bytes}/{total_bytes} {bytes_per_sec}",
            )?
            .progress_chars("=> "),
    );

    for item in RELEASE_DATA.iter() {
        let torrent_url = item.1.to_string();

        if torrent_url.is_empty() {
            total_pb.inc(1);
            continue;
        }

        let torrent_url = Url::parse(&torrent_url)?;
        let file_name = get_file_name_from_url(&torrent_url)?;
        let torrent_path = target_path.join(file_name.clone());

        if !torrent_path.exists() {
            file_pb.set_prefix(format!("Downloading: {}", file_name));
            file_pb.set_position(0);
            download_file(&torrent_url, &torrent_path, &file_pb).await?;
            file_pb.finish_with_message("Download completed");
        }

        total_pb.inc(1);
    }
    Ok(())
}

pub async fn init_releases(torrents_path: &Path) -> Result<()> {
    for item in RELEASE_DATA.iter() {
        let date = item.0.to_string();
        let torrent_url = item.1.to_string();
        let name = item.2.to_string();

        let (torrent_url, torrent_path) = if !torrent_url.is_empty() {
            let url = Url::parse(&torrent_url)?;
            let file_name = get_file_name_from_url(&url)?;
            let torrent_path = torrents_path.join(file_name);
            (Some(url), Some(torrent_path))
        } else {
            (None, None)
        };

        let (directory, file_count, total_size, release_files) =
            if let Some(ref path) = torrent_path {
                match Torrent::read_from_file(path.clone()) {
                    Ok(torrent) => {
                        let files = torrent
                            .files
                            .ok_or_else(|| eyre!("Could not obtain torrent files"))?;
                        let first_file = &files[0];
                        // We want to store the directory below '911datasets.org'.
                        let directory: String = {
                            let mut ancestors = first_file.path.ancestors();
                            let mut second_to_last = None;
                            let mut last = ancestors.next();
                            for current in ancestors {
                                second_to_last = last;
                                last = Some(current);
                            }
                            second_to_last
                                .map(|p| p.to_path_buf())
                                .ok_or_else(|| eyre!("Could not obtain release directory"))?
                                .to_string_lossy()
                                .to_string()
                        };

                        let mut release_files = Vec::new();
                        let mut total_size = 0;
                        for file in files.iter() {
                            release_files.push(ReleaseFile {
                                id: 0, // The ID will be assigned upon save.
                                path: file.path.clone(),
                                size: file.length,
                            });
                            total_size += file.length;
                        }
                        (
                            Some(directory),
                            Some(files.len()),
                            Some(total_size as u64),
                            Some(release_files),
                        )
                    }
                    Err(_) => (None, None, None, None),
                }
            } else {
                (None, None, None, None)
            };

        println!("Saving release {name}...");
        let new_release = Release {
            id: 0,
            date: NaiveDate::parse_from_str(&date, "%Y-%m-%d")?,
            name: name.clone(),
            directory_name: directory,
            file_count: file_count.map(|f| f as i16),
            size: total_size.map(|s| s as i64),
            torrent_url: torrent_url.map(|u| u.to_string()),
            files: if let Some(files) = release_files {
                files
            } else {
                Vec::new()
            },
        };
        let saved_release = db::save_release(new_release).await?;
        if let Some(path) = torrent_path {
            db::save_torrent(saved_release.id, &path).await?;
        }
    }

    Ok(())
}

pub async fn get_torrent_tree(release_id: i32) -> Result<Option<Vec<(PathBuf, u64)>>> {
    let torrent_content = db::get_torrent_content(release_id).await?;
    if let Some(content) = torrent_content {
        let torrent = Torrent::read_from_bytes(content)?;
        let files = torrent
            .files
            .ok_or_else(|| eyre!("Failed to obtain torrent files"))?;
        let tree = files
            .iter()
            .map(|f| (f.path.clone(), f.length as u64))
            .collect::<Vec<(PathBuf, u64)>>();
        return Ok(Some(tree));
    }
    Ok(None)
}

pub async fn get_release_extensions(release_id: i32) -> Result<Option<Vec<(String, i32)>>> {
    let tree = get_torrent_tree(release_id).await?;
    if let Some(tree) = tree {
        let mut extension_counts = HashMap::new();
        for (path, _) in tree {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                let ext_lower = ext.to_lowercase();
                *extension_counts.entry(ext_lower).or_insert(0) += 1;
            }
        }

        let mut sorted_extensions: Vec<_> = extension_counts.into_iter().collect();
        sorted_extensions.sort_by(|a, b| a.0.cmp(&b.0));
        return Ok(Some(sorted_extensions));
    }
    Ok(None)
}

pub async fn list_release_range_extensions(
    start_release_id: i32,
    end_release_id: i32,
) -> Result<()> {
    let mut cumulative_extensions = HashMap::new();
    for id in start_release_id..=end_release_id {
        if let Some(extensions) = get_release_extensions(id).await? {
            for (ext, count) in extensions {
                *cumulative_extensions.entry(ext).or_insert(0) += count;
            }
        }
    }

    let mut sorted: Vec<_> = cumulative_extensions.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));
    for (extension, count) in sorted {
        println!("{}: {}", extension, count);
    }

    Ok(())
}

pub async fn list_release_extensions(release_id: i32) -> Result<()> {
    let extensions = get_release_extensions(release_id).await?;
    if let Some(extensions) = extensions {
        for (ext, count) in extensions {
            println!("{}: {}", ext, count);
        }
    } else {
        println!("Release {} does not have a torrent", release_id);
    }
    Ok(())
}

pub async fn export_video_list(
    start_release_id: i32,
    end_release_id: i32,
    out_path: &Path,
) -> Result<()> {
    let mut writer = Writer::from_writer(std::fs::File::create(out_path)?);
    writer.write_record(["master video id", "release name", "file path", "file size"])?;

    for release_id in start_release_id..=end_release_id {
        println!("Processing release {release_id}...");
        let release = db::get_release(release_id).await?;
        if let Some(torrent_tree) = get_torrent_tree(release_id).await? {
            for (file_path, file_size) in torrent_tree {
                if is_video_file(&file_path) {
                    writer.write_record([
                        "0",
                        &release.name,
                        file_path.to_str().unwrap_or(""),
                        &human_readable_size(file_size),
                    ])?;
                }
            }
        }
    }

    writer.flush()?;
    Ok(())
}

pub async fn export_master_videos(out_path: &Path) -> Result<()> {
    let mut writer = Writer::from_writer(std::fs::File::create(out_path)?);
    writer.write_record(["id", "title", "date", "description"])?;

    let master_videos = vec![MasterVideo::default()];
    for video in master_videos.iter() {
        writer.write_record([
            video.id.to_string(),
            video.title.clone(),
            video.date.map_or("".to_string(), |d| d.to_string()),
            video.description.clone(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

pub async fn download_file(url: &Url, target_path: &PathBuf, file_pb: &ProgressBar) -> Result<()> {
    let client = reqwest::Client::new();
    let mut request_builder = client.get(url.clone());
    let tmp_path = target_path.with_extension("part");

    let mut start = 0;
    if tmp_path.exists() {
        start = tokio::fs::metadata(&tmp_path).await?.len() as usize;
        file_pb.set_position(start as u64);
        request_builder = request_builder.header("Range", format!("bytes={}-", start));
    }

    let mut response = request_builder.send().await?;
    if response.status() == 404 {
        return Err(eyre!("File not found at {}", url.to_string()));
    }
    if !response.status().is_success() {
        return Err(eyre!(
            "Failed to download {}: {} response",
            url,
            response.status()
        ));
    }

    if let Some(len) = response.content_length() {
        file_pb.set_length(len);
    }
    let file = if start > 0 {
        OpenOptions::new().append(true).open(&tmp_path).await?
    } else {
        tokio::fs::File::create(&tmp_path).await?
    };

    let mut writer = BufWriter::new(file);
    while let Some(chunk) = response.chunk().await? {
        writer.write_all(&chunk).await?;
        file_pb.inc(chunk.len() as u64);
    }

    writer.flush().await?;
    tokio::fs::rename(&tmp_path, target_path).await?;

    Ok(())
}

fn get_file_name_from_url(url: &Url) -> Result<String> {
    let file_name = url
        .path_segments()
        .ok_or(eyre!("Failed to parse path segments"))?
        .last()
        .ok_or(eyre!("Failed to parse path segments"))?;
    Ok(file_name.to_string())
}

fn is_video_file(file_path: &Path) -> bool {
    let video_extensions = [
        "avi", "mp4", "mov", "wmv", "mpg", "mpe", "mpeg", "asf", "asx", "m1v", "vob",
    ];
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            video_extensions
                .iter()
                .any(|&ve| ve.eq_ignore_ascii_case(ext))
        })
        .unwrap_or(false)
}
