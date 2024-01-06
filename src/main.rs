pub mod cumulus;
pub mod db;
pub mod error;
pub mod images;
pub mod models;
pub mod releases;
pub mod static_data;

use crate::cumulus::*;
use crate::db::*;
use crate::images::*;
use crate::models::MasterVideo;
use crate::releases::*;
use clap::{Parser, Subcommand};
use color_eyre::{eyre::eyre, Result};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "db-builder", version = env!("CARGO_PKG_VERSION"))]
struct Opt {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(subcommand)]
    AccessDb(AccessDbSubcommands),
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
    Cumulus(CumulusSubcommands),
    #[clap(subcommand)]
    Images(ImagesSubcommands),
    #[clap(subcommand)]
    Releases(ReleasesSubcommands),
    #[clap(subcommand)]
    Videos(VideosSubcommands),
}

/// Tools for importing NIST's Access database
///
/// External tools must be used to export the database tables to CSV.
#[derive(Subcommand, Debug)]
enum AccessDbSubcommands {
    /// Import a CSV export of the NIST Tapes table from their Access database
    #[clap(name = "import-tapes")]
    ImportTapes {
        /// Path to the CSV export
        #[arg(long)]
        path: PathBuf,
    },
    /// Import a CSV export of the NIST Videos table from their Access database
    #[clap(name = "import-videos")]
    ImportVideos {
        /// Path to the CSV export
        #[arg(long)]
        path: PathBuf,
    },
}

/// Tools for working with the Cumulus exports
#[derive(Subcommand, Debug)]
enum CumulusSubcommands {
    /// Display the difference between two sets of fields
    #[clap(name = "diff-fields")]
    DiffFields {
        /// Path to the first Cumulus data dump file
        #[arg(long)]
        first_cumulus_export_path: PathBuf,
        /// Path to the second Cumulus data dump file
        #[arg(long)]
        second_cumulus_export_path: PathBuf,
    },
    /// Retrieve an asset with a given name
    #[clap(name = "get")]
    Get {
        /// Path to the Cumulus data dump file
        #[arg(long)]
        cumulus_export_path: PathBuf,
        /// Name of the asset to retrieve
        #[arg(long)]
        name: String,
    },
    /// List the fields in a Cumulus export
    #[clap(name = "ls-fields")]
    LsFields {
        /// Path to the Cumulus data dump file
        #[arg(long)]
        cumulus_export_path: PathBuf,
    },
}

/// Manage images
#[derive(Subcommand, Debug)]
enum ImagesSubcommands {
    /// Convert the Cumulus photo export to a CSV
    #[clap(name = "convert")]
    Convert {
        /// Path to the Cumulus data dump file
        #[arg(long)]
        cumulus_export_path: PathBuf,
        /// Path to the output CSV file
        #[arg(long)]
        out_path: PathBuf,
    },
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
    },
}

/// Manage 911datasets.org releases
#[derive(Subcommand, Debug)]
enum ReleasesSubcommands {
    /// Initialise the 911datasets.org releases
    #[clap(name = "init")]
    Init {
        /// Path to the torrent directory
        #[arg(long)]
        torrent_path: PathBuf,
    },
    /// List all releases
    #[clap(name = "ls")]
    Ls {},
    /// List all the file extensions in a release
    #[clap(name = "ls-extensions")]
    LsExtensions {
        /// The ID of the release.
        ///
        /// If no ID is supplied, extensions for all releases will be listed.
        #[arg(long)]
        release_id: Option<u32>,
    },
}

