pub mod cumulus;
pub mod db;
pub mod error;
pub mod images;
pub mod models;
pub mod releases;
pub mod static_data;

use crate::cumulus::read_cumulus_photo_export;
use crate::images::*;
use crate::releases::*;
use clap::{Parser, Subcommand};
use color_eyre::Result;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "db-builder", version = env!("CARGO_PKG_VERSION"))]
struct Opt {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Download all the 911datasets.org torrent files.
    ///
    /// The URLs are encoded in the binary.
    #[clap(name = "download-torrents")]
    DownloadTorrents {
        /// The output directory for the torrents
        #[arg(long)]
        path: PathBuf,
    },
    #[clap(subcommand)]
    Images(ImagesSubcommands),
    #[clap(subcommand)]
    Releases(ReleasesSubcommands),
}

/// Manage images
#[derive(Subcommand, Debug)]
enum ImagesSubcommands {
    /// Import image content from a release.
    #[clap(name = "import")]
    Import {
        /// Path to the Cumulus data dump file
        #[arg(long)]
        cumulus_export_path: PathBuf,
        /// The ID of the release
        #[arg(long)]
        release_id: u16,
        /// Path to the base 911datasets.org directory
        #[arg(long)]
        releases_base_path: PathBuf,
        /// Path to the torrent file corresponding to the release
        #[arg(long)]
        torrent_path: PathBuf,
    },
}

/// Manage releases
#[derive(Subcommand, Debug)]
enum ReleasesSubcommands {
    /// Initialise the 911datasets.org releases.
    #[clap(name = "init")]
    Init {
        /// Path to the torrent directory
        #[arg(long)]
        torrent_path: PathBuf,
    },
    /// List all releases
    #[clap(name = "ls")]
    Ls {},
    /// List all the file extensions in the release
    #[clap(name = "ls-extensions")]
    LsExtensions {
        /// Path to the torrent file for the release
        #[arg(long)]
        torrent_path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::parse();
    match opt.command {
        Commands::DownloadTorrents { path } => {
            download_torrents(&path).await?;
            Ok(())
        }
        Commands::Images(images_command) => match images_command {
            ImagesSubcommands::Import {
                cumulus_export_path,
                release_id,
                releases_base_path,
                torrent_path,
            } => {
                let images = read_cumulus_photo_export(cumulus_export_path)?;
                println!("Retrieved {} images from the Cumulus export", images.len());
                import_images(
                    release_id as i32,
                    images,
                    &releases_base_path,
                    &torrent_path,
                )
                .await?;
                Ok(())
            }
        },
        Commands::Releases(releases_command) => match releases_command {
            ReleasesSubcommands::Init { torrent_path } => {
                init_releases(torrent_path).await?;
                Ok(())
            }
            ReleasesSubcommands::Ls {} => {
                list_releases().await?;
                Ok(())
            }
            ReleasesSubcommands::LsExtensions { torrent_path } => {
                let tree = get_torrent_tree(&torrent_path)?;
                let mut extension_counts = HashMap::new();
                for (path, _) in tree {
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        *extension_counts.entry(ext_lower).or_insert(0) += 1;
                    }
                }
                for (ext, count) in extension_counts {
                    println!("{}: {}", ext, count);
                }
                Ok(())
            }
        },
    }
}
