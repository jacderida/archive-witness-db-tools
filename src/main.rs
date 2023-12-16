use archive_witness_db_builder::releases::{download_torrents, import_releases};
use clap::{Parser, Subcommand};
use color_eyre::Result;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "db-builder", version = env!("CARGO_PKG_VERSION"))]
struct Opt {
    #[command(subcommand)]
    command: Option<Commands>,
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
    /// Import the 911datasets.org releases.
    ///
    /// The binary contains some static data to initialise the releases, but we also use the
    /// torrent files for some additional information.
    #[clap(name = "import-releases")]
    ImportReleases {
        /// Path to the torrent directory
        #[arg(long)]
        torrent_path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::parse();
    match opt.command {
        Some(Commands::DownloadTorrents { path }) => {
            download_torrents(&path).await?;
            Ok(())
        }
        Some(Commands::ImportReleases { torrent_path }) => {
            import_releases(torrent_path)?;
            Ok(())
        }
        None => Ok(()),
    }
}
