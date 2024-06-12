use crate::{editing::forms::Form, helpers::print_banner};
use color_eyre::{eyre::eyre, Result};
use colored::Colorize;
use dialoguer::Editor;
use skim::prelude::*;
use std::io::{Cursor, Write};

#[derive(Default)]
struct ReportSummary {
    total: usize,
    allocated: usize,
    missing: usize,
}

impl ReportSummary {
    fn print(&self) {
        println!();
        print_banner("Summary");
        println!("Total videos: {}", self.total);
        println!("Allocated videos: {}", self.allocated);
        println!("Missing videos: {}", self.missing);
        println!(
            "Unallocated videos: {}",
            self.total - self.allocated - self.missing
        );
    }
}

pub async fn edit(id: Option<u32>) -> Result<()> {
    let tapes = db::get_nist_tapes().await?;
    let tape = if let Some(id) = id {
        tapes
            .into_iter()
            .find(|t| t.tape_id as u32 == id)
            .ok_or_else(|| eyre!("Could not find tape with ID {id}"))?
    } else {
        let mut search_string = String::new();
        for tape in tapes.iter() {
            search_string.push_str(&format!("{} {}\n", tape.tape_id, tape.tape_name));
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
                crate::editing::nist_tapes::get_release_files_from_form(&form)?
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

pub async fn ls(
    find: Option<String>,
    only_display_unallocated: bool,
    exclude_missing: bool,
) -> Result<()> {
    let mut summary = ReportSummary::default();
    let tapes_grouped_by_video = db::get_nist_tapes_grouped_by_video().await?;
    summary.total = tapes_grouped_by_video.len();

    for (video, tapes) in tapes_grouped_by_video.iter() {
        if tapes.iter().any(|t| !t.release_files.is_empty()) {
            summary.allocated += 1;
            if only_display_unallocated {
                continue;
            }
        }
        if video.is_missing && exclude_missing {
            continue;
        }

        let mut s = String::new();
        s.push_str(&format!("{}: {}", video.video_id, video.video_title));
        if let Some(date) = video.broadcast_date {
            s.push_str(&format!(" ({})", date));
        }
        let c = if video.is_missing {
            summary.missing += 1;
            s.push_str(" [MISSING]");
            s.bright_red().bold()
        } else {
            s.blue()
        };

        if let Some(term) = &find {
            if video
                .video_title
                .to_lowercase()
                .contains(&term.to_lowercase())
            {
                println!("{c}");
            } else {
                continue;
            }
        } else {
            println!("{c}");
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
                if s.to_lowercase().contains(&term.to_lowercase()) {
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

        if let Some(notes) = &video.additional_notes {
            print_additional_notes(notes);
        }
        println!("--------------------------------------------------------------");
    }

    if find.is_none() && !only_display_unallocated && !exclude_missing {
        summary.print();
    }
    Ok(())
}

pub async fn print(id: u32) -> Result<()> {
    let tape = db::get_nist_tapes()
        .await?
        .into_iter()
        .find(|t| t.tape_id == id as i32)
        .ok_or_else(|| eyre!("Could not find tape with ID {id}"))?;
    tape.print();
    Ok(())
}

fn print_additional_notes(notes: &str) {
    let wrapped_lines = textwrap::wrap(notes, 80);
    let indent = "  ";
    if let Some(first_line) = wrapped_lines.first() {
        println!("* {}", first_line.purple());
    }
    for line in wrapped_lines.iter().skip(1) {
        println!("{}{}", indent, line.purple());
    }
}
