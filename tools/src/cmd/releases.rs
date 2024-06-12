use crate::{helpers::print_banner, static_data::VideoReleaseType};
use color_eyre::{eyre::eyre, Result};
use colored::Colorize;
use db::models::{NistTape, Release};
use std::{
    collections::HashSet,
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub async fn download_torrents(path: &Path) -> Result<()> {
    crate::releases::download_torrents(path).await?;
    Ok(())
}

pub async fn find(term: &str) -> Result<()> {
    let results = db::find_release_files(term).await?;
    for (release_name, files) in results {
        println!("{release_name}:");
        for file in files {
            println!("{}", file.to_string_lossy());
        }
        println!();
    }
    Ok(())
}

pub async fn init(path: &Path) -> Result<()> {
    crate::releases::init_releases(path).await?;
    Ok(())
}

pub async fn ls() -> Result<()> {
    let releases = db::get_releases().await?;
    for release in releases.iter() {
        println!("{}: {}", release.id, release.name);
    }
    Ok(())
}

pub async fn files_ls(id: u32) -> Result<()> {
    let release = db::get_release(id as i32).await?;
    for file in release.files.iter() {
        println!("{}", file.path.to_string_lossy());
    }
    Ok(())
}

pub async fn files_ls_extensions(
    release_id: Option<u32>,
    start_release_id: Option<u32>,
    end_release_id: Option<u32>,
    sum: bool,
) -> Result<()> {
    if let (Some(start), Some(end)) = (start_release_id, end_release_id) {
        if sum {
            crate::releases::list_release_range_extensions(start as i32, end as i32).await?;
        } else {
            let releases = db::get_releases().await?;
            for id in start..=end {
                let name = releases
                    .iter()
                    .find(|r| r.id == id as i32)
                    .map(|r| r.name.clone())
                    .ok_or_else(|| eyre!("Could not find release with ID {id}"))?;
                print_banner(&format!("{}: {}", id, &name));
                crate::releases::list_release_extensions(id as i32).await?;
            }
        }
    } else if let Some(release_id) = release_id {
        crate::releases::list_release_extensions(release_id as i32).await?;
    } else {
        let releases = db::get_releases().await?;
        for release in releases.iter() {
            print_banner(&format!("{}: {}", release.id, &release.name));
            crate::releases::list_release_extensions(release.id).await?;
        }
    }
    Ok(())
}

#[derive(Default)]
struct ReportSummary {
    total: usize,
    allocated: usize,
}

impl ReportSummary {
    fn print(&self) {
        println!();
        print_banner("Summary");
        println!("Total release files/dirs: {}", self.total);
        println!("Allocated: {}", self.allocated);
        println!("Unallocated: {}", self.total - self.allocated);
    }
}

pub async fn report_nist_videos_allocated() -> Result<()> {
    let tapes = db::get_nist_tapes().await?; // Get the release list *without* any files.
    let releases = db::get_releases().await?;
    let mut summary = ReportSummary::default();

    for (name, release_type) in crate::static_data::VIDEO_RELEASES.iter() {
        let id = releases
            .iter()
            .find(|r| r.name == *name)
            .map(|r| r.id)
            .ok_or_else(|| eyre!("Could not find release with name {name}"))?;
        let release = db::get_release(id).await?; // Get the full release *with* files.

        print_banner(&format!("{}: {}", release.id, &release.name));
        match release_type {
            VideoReleaseType::Dvd => {
                print_dvds(
                    &release,
                    &tapes,
                    &[],
                    &mut summary.allocated,
                    &mut summary.total,
                );
            }
            VideoReleaseType::Files => {
                for file in release.files.iter().map(|f| &f.path) {
                    if !is_video_file(file) {
                        continue;
                    }

                    summary.total += 1;
                    // Find any tapes whose files contain `file`.
                    let found_tapes = tapes
                        .iter()
                        .filter(|t| {
                            t.release_files.iter().any(|f| {
                                let path = &f.0;
                                file == path
                            })
                        })
                        .collect::<Vec<&NistTape>>();
                    let file_name = file
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .ok_or_else(|| eyre!("Could not obtain file name"))?;
                    if found_tapes.is_empty() {
                        println!("{}: video not yet allocated", file_name);
                    } else {
                        summary.allocated += 1;
                        // Each tape will be related to the same video.
                        let msg = format!(
                            "{}: {} [{}]",
                            file_name,
                            found_tapes[0].video.video_title,
                            found_tapes[0].video.video_id
                        );
                        println!("{}", msg.green());
                    }
                }
            }
            VideoReleaseType::DvdAndMisc(exclusions) => {
                print_dvds(
                    &release,
                    &tapes,
                    exclusions,
                    &mut summary.allocated,
                    &mut summary.total,
                );
            }
        }
    }

    summary.print();

    Ok(())
}

fn print_dvds(
    release: &Release,
    tapes: &[NistTape],
    dirs_to_exclude: &[PathBuf],
    total_found: &mut usize,
    total_processed: &mut usize,
) {
    let mut dvd_directories = release
        .files
        .iter()
        .filter_map(|file| {
            file.path
                .iter()
                .nth(3)
                .map(|component| component.to_string_lossy().into_owned())
        })
        .collect::<HashSet<String>>();
    let mut dvd_directories: Vec<String> = dvd_directories.drain().collect();
    dvd_directories.sort();

    for dir in dvd_directories.iter() {
        if dirs_to_exclude
            .iter()
            .any(|d| d.to_string_lossy().to_string().contains(dir))
        {
            continue;
        }

        *total_processed += 1;
        // Find tapes whose files contain `dir` in their path.
        let found_tapes = tapes
            .iter()
            .flat_map(|t| {
                t.release_files.iter().filter_map(move |f| {
                    let path = &f.0;
                    let path = path.to_string_lossy().to_string();
                    if path.contains(dir) {
                        Some(t)
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<&NistTape>>();
        if found_tapes.is_empty() {
            println!("{}: video not yet allocated", dir);
        } else {
            *total_found += 1;
            // Each tape will be related to the same video.
            let msg = format!(
                "{}: {} [{}]",
                dir, found_tapes[0].video.video_title, found_tapes[0].video.video_id
            );
            println!("{}", msg.green());
        }
    }
}

fn is_video_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(OsStr::to_str) {
        let lower_ext = extension.to_lowercase();
        return matches!(
            lower_ext.as_str(),
            "mp4" | "mkv" | "mov" | "avi" | "wmv" | "flv" | "webm" | "mpeg" | "mpg" | "m4v"
        );
    }
    false
}
