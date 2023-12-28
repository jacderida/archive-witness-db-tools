use crate::models::Release;
use crate::static_data::RELEASE_DATA;
use chrono::NaiveDate;
use color_eyre::{eyre::eyre, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lava_torrent::torrent::v1::Torrent;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use url::Url;

pub async fn download_torrents(target_path: &PathBuf) -> Result<()> {
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

pub async fn init_releases(torrents_path: PathBuf) -> Result<()> {
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

        let (directory, file_count, size) = if let Some(path) = torrent_path {
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
                        while let Some(current) = ancestors.next() {
                            second_to_last = last;
                            last = Some(current);
                        }
                        second_to_last
                            .map(|p| p.to_path_buf())
                            .ok_or_else(|| eyre!("Could not obtain release directory"))?
                            .to_string_lossy()
                            .to_string()
                    };
                    let mut size = 0;
                    for file in files.iter() {
                        size += file.length;
                    }
                    (Some(directory), Some(files.len()), Some(size as u64))
                }
                Err(_) => (None, None, None),
            }
        } else {
            (None, None, None)
        };

        println!("Saving release {name}...");
        let new_release = Release {
            id: 0,
            date: NaiveDate::parse_from_str(&date, "%Y-%m-%d")?,
            name: name.clone(),
            directory_name: directory,
            file_count: file_count.map(|f| f as i16),
            size: size.map(|s| s as i64),
            torrent_url: torrent_url.map(|u| u.to_string()),
        };
        crate::db::save_release(new_release).await?;
    }

    Ok(())
}

pub fn get_torrent_tree(torrent_path: &PathBuf) -> Result<Vec<(PathBuf, u64)>> {
    let torrent = Torrent::read_from_file(torrent_path.clone())?;
    let files = torrent
        .files
        .ok_or_else(|| eyre!("Failed to obtain torrent files"))?;
    let tree = files
        .iter()
        .map(|f| (f.path.clone(), f.length as u64))
        .collect::<Vec<(PathBuf, u64)>>();
    Ok(tree)
}

pub async fn list_releases() -> Result<()> {
    let releases = crate::db::get_releases().await?;
    for release in releases.iter() {
        println!("{}: {}", release.id, release.name);
    }
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
            "Failed to download file: {} response",
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
