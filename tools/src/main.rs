pub mod cmd;
pub mod editing;
pub mod helpers;
pub mod releases;
pub mod static_data;

use crate::releases::*;
use clap::{Parser, Subcommand};
use color_eyre::Result;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "tools", version = env!("CARGO_PKG_VERSION"))]
struct Opt {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(subcommand)]
    Cumulus(CumulusSubcommands),
    #[clap(subcommand, name = "masters")]
    MasterVideos(MasterVideosSubcommands),
    #[clap(subcommand)]
    News(NewsSubcommands),
    #[clap(subcommand)]
    Nist(NistSubcommands),
    #[clap(subcommand)]
    Releases(ReleasesSubcommands),
    #[clap(subcommand)]
    Videos(VideosSubcommands),
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

/// Manage master videos
#[derive(Subcommand, Debug)]
enum MasterVideosSubcommands {
    /// Add a master video using an interactive editor.
    ///
    /// The path argument can be used to add a record in a non-interactive fashion, by providing a
    /// completed master video form.
    #[clap(name = "add")]
    Add {
        /// Path to a file containing a completed master video form.
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Edit a master video using an interactive editor.
    #[clap(name = "edit")]
    Edit {
        #[arg(long)]
        id: u32,
    },
    /// List all the master videos
    #[clap(name = "ls")]
    Ls {},
    /// Print a master video record
    #[clap(name = "print")]
    Print {
        #[arg(long)]
        id: u32,
    },
}

/// Manage news broadcasts, networks and affiliates.
#[derive(Subcommand, Debug)]
enum NewsSubcommands {
    #[clap(subcommand)]
    Affiliates(NewsAffiliatesSubcommands),
    #[clap(subcommand)]
    Broadcasts(NewsBroadcastsSubcommands),
    #[clap(subcommand)]
    Networks(NewsNetworksSubcommands),
}

/// Manage news networks
#[derive(Subcommand, Debug)]
enum NewsNetworksSubcommands {
    /// Add a news network
    #[clap(name = "add")]
    Add {
        /// Path to a file containing a populated news network template.
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Edit a news network
    #[clap(name = "edit")]
    Edit {
        /// The ID of the network to edit
        #[arg(long)]
        id: u32,
    },
    /// List all news networks
    #[clap(name = "ls")]
    Ls {},
    /// Print a news network
    #[clap(name = "print")]
    Print {
        /// The ID of the network
        #[arg(long)]
        id: u32,
    },
}

/// Manage news affiliates
#[derive(Subcommand, Debug)]
enum NewsAffiliatesSubcommands {
    /// Add a news affiliate
    #[clap(name = "add")]
    Add {
        /// Path to a file containing a populated news affiliate template.
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Edit a news affiliate
    #[clap(name = "edit")]
    Edit {
        /// The ID of the affiliate to edit
        #[arg(long)]
        id: u32,
    },
    /// List all news affiliates
    #[clap(name = "ls")]
    Ls {},
    /// Print a news affiliate
    #[clap(name = "print")]
    Print {
        /// The ID of the affiliate
        #[arg(long)]
        id: u32,
    },
}

/// Manage news broadcasts
#[derive(Subcommand, Debug)]
enum NewsBroadcastsSubcommands {
    /// Add a news broadcast
    #[clap(name = "add")]
    Add {
        /// Path to a file containing a populated broadcast form.
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Edit a news broadcast
    #[clap(name = "edit")]
    Edit {
        /// The ID of the broadcast to edit
        #[arg(long)]
        id: u32,
    },
    /// List all news broadcasts
    #[clap(name = "ls")]
    Ls {},
    /// Print a news broadcast
    #[clap(name = "print")]
    Print {
        /// The ID of the broadcast
        #[arg(long)]
        id: u32,
    },
}

/// Tools for working with NIST's databases.
#[derive(Subcommand, Debug)]
enum NistSubcommands {
    #[clap(subcommand)]
    Import(NistImportSubcommands),
    #[clap(subcommand)]
    Tapes(NistTapesSubcommands),
    #[clap(subcommand)]
    Videos(NistVideosSubcommands),
}

/// Import CSV exports of NIST's Access database tables into the Postgres database.
#[derive(Subcommand, Debug)]
enum NistImportSubcommands {
    /// Import a CSV export of the NIST Tapes table from their Access database
    ///
    /// The videos table must be imported before the tapes table.
    #[clap(name = "tapes")]
    Tapes {
        /// Path to the CSV export
        #[arg(long)]
        path: PathBuf,
    },
    /// Import a CSV export of the NIST Videos table from their Access database.
    ///
    /// The videos table must be imported before the tapes table.
    #[clap(name = "videos")]
    Videos {
        /// Path to the CSV export
        #[arg(long)]
        path: PathBuf,
    },
}

/// Manage tapes from NIST's database.
#[derive(Subcommand, Debug)]
enum NistTapesSubcommands {
    /// Edit a tape to associate it with released files.
    #[clap(name = "edit")]
    Edit {
        /// The ID of the tape.
        #[arg(long)]
        id: Option<u32>,
    },
    /// List the tapes.
    ///
    /// By default, the duplicate tapes will be filtered.
    #[clap(name = "ls")]
    Ls {
        /// Exclude missing entries from the list.
        #[arg(long)]
        exclude_missing: bool,
        /// Simple contains-based search that will filter records that don't match the search term.
        ///
        /// The search is case insensitive.
        #[arg(long, value_name = "TERM")]
        find: Option<String>,
        /// Only display entries that have not yet been allocated to release files.
        #[arg(long)]
        not_allocated: bool,
        /// The wrap length for the additional notes field.
        ///
        /// Default is 100.
        #[arg(long)]
        wrap_length: Option<usize>,
    },
    /// Print a full tape record.
    #[clap(name = "print")]
    Print {
        /// The ID of the tape.
        #[arg(long)]
        id: u32,
    },
}

/// Manage videos from NIST's database.
#[derive(Subcommand, Debug)]
enum NistVideosSubcommands {
    /// Edit a video to mark as missing or add additional notes.
    #[clap(name = "edit")]
    Edit {
        /// The ID of the video.
        #[arg(long)]
        id: u32,
    },
    /// List the videos.
    ///
    /// By default, the duplicate tapes will be filtered.
    #[clap(name = "ls")]
    Ls {},
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
    /// Manage files for the release
    #[clap(subcommand)]
    Files(ReleasesFilesSubcommands),
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
    /// Print reports for releases.
    #[clap(subcommand)]
    Reports(ReleasesReportsSubcommands),
}

/// Manage videos from NIST's database.
#[derive(Subcommand, Debug)]
enum ReleasesFilesSubcommands {
    /// List the files for the release.
    #[clap(name = "ls")]
    Ls {
        /// The ID of the release.
        #[arg(long)]
        id: u32,
    },
    /// List all the file extensions in a release.
    ///
    /// The command can work with an individual, a range, or all releases.
    #[clap(name = "ls-extensions")]
    LsExtensions {
        /// The end release ID of the range.
        #[arg(long)]
        end_id: Option<u32>,
        /// The ID of the release.
        ///
        /// If no ID is supplied, extensions for all releases will be listed.
        #[arg(long)]
        id: Option<u32>,
        /// The starting release ID of the range.
        #[arg(long)]
        start_id: Option<u32>,
        /// If a range is being used, set this flag to sum the counts over the range.
        ///
        /// If not used, the range will specify the file extensions individually
        #[arg(long)]
        sum: bool,
    },
}

/// Print reports for releases
#[derive(Subcommand, Debug)]
enum ReleasesReportsSubcommands {
    /// Print a report showing the release files that have been allocated to videos from NIST's
    /// access database.
    #[clap(name = "nist-videos-allocated")]
    NistVideosAllocated {},
}

/// Manage videos
#[derive(Subcommand, Debug)]
enum VideosSubcommands {
    /// Add a video to the database.
    ///
    /// It can be added in a few different ways:
    ///
    /// * When no arguments are supplied, an interactive editor is used.
    /// * When the --path argument is used, a completed form can be supplied.
    /// * When the --youtube-id argument is used, the video will be created based on the entry for
    ///   that video in the SQLite database. The --master-id argument must be used in conjunction.
    #[clap(name = "add")]
    Add {
        /// The ID of the master video.
        ///
        /// Use in conjunction with the --youtube-id argument.
        #[arg(long)]
        master_id: Option<u32>,
        /// Path to a file containing a populated video template.
        #[arg(long)]
        path: Option<PathBuf>,
        /// The ID of a YouTube video in the SQLite database.
        ///
        /// If used, the --master-id argument must also be supplied to relate the video to a
        /// master.
        #[arg(long)]
        youtube_id: Option<String>,
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
    /// List all videos
    #[clap(name = "ls")]
    Ls {},
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
        Commands::Cumulus(cumulus_command) => match cumulus_command {
            CumulusSubcommands::DiffFields {
                first_cumulus_export_path,
                second_cumulus_export_path,
            } => {
                cmd::cumulus::diff_fields(&first_cumulus_export_path, &second_cumulus_export_path)
                    .await
            }
            CumulusSubcommands::Get {
                cumulus_export_path,
                name,
            } => cmd::cumulus::get(&cumulus_export_path, &name).await,
            CumulusSubcommands::LsFields {
                cumulus_export_path,
            } => cmd::cumulus::ls_fields(&cumulus_export_path).await,
        },
        Commands::MasterVideos(master_videos_command) => match master_videos_command {
            MasterVideosSubcommands::Add { path } => cmd::master_videos::add(path).await,
            MasterVideosSubcommands::Edit { id } => cmd::master_videos::edit(id).await,
            MasterVideosSubcommands::Ls {} => cmd::master_videos::ls().await,
            MasterVideosSubcommands::Print { id } => cmd::master_videos::print(id).await,
        },
        Commands::News(news_command) => match news_command {
            NewsSubcommands::Affiliates(affiliates_command) => match affiliates_command {
                NewsAffiliatesSubcommands::Add { path } => cmd::news_affiliates::add(path).await,
                NewsAffiliatesSubcommands::Edit { id } => cmd::news_affiliates::edit(id).await,
                NewsAffiliatesSubcommands::Ls {} => cmd::news_affiliates::ls().await,
                NewsAffiliatesSubcommands::Print { id } => cmd::news_affiliates::print(id).await,
            },
            NewsSubcommands::Broadcasts(broadcasts_command) => match broadcasts_command {
                NewsBroadcastsSubcommands::Add { path } => cmd::news_broadcasts::add(path).await,
                NewsBroadcastsSubcommands::Edit { id } => cmd::news_broadcasts::edit(id).await,
                NewsBroadcastsSubcommands::Ls {} => cmd::news_broadcasts::ls().await,
                NewsBroadcastsSubcommands::Print { id } => cmd::news_broadcasts::edit(id).await,
            },
            NewsSubcommands::Networks(networks_command) => match networks_command {
                NewsNetworksSubcommands::Add { path } => cmd::news_networks::add(path).await,
                NewsNetworksSubcommands::Edit { id } => cmd::news_networks::edit(id).await,
                NewsNetworksSubcommands::Ls {} => cmd::news_networks::ls().await,
                NewsNetworksSubcommands::Print { id } => cmd::news_networks::print(id).await,
            },
        },
        Commands::Nist(nist_command) => match nist_command {
            NistSubcommands::Import(import_command) => match import_command {
                NistImportSubcommands::Tapes { path } => cmd::nist_import::tapes(&path).await,
                NistImportSubcommands::Videos { path } => cmd::nist_import::videos(&path).await,
            },
            NistSubcommands::Tapes(tapes_command) => match tapes_command {
                NistTapesSubcommands::Edit { id } => cmd::nist_tapes::edit(id).await,
                NistTapesSubcommands::Ls {
                    exclude_missing,
                    find,
                    not_allocated,
                    wrap_length,
                } => cmd::nist_tapes::ls(find, not_allocated, exclude_missing, wrap_length).await,
                NistTapesSubcommands::Print { id } => cmd::nist_tapes::print(id).await,
            },
            NistSubcommands::Videos(videos_command) => match videos_command {
                NistVideosSubcommands::Edit { id } => cmd::nist_videos::edit(id).await,
                NistVideosSubcommands::Ls {} => cmd::nist_videos::ls().await,
            },
        },
        Commands::Releases(releases_command) => match releases_command {
            ReleasesSubcommands::DownloadTorrents { path } => {
                cmd::releases::download_torrents(&path).await
            }
            ReleasesSubcommands::Files(files_command) => match files_command {
                ReleasesFilesSubcommands::Ls { id } => cmd::releases::files_ls(id).await,
                ReleasesFilesSubcommands::LsExtensions {
                    id,
                    end_id,
                    start_id,
                    sum,
                } => cmd::releases::files_ls_extensions(id, start_id, end_id, sum).await,
            },
            ReleasesSubcommands::Find { term } => cmd::releases::find(&term).await,
            ReleasesSubcommands::Init { torrent_path } => cmd::releases::init(&torrent_path).await,
            ReleasesSubcommands::Ls {} => cmd::releases::ls().await,
            ReleasesSubcommands::Reports(reports_command) => match reports_command {
                ReleasesReportsSubcommands::NistVideosAllocated {} => {
                    cmd::releases::report_nist_videos_allocated().await
                }
            },
        },
        Commands::Videos(videos_command) => match videos_command {
            VideosSubcommands::Add {
                master_id,
                path,
                youtube_id,
            } => cmd::videos::add(master_id, path, youtube_id).await,
            VideosSubcommands::Convert {
                cumulus_export_path,
                out_path,
            } => cmd::videos::convert(&cumulus_export_path, &out_path).await,
            VideosSubcommands::Edit { id } => cmd::videos::edit(id).await,
            VideosSubcommands::Export {
                end_release_id,
                out_path,
                start_release_id,
            } => cmd::videos::export(end_release_id, &out_path, start_release_id).await,
            VideosSubcommands::ExportMaster { out_path } => {
                cmd::videos::export_master(&out_path).await
            }
            VideosSubcommands::Ls {} => cmd::videos::ls().await,
            VideosSubcommands::Print { id } => cmd::videos::print(id).await,
        },
    }
}
