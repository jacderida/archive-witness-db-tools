use archive_witness_db_builder::cumulus::read_cumulus_photo_export;
use archive_witness_db_builder::releases::*;
use clap::{Parser, Subcommand};
use color_eyre::Result;
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

/// Manage releases
#[derive(Subcommand, Debug)]
enum ReleasesSubcommands {
    /// Import the 911datasets.org releases.
    #[clap(name = "import")]
    Import {
        /// Path to the torrent directory
        #[arg(long)]
        torrent_path: PathBuf,
    },
    /// List all releases
    #[clap(name = "ls")]
    Ls {},
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
        /// Path to the torrent file corresponding to the release
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
        Commands::Releases(releases_command) => match releases_command {
            ReleasesSubcommands::Import { torrent_path } => {
                import_releases(torrent_path).await?;
                Ok(())
            }
            ReleasesSubcommands::Ls {} => {
                list_releases().await?;
                Ok(())
            }
        },
        Commands::Images(images_command) => match images_command {
            ImagesSubcommands::Import {
                cumulus_export_path,
                release_id,
                torrent_path,
            } => {
                let images = read_cumulus_photo_export(cumulus_export_path)?;
                println!("Retrieved {} images from the Cumulus export", images.len());
                Ok(())
            }
        },
    }
}