/// Manage videos
#[derive(Subcommand, Debug)]
enum VideosSubcommands {
    /// Build the master video list from the NIST video list
    #[clap(name = "build-master")]
    BuildMaster {},
    /// Convert the Cumulus video export to a CSV
    #[clap(name = "convert")]
    Convert {
        /// Path to the Cumulus data dump file
        #[arg(long)]
        cumulus_export_path: PathBuf,
        /// Path to the output CSV file
        #[arg(long)]
        out_path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let opt = Opt::parse();
    match opt.command {
        Commands::AccessDb(access_command) => match access_command {
            AccessDbSubcommands::ImportTapes { path } => {
                print!("Importing the Tapes table from the NIST database...");
                import_nist_tapes_table_from_csv(path).await?;
                print!("done");
                Ok(())
            }
            AccessDbSubcommands::ImportVideos { path } => {
                print!("Importing the Videos table from the NIST database...");
                import_nist_video_table_from_csv(path).await?;
                print!("done");
                Ok(())
            }
        },
        Commands::DownloadTorrents { path } => {
            download_torrents(&path).await?;
            Ok(())
        }
        Commands::Cumulus(cumulus_command) => match cumulus_command {
            CumulusSubcommands::Get {
                cumulus_export_path,
                name,
            } => {
                println!("Searching for assets named {name}...");
                let assets = get_asset(cumulus_export_path, &name)?;
                if assets.is_empty() {
                    println!("Not assets found");
                    return Ok(());
                }
                for asset in assets.iter() {
                    asset.print();
                }
                Ok(())
            }
            CumulusSubcommands::DiffFields {
                first_cumulus_export_path,
                second_cumulus_export_path,
            } => {
                let fields1 = get_fields(first_cumulus_export_path)?;
                let fields2 = get_fields(second_cumulus_export_path)?;
                let set1: HashSet<_> = fields1.iter().collect();
                let set2: HashSet<_> = fields2.iter().collect();
                for item in set1.symmetric_difference(&set2) {
                    println!("{item}");
                }
                Ok(())
            }
            CumulusSubcommands::LsFields {
                cumulus_export_path,
            } => {
                let mut fields = get_fields(cumulus_export_path)?;
                fields.sort();
                println!("{} fields:", fields.len());
                println!("{}", fields.join(", "));
                Ok(())
            }
        },
        Commands::Images(images_command) => match images_command {
            ImagesSubcommands::Convert {
                cumulus_export_path,
                out_path,
            } => {
                println!(
                    "Converting {} to {}",
                    cumulus_export_path.to_string_lossy(),
                    out_path.to_string_lossy()
                );
                println!("This can take 30 to 60 seconds...");
                convert_images_to_csv(cumulus_export_path, out_path)?;
                Ok(())
            }
            ImagesSubcommands::Import {
                cumulus_export_path,
                release_id,
                releases_base_path,
            } => {
                let images = read_cumulus_export::<_, CumulusImage>(cumulus_export_path)?;
                println!("Retrieved {} images from the Cumulus export", images.len());
                import_images(release_id as i32, images, &releases_base_path).await?;
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
            ReleasesSubcommands::LsExtensions { release_id } => {
                if let Some(release_id) = release_id {
                    list_release_extensions(release_id as i32).await?;
                } else {
                    let releases = crate::db::get_releases().await?;
                    for release in releases.iter() {
                        let banner = "=".repeat(release.name.len());
                        println!("{}", banner);
                        println!("{}", release.name);
                        println!("{}", banner);
                        list_release_extensions(release.id as i32).await?;
                        println!();
                    }
                }
                Ok(())
            }
        },
        Commands::Videos(videos_command) => match videos_command {
            VideosSubcommands::BuildMaster {} => {
                println!("Building master video list from the NIST videos and tapes list");
                let videos = get_nist_videos().await?;
                let tapes = get_nist_tapes().await?;
                let mut master_videos = Vec::new();

                for video in videos.iter() {
                    // There could be many tapes that satisfy this criteria, but we have to pick
                    // one, so we'll just take the first one returned. We will need to look at the
                    // list manually to fix it up. This will be done as a separate process.
                    let tape = tapes.iter().find(|t| t.derived_from == 0).ok_or_else(|| {
                        eyre!(format!(
                            "Could not retrieve tape for video with ID {}",
                            video.video_id
                        ))
                    })?;
                    let master_video = MasterVideo {
                        id: 0,
                        title: video.video_title.clone(),
                        date: video.broadcast_date,
                        description: None,
                        format: Some(tape.format.clone()),
                        network: video.network.clone(),
                        source: Some(tape.tape_source.clone()),
                        notes: video.notes.clone(),
                    };
                    master_videos.push(master_video);
                }

                print!("Saving master video list...");
                save_master_video_list(master_videos).await?;
                print!("done");

                Ok(())
            }
            VideosSubcommands::Convert {
                cumulus_export_path,
                out_path,
            } => {
                println!(
                    "Converting {} to {}",
                    cumulus_export_path.to_string_lossy(),
                    out_path.to_string_lossy()
                );
                println!("This can take 30 to 60 seconds...");
                convert_videos_to_csv(cumulus_export_path, out_path)?;
                Ok(())
            }
        },
    }
}
