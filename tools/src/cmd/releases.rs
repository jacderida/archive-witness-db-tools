use color_eyre::Result;
use std::path::Path;

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

pub async fn ls_extensions(
    release_id: Option<u32>,
    start_release_id: Option<u32>,
    end_release_id: Option<u32>,
) -> Result<()> {
    if let (Some(start), Some(end)) = (start_release_id, end_release_id) {
        crate::releases::list_release_range_extensions(start as i32, end as i32).await?;
    } else if let Some(release_id) = release_id {
        crate::releases::list_release_extensions(release_id as i32).await?;
    } else {
        let releases = db::get_releases().await?;
        for release in releases.iter() {
            let banner = "=".repeat(release.name.len());
            println!("{}", banner);
            println!("{}", release.name);
            println!("{}", banner);
            crate::releases::list_release_extensions(release.id).await?;
            println!();
        }
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
