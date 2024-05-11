pub mod editing;
pub mod helpers;
pub mod images;
pub mod releases;
pub mod static_data;

use crate::images::*;
use crate::releases::*;
use archive_wit_db::{
    cumulus::*,
    models::{MasterVideo, Video},
};
use clap::{Parser, Subcommand};
use color_eyre::{eyre::eyre, Result};
use dialoguer::Editor;
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
    #[clap(subcommand)]
    Cumulus(CumulusSubcommands),
    #[clap(subcommand)]
    Images(ImagesSubcommands),
    #[clap(subcommand, name = "masters")]
    MasterVideos(MasterVideosSubcommands),
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

/// Manage master videos
#[derive(Subcommand, Debug)]
enum MasterVideosSubcommands {
    /// Add a master video using an interactive editor.
    #[clap(name = "add")]
    Add {
        /// Path to a file containing a populated video template.
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Edit a master video using an interactive editor.
    #[clap(name = "edit")]
    Edit {
        #[arg(long)]
        id: u32,
    },
    /// Print a master video record
    #[clap(name = "print")]
    Print {
        #[arg(long)]
        id: u32,
    },
}

/// Manage 911datasets.org releases
#[derive(Subcommand, Debug)]
enum ReleasesSubcommands {
    /// Download all the 911datasets.org torrent files.
    ///
    /// The URLs are encoded in the binary.
    #[clap(name = "download-torrents")]
    DownloadTorrents {
        /// The output directory for the torrents
        #[arg(long)]
        path: PathBuf,
    },
    /// Find any files that contain references to a search string.
    ///
    /// For example, you might be looking for files with the term "42A0293 - G27D23" in their path.
    #[clap(name = "find")]
    Find {
        /// The term to search for.
        term: String,
    },
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
    /// List all the file extensions in a release.
    ///
    /// The command can work with an individual, a range, or all releases.
    #[clap(name = "ls-extensions")]
    LsExtensions {
        /// The ID of the release.
        ///
        /// If no ID is supplied, extensions for all releases will be listed.
        #[arg(long)]
        release_id: Option<u32>,
        /// The starting release ID of the range.
        #[arg(long)]
        start_release_id: Option<u32>,
        /// The end release ID of the range.
        #[arg(long)]
        end_release_id: Option<u32>,
    },
    /// List all the files in a release.
    #[clap(name = "ls-files")]
    LsFiles {
        /// The ID of the release.
        #[arg(long)]
        id: u32,
    },
}

/// Manage videos
#[derive(Subcommand, Debug)]
enum VideosSubcommands {
    /// Add a video using an interactive editor or from a templated file.
    #[clap(name = "add")]
    Add {
        /// Path to a file containing a populated video template.
        #[arg(long)]
        path: Option<PathBuf>,
    },
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
    /// Edit a video using an interactive editor.
    #[clap(name = "edit")]
    Edit {
        #[arg(long)]
        id: u32,
    },
    /// Exports the video list for a given range of releases
    #[clap(name = "export")]
    Export {
        /// The end release ID of the range
        #[arg(long)]
        end_release_id: u32,
        /// Path to the output CSV file
        #[arg(long)]
        out_path: PathBuf,
        /// The starting release ID of the range
        #[arg(long)]
        start_release_id: u32,
    },
    /// Exports the master video list to CSV
    #[clap(name = "export-master")]
    ExportMaster {
        /// Path to the output CSV file
        #[arg(long)]
        out_path: PathBuf,
    },
    /// Print the details of a video.
    #[clap(name = "print")]
    Print {
        /// The ID of the video.
        #[arg(long)]
        id: u32,
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
                archive_wit_db::import_nist_tapes_table_from_csv(path).await?;
                print!("done");
                Ok(())
            }
            AccessDbSubcommands::ImportVideos { path } => {
                print!("Importing the Videos table from the NIST database...");
                archive_wit_db::import_nist_video_table_from_csv(path).await?;
                print!("done");
                Ok(())
            }
        },
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
        Commands::MasterVideos(master_videos_command) => match master_videos_command {
            MasterVideosSubcommands::Add { path } => {
                let news_broadcasts = archive_wit_db::get_news_broadcasts().await?;
                let people = archive_wit_db::get_people().await?;
                let video = if let Some(path) = path {
                    let edited_template = std::fs::read_to_string(path)?;
                    let video = editing::parse_master_video_editor_template(
                        0,
                        &edited_template,
                        &news_broadcasts,
                        &people,
                    )?;
                    video
                } else {
                    let template = editing::build_master_video_editor_template(
                        &MasterVideo::default(),
                        &news_broadcasts,
                    );
                    match Editor::new().edit(&template) {
                        Ok(edited_template) => {
                            if let Some(edited) = edited_template {
                                let video = editing::parse_master_video_editor_template(
                                    0,
                                    &edited,
                                    &news_broadcasts,
                                    &people,
                                )?;
                                video
                            } else {
                                return Err(eyre!(
                                    "An unknown error occurred when editing the master video"
                                ));
                            }
                        }
                        Err(_) => {
                            println!("New record will not be added to the database");
                            return Ok(());
                        }
                    }
                };

                let updated = archive_wit_db::save_master_video(video).await?;
                println!("==================");
                println!("Saved master video");
                println!("==================");
                updated.print();

                Ok(())
            }
            MasterVideosSubcommands::Edit { id } => {
                let news_broadcasts = archive_wit_db::get_news_broadcasts().await?;
                let people = archive_wit_db::get_people().await?;
                let master_video = archive_wit_db::get_master_video(id as i32, None).await?;

                let template =
                    editing::build_master_video_editor_template(&master_video, &news_broadcasts);
                let edited_master = match Editor::new().edit(&template) {
                    Ok(edited_template) => {
                        if let Some(edited) = edited_template {
                            let video = editing::parse_master_video_editor_template(
                                master_video.id,
                                &edited,
                                &news_broadcasts,
                                &people,
                            )?;
                            video
                        } else {
                            println!("New record will not be added to the database");
                            return Ok(());
                        }
                    }
                    Err(_) => {
                        return Err(eyre!(
                            "An unknown error occurred when editing the master video"
                        ));
                    }
                };

                let updated = archive_wit_db::save_master_video(edited_master).await?;
                println!("==================");
                println!("Saved master video");
                println!("==================");
                updated.print();

                Ok(())
            }
            MasterVideosSubcommands::Print { id } => {
                let master_video = archive_wit_db::get_master_video(id as i32, None).await?;
                master_video.print();
                Ok(())
            }
        },
        Commands::Releases(releases_command) => match releases_command {
            ReleasesSubcommands::DownloadTorrents { path } => {
                download_torrents(&path).await?;
                Ok(())
            }
            ReleasesSubcommands::Find { term } => {
                let results = archive_wit_db::find_release_files(&term).await?;
                for (release_name, files) in results {
                    println!("{release_name}:");
                    for file in files {
                        println!("{}", file.to_string_lossy());
                    }
                    println!();
                }
                Ok(())
            }
            ReleasesSubcommands::Init { torrent_path } => {
                init_releases(torrent_path).await?;
                Ok(())
            }
            ReleasesSubcommands::Ls {} => {
                list_releases().await?;
                Ok(())
            }
            ReleasesSubcommands::LsExtensions {
                release_id,
                start_release_id,
                end_release_id,
            } => {
                if let (Some(start), Some(end)) = (start_release_id, end_release_id) {
                    list_release_range_extensions(start as i32, end as i32).await?;
                } else if let Some(release_id) = release_id {
                    list_release_extensions(release_id as i32).await?;
                } else {
                    let releases = archive_wit_db::get_releases().await?;
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
            ReleasesSubcommands::LsFiles { id } => {
                let release = archive_wit_db::get_release(id as i32).await?;
                for file in release.files.iter() {
                    println!("{}", file.path.to_string_lossy());
                }
                Ok(())
            }
        },
        Commands::Videos(videos_command) => match videos_command {
            VideosSubcommands::Add { path } => {
                let masters = archive_wit_db::get_master_videos().await?;
                let video = Video::default();
                let template = editing::build_video_editor_template(&video, &masters);

                let video = if let Some(path) = path {
                    let edited_template = std::fs::read_to_string(path)?;
                    editing::parse_video_editor_template(video.id, &edited_template, &masters)?
                } else {
                    match Editor::new().edit(&template) {
                        Ok(edited_template) => {
                            if let Some(edited) = edited_template {
                                editing::parse_video_editor_template(video.id, &edited, &masters)?
                            } else {
                                println!("New record will not be added to the database");
                                return Ok(());
                            }
                        }
                        Err(_) => {
                            return Err(eyre!("An unknown error occurred when editing the video"));
                        }
                    }
                };

                let updated = archive_wit_db::save_video(video).await?;
                println!("===========");
                println!("Saved video");
                println!("===========");
                updated.print();

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
            VideosSubcommands::Edit { id } => {
                let masters = archive_wit_db::get_master_videos().await?;
                let video = archive_wit_db::get_video(id as i32, None).await?;
                let template = editing::build_video_editor_template(&video, &masters);

                let edited_video = match Editor::new().edit(&template) {
                    Ok(edited_template) => {
                        if let Some(edited) = edited_template {
                            editing::parse_video_editor_template(video.id, &edited, &masters)?
                        } else {
                            println!("Changes to the video record will not be saved");
                            return Ok(());
                        }
                    }
                    Err(_) => {
                        return Err(eyre!("An unknown error occurred when editing the video"));
                    }
                };

                let updated = archive_wit_db::save_video(edited_video).await?;
                println!("===========");
                println!("Saved video");
                println!("===========");
                updated.print();

                Ok(())
            }
            VideosSubcommands::Export {
                end_release_id,
                out_path,
                start_release_id,
            } => {
                export_video_list(start_release_id as i32, end_release_id as i32, &out_path)
                    .await?;
                Ok(())
            }
            VideosSubcommands::ExportMaster { out_path } => {
                println!(
                    "Exporting master video list to {}",
                    out_path.to_string_lossy()
                );
                export_master_videos(&out_path).await?;
                Ok(())
            }
            VideosSubcommands::Print { id } => {
                let video = archive_wit_db::get_video(id as i32, None).await?;
                video.print();
                Ok(())
            }
        },
    }
}
