pub mod editing;
pub mod helpers;
pub mod releases;
pub mod static_data;

use crate::releases::*;
use clap::{Parser, Subcommand};
use color_eyre::{eyre::eyre, Result};
use colored::Colorize;
use db::{
    cumulus::*,
    helpers::parse_duration,
    models::{MasterVideo, NewsAffiliate, NewsBroadcast, NewsNetwork, Video},
};
use dialoguer::Editor;
use editing::forms::Form;
use skim::prelude::*;
use sqlx::postgres::types::PgInterval;
use std::collections::HashSet;
use std::io::{Cursor, Write};
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
        /// Simple contains-based search that will filter records that don't match the search term.
        #[arg(long, value_name = "TERM")]
        find: Option<String>,
        /// Set to filter out tapes that have release files associated with them.
        ///
        /// The remaining list will be tapes that still need to be located.
        #[arg(long)]
        filter_found: bool,
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
        Commands::MasterVideos(master_videos_command) => match master_videos_command {
            MasterVideosSubcommands::Add { path } => {
                let news_broadcasts = db::get_news_broadcasts().await?;
                let people = db::get_people().await?;
                let video = if let Some(path) = path {
                    let completed_form = std::fs::read_to_string(path)?;
                    let form = Form::from_master_video_str(&completed_form)?;
                    editing::masters::master_video_from_form(0, &form, &news_broadcasts, &people)?
                } else {
                    let mut form = Form::from(&MasterVideo::default());
                    form.add_choices(
                        "News Broadcasts",
                        news_broadcasts.iter().map(|b| b.to_string()).collect(),
                    )?;
                    match Editor::new().edit(&form.as_string()) {
                        Ok(completed_form) => {
                            if let Some(cf) = completed_form {
                                let form = Form::from_master_video_str(&cf)?;
                                editing::masters::master_video_from_form(
                                    0,
                                    &form,
                                    &news_broadcasts,
                                    &people,
                                )?
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

                let updated = db::save_master_video(video).await?;
                println!("==================");
                println!("Saved master video");
                println!("==================");
                updated.print();

                Ok(())
            }
            MasterVideosSubcommands::Edit { id } => {
                let news_broadcasts = db::get_news_broadcasts().await?;
                let people = db::get_people().await?;
                let master_video = db::get_master_video(id as i32, None).await?;

                let form = Form::from(&master_video);
                let edited_master = match Editor::new().edit(&form.as_string()) {
                    Ok(completed_form) => {
                        if let Some(cf) = completed_form {
                            let form = Form::from_master_video_str(&cf)?;
                            editing::masters::master_video_from_form(
                                master_video.id,
                                &form,
                                &news_broadcasts,
                                &people,
                            )?
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

                let updated = db::save_master_video(edited_master).await?;
                println!("==================");
                println!("Saved master video");
                println!("==================");
                updated.print();

                Ok(())
            }
            MasterVideosSubcommands::Ls {} => {
                let masters = db::get_master_videos().await?;
                for master in masters.iter() {
                    let videos = db::get_videos_for_master(master.id).await?;
                    master.print_row();
                    for video in videos.iter() {
                        if video.is_primary {
                            println!(
                                "    {}: {} ({})*",
                                video.id, video.title, video.channel_username
                            );
                        } else {
                            println!(
                                "    {}: {} ({})",
                                video.id, video.title, video.channel_username
                            );
                        }
                    }
                }
                Ok(())
            }
            MasterVideosSubcommands::Print { id } => {
                let master_video = db::get_master_video(id as i32, None).await?;
                master_video.print();
                Ok(())
            }
        },
        Commands::News(news_command) => match news_command {
            NewsSubcommands::Affiliates(affiliates_command) => match affiliates_command {
                NewsAffiliatesSubcommands::Add { path } => {
                    let networks = db::get_news_networks(None).await?;
                    let affiliate = if let Some(path) = path {
                        let completed_form = std::fs::read_to_string(path)?;
                        let form = Form::from_news_affiliate_str(&completed_form)?;
                        editing::news::news_affiliate_from_form(0, &form, &networks)?
                    } else {
                        let form = Form::from(&NewsAffiliate::default());
                        match Editor::new().edit(&form.as_string()) {
                            Ok(completed_form) => {
                                if let Some(cf) = completed_form {
                                    let form = Form::from_news_affiliate_str(&cf)?;
                                    editing::news::news_affiliate_from_form(0, &form, &networks)?
                                } else {
                                    println!("New record will not be added to the database");
                                    return Ok(());
                                }
                            }
                            Err(_) => {
                                return Err(eyre!(
                                    "An unknown error occurred when editing the video"
                                ));
                            }
                        }
                    };

                    let updated = db::save_news_affiliate(affiliate).await?;
                    println!("===============");
                    println!("Saved affiliate");
                    println!("===============");
                    updated.print();
                    Ok(())
                }
                NewsAffiliatesSubcommands::Edit { id } => {
                    let networks = db::get_news_networks(None).await?;
                    let affiliate = db::get_news_affiliate(id as i32, None).await?;
                    let form = Form::from(&affiliate);
                    let affiliate = match Editor::new().edit(&form.as_string()) {
                        Ok(completed_form) => {
                            if let Some(cf) = completed_form {
                                let form = Form::from_news_affiliate_str(&cf)?;
                                editing::news::news_affiliate_from_form(
                                    affiliate.id,
                                    &form,
                                    &networks,
                                )?
                            } else {
                                println!("New record will not be added to the database");
                                return Ok(());
                            }
                        }
                        Err(_) => {
                            return Err(eyre!("An unknown error occurred when editing the video"));
                        }
                    };

                    let updated = db::save_news_affiliate(affiliate).await?;
                    println!("===============");
                    println!("Saved affiliate");
                    println!("===============");
                    updated.print();
                    Ok(())
                }
                NewsAffiliatesSubcommands::Ls {} => {
                    let affiliates = db::get_news_affiliates(None).await?;
                    for affiliate in affiliates.iter() {
                        affiliate.print_row();
                    }
                    Ok(())
                }
                NewsAffiliatesSubcommands::Print { id } => {
                    let affiliate = db::get_news_affiliate(id as i32, None).await?;
                    affiliate.print();
                    Ok(())
                }
            },
            NewsSubcommands::Broadcasts(broadcasts_command) => match broadcasts_command {
                NewsBroadcastsSubcommands::Add { path } => {
                    let networks = db::get_news_networks(None).await?;
                    let affiliates = db::get_news_affiliates(None).await?;

                    let broadcast = if let Some(path) = path {
                        let completed_form = std::fs::read_to_string(path)?;
                        let form = Form::from_news_broadcast_str(&completed_form)?;
                        editing::news::news_broadcast_from_form(0, &form, &networks, &affiliates)?
                    } else {
                        let mut form = Form::from(&NewsBroadcast::default());
                        form.add_choices(
                            "Network",
                            networks.iter().map(|n| n.name.clone()).collect(),
                        )?;
                        form.add_choices(
                            "Affiliate",
                            affiliates.iter().map(|a| a.name.clone()).collect(),
                        )?;
                        match Editor::new().edit(&form.as_string()) {
                            Ok(completed_form) => {
                                if let Some(cf) = completed_form {
                                    let form = Form::from_news_broadcast_str(&cf)?;
                                    editing::news::news_broadcast_from_form(
                                        0,
                                        &form,
                                        &networks,
                                        &affiliates,
                                    )?
                                } else {
                                    println!("New record will not be added to the database");
                                    return Ok(());
                                }
                            }
                            Err(_) => {
                                return Err(eyre!(
                                    "An unknown error occurred when editing the video"
                                ));
                            }
                        }
                    };

                    let updated = db::save_news_broadcast(broadcast).await?;
                    println!("===============");
                    println!("Saved broadcast");
                    println!("===============");
                    updated.print();
                    Ok(())
                }
                NewsBroadcastsSubcommands::Edit { id } => {
                    let networks = db::get_news_networks(None).await?;
                    let affiliates = db::get_news_affiliates(None).await?;
                    let broadcast = db::get_news_broadcast(id as i32, None).await?;
                    let form = Form::from(&broadcast);

                    let broadcast = match Editor::new().edit(&form.as_string()) {
                        Ok(completed_form) => {
                            if let Some(cf) = completed_form {
                                let form = Form::from_news_broadcast_str(&cf)?;
                                editing::news::news_broadcast_from_form(
                                    broadcast.id,
                                    &form,
                                    &networks,
                                    &affiliates,
                                )?
                            } else {
                                println!("New record will not be added to the database");
                                return Ok(());
                            }
                        }
                        Err(_) => {
                            return Err(eyre!("An unknown error occurred when editing the video"));
                        }
                    };

                    let updated = db::save_news_broadcast(broadcast).await?;
                    println!("===============");
                    println!("Saved broadcast");
                    println!("===============");
                    updated.print();
                    Ok(())
                }
                NewsBroadcastsSubcommands::Ls {} => {
                    let broadcasts = db::get_news_broadcasts().await?;
                    for broadcast in broadcasts.iter() {
                        broadcast.print_row();
                    }
                    Ok(())
                }
                NewsBroadcastsSubcommands::Print { id } => {
                    let broadcast = db::get_news_broadcast(id as i32, None).await?;
                    broadcast.print();
                    Ok(())
                }
            },
            NewsSubcommands::Networks(networks_command) => match networks_command {
                NewsNetworksSubcommands::Add { path } => {
                    let network = if let Some(path) = path {
                        let completed_form = std::fs::read_to_string(path)?;
                        let form = Form::from_news_network_str(&completed_form)?;
                        editing::news::news_network_from_form(0, &form)?
                    } else {
                        let form = Form::from(&NewsNetwork::default());
                        match Editor::new().edit(&form.as_string()) {
                            Ok(completed_form) => {
                                if let Some(cf) = completed_form {
                                    let form = Form::from_news_network_str(&cf)?;
                                    editing::news::news_network_from_form(0, &form)?
                                } else {
                                    println!("New record will not be added to the database");
                                    return Ok(());
                                }
                            }
                            Err(_) => {
                                return Err(eyre!(
                                    "An unknown error occurred when editing the video"
                                ));
                            }
                        }
                    };

                    let updated = db::save_news_network(network).await?;
                    println!("=============");
                    println!("Saved network");
                    println!("=============");
                    updated.print();
                    Ok(())
                }
                NewsNetworksSubcommands::Edit { id } => {
                    let network = db::get_news_network(id as i32, None).await?;
                    let form = Form::from(&network);
                    let network = match Editor::new().edit(&form.as_string()) {
                        Ok(completed_form) => {
                            if let Some(cf) = completed_form {
                                let form = Form::from_news_network_str(&cf)?;
                                editing::news::news_network_from_form(network.id, &form)?
                            } else {
                                println!("New record will not be added to the database");
                                return Ok(());
                            }
                        }
                        Err(_) => {
                            return Err(eyre!("An unknown error occurred when editing the video"));
                        }
                    };

                    let updated = db::save_news_network(network).await?;
                    println!("=============");
                    println!("Saved network");
                    println!("=============");
                    updated.print();
                    Ok(())
                }
                NewsNetworksSubcommands::Ls {} => {
                    let networks = db::get_news_networks(None).await?;
                    for network in networks.iter() {
                        network.print_row();
                    }
                    Ok(())
                }
                NewsNetworksSubcommands::Print { id } => {
                    let network = db::get_news_network(id as i32, None).await?;
                    network.print();
                    Ok(())
                }
            },
        },
        Commands::Nist(nist_command) => match nist_command {
            NistSubcommands::Import(import_command) => match import_command {
                NistImportSubcommands::Tapes { path } => {
                    print!("Importing the Tapes table from the NIST database...");
                    db::import_nist_tapes_table_from_csv(path).await?;
                    print!("done");
                    Ok(())
                }
                NistImportSubcommands::Videos { path } => {
                    print!("Importing the Videos table from the NIST database...");
                    db::import_nist_videos_table_from_csv(path).await?;
                    print!("done");
                    Ok(())
                }
            },
            NistSubcommands::Tapes(tapes_command) => match tapes_command {
                NistTapesSubcommands::Edit { id } => {
                    let tapes = db::get_nist_tapes().await?;
                    let tape = if let Some(id) = id {
                        tapes
                            .into_iter()
                            .find(|t| t.tape_id as u32 == id)
                            .ok_or_else(|| eyre!("Could not find tape with ID {id}"))?
                    } else {
                        let mut search_string = String::new();
                        for tape in tapes.iter() {
                            search_string
                                .push_str(&format!("{} {}\n", tape.tape_id, tape.tape_name));
                        }
                        let options = SkimOptionsBuilder::default()
                            .height(Some("70%"))
                            .multi(false)
                            .prompt(Some("Please select a tape to edit\n"))
                            .build()?;
                        let item_reader = SkimItemReader::default();
                        let items = item_reader.of_bufread(Cursor::new(search_string));
                        let selected = Skim::run_with(&options, Some(items))
                            .map(|out| out.selected_items)
                            .unwrap();
                        let result = String::from(selected.first().unwrap().output());
                        let split: Vec<String> = result.split(' ').map(String::from).collect();
                        let id: i32 = split[0].parse()?;
                        tapes
                            .into_iter()
                            .find(|t| t.tape_id == id)
                            .ok_or_else(|| eyre!("Could not find tape with ID {id}"))?
                    };

                    println!("About to edit {}", tape.tape_name);
                    println!("Proceed? [y/n]");
                    std::io::stdout().flush()?;
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if input.trim().to_lowercase() != "y" {
                        return Ok(());
                    }

                    let form = Form::from(&tape);
                    let files = match Editor::new().edit(&form.as_string()) {
                        Ok(completed_form) => {
                            if let Some(cf) = completed_form {
                                let form = Form::from_nist_tape_str(&cf)?;
                                editing::nist_tapes::get_release_files_from_form(&form)?
                            } else {
                                println!("New record will not be added to the database");
                                return Ok(());
                            }
                        }
                        Err(_) => {
                            return Err(eyre!("An unknown error occurred when editing the video"));
                        }
                    };

                    let updated = db::save_nist_tape_files(tape.tape_id, files).await?;
                    println!("===============");
                    println!("Saved NIST tape");
                    println!("===============");
                    updated.print();
                    Ok(())
                }
                NistTapesSubcommands::Ls { find, filter_found } => {
                    let tapes_grouped_by_video = db::get_nist_tapes_grouped_by_video().await?;
                    for (video, tapes) in tapes_grouped_by_video.iter() {
                        if filter_found {
                            if tapes.iter().any(|t| !t.release_files.is_empty()) {
                                continue;
                            }
                        }

                        if let Some(term) = &find {
                            if video.video_title.contains(term) {
                                if let Some(date) = video.broadcast_date {
                                    println!(
                                        "{}: {} ({})",
                                        video.video_id,
                                        video.video_title.blue(),
                                        date
                                    );
                                } else {
                                    println!("{}: {}", video.video_id, video.video_title.blue());
                                }
                            } else {
                                continue;
                            }
                        } else if let Some(date) = video.broadcast_date {
                            println!(
                                "{}: {} ({})",
                                video.video_id,
                                video.video_title.blue(),
                                date
                            );
                        } else {
                            println!("{}: {}", video.video_id, video.video_title.blue());
                        }

                        for tape in tapes.iter() {
                            let mut s = String::new();
                            s.push_str(&format!("  {}: ", tape.tape_id));
                            if tape.derived_from == 0 {
                                s.push_str(&format!("{} ", tape.tape_name));
                            } else {
                                s.push_str(&format!("{}", tape.tape_name.yellow()));
                                s.push_str(&format!(" [{}] ", tape.derived_from));
                            }
                            s.push_str(&format!("({}m)", tape.duration_min));

                            if tape.batch {
                                s.push_str(&format!(" {}", "B".to_string().blue()));
                            }
                            if tape.clips {
                                s.push_str(&format!(" {}", "C".to_string().blue()));
                            }
                            if tape.timecode {
                                s.push_str(&format!(" {}", "T".to_string().blue()));
                            }

                            if let Some(term) = &find {
                                if s.contains(term) {
                                    println!("{}", s);
                                    if let Some(nist_refs) = tape.release_ref()? {
                                        for nist_ref in nist_refs {
                                            println!("    {}", nist_ref.green());
                                        }
                                    }
                                }
                            } else {
                                println!("{}", s);
                                if let Some(nist_refs) = tape.release_ref()? {
                                    for nist_ref in nist_refs {
                                        println!("    {}", nist_ref.green());
                                    }
                                }
                            }
                        }
                    }
                    Ok(())
                }
                NistTapesSubcommands::Print { id } => {
                    let tape = db::get_nist_tapes()
                        .await?
                        .into_iter()
                        .find(|t| t.tape_id == id as i32)
                        .ok_or_else(|| eyre!("Could not find tape with ID {id}"))?;
                    tape.print();
                    Ok(())
                }
            },
            NistSubcommands::Videos(videos_command) => match videos_command {
                NistVideosSubcommands::Ls {} => {
                    let videos = db::get_nist_videos().await?;
                    for video in videos.iter() {
                        video.print_row();
                    }
                    Ok(())
                }
            },
        },
        Commands::Releases(releases_command) => match releases_command {
            ReleasesSubcommands::DownloadTorrents { path } => {
                download_torrents(&path).await?;
                Ok(())
            }
            ReleasesSubcommands::Find { term } => {
                let results = db::find_release_files(&term).await?;
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
                    let releases = db::get_releases().await?;
                    for release in releases.iter() {
                        let banner = "=".repeat(release.name.len());
                        println!("{}", banner);
                        println!("{}", release.name);
                        println!("{}", banner);
                        list_release_extensions(release.id).await?;
                        println!();
                    }
                }
                Ok(())
            }
            ReleasesSubcommands::LsFiles { id } => {
                let release = db::get_release(id as i32).await?;
                for file in release.files.iter() {
                    println!("{}", file.path.to_string_lossy());
                }
                Ok(())
            }
        },
        Commands::Videos(videos_command) => match videos_command {
            VideosSubcommands::Add {
                master_id,
                path,
                youtube_id,
            } => {
                let video = if let Some(youtube_id) = youtube_id {
                    let master = db::get_master_video(
                        master_id.ok_or_else(|| {
                            eyre!("A master ID must be supplied along with a YouTube ID")
                        })? as i32,
                        None,
                    )
                    .await?;
                    let yt_video = db_youtube::get_video(&youtube_id).await?;

                    Video {
                        channel_username: yt_video.channel_name,
                        description: yt_video.description,
                        duration: if let Some(duration) = yt_video.duration {
                            PgInterval::try_from(parse_duration(&duration)).unwrap()
                        } else {
                            PgInterval::try_from(parse_duration("0")).unwrap()
                        },
                        id: 0,
                        is_primary: false,
                        link: format!("https://www.youtube.com/watch?v={youtube_id}"),
                        master: master.clone(),
                        title: yt_video.title,
                    }
                } else if let Some(path) = path {
                    let masters = db::get_master_videos().await?;
                    let completed_form = std::fs::read_to_string(path)?;
                    let form = Form::from_video_str(&completed_form)?;
                    editing::videos::video_from_form(0, &form, &masters)?
                } else {
                    let masters = db::get_master_videos().await?;
                    let mut form = Form::from(&Video::default());
                    form.add_choices("Master", masters.iter().map(|m| m.title.clone()).collect())?;
                    match Editor::new().edit(&form.as_string()) {
                        Ok(completed_form) => {
                            if let Some(cf) = completed_form {
                                let form = Form::from_video_str(&cf)?;
                                editing::videos::video_from_form(0, &form, &masters)?
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

                let updated = db::save_video(video).await?;
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
                let masters = db::get_master_videos().await?;
                let video = db::get_video(id as i32, None).await?;

                let form = Form::from(&video);
                let edited_video = match Editor::new().edit(&form.as_string()) {
                    Ok(completed_form) => {
                        if let Some(cf) = completed_form {
                            let form = Form::from_video_str(&cf)?;
                            editing::videos::video_from_form(video.id, &form, &masters)?
                        } else {
                            println!("Changes to the video record will not be saved");
                            return Ok(());
                        }
                    }
                    Err(_) => {
                        return Err(eyre!("An unknown error occurred when editing the video"));
                    }
                };

                let updated = db::save_video(edited_video).await?;
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
            VideosSubcommands::Ls {} => {
                let videos = db::get_videos().await?;
                for video in videos.iter() {
                    video.print_row();
                }
                Ok(())
            }
            VideosSubcommands::Print { id } => {
                let video = db::get_video(id as i32, None).await?;
                video.print();
                Ok(())
            }
        },
    }
}
