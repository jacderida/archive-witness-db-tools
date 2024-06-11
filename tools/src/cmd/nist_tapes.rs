use crate::editing::forms::Form;
use color_eyre::{eyre::eyre, Result};
use colored::Colorize;
use dialoguer::Editor;
use skim::prelude::*;
use std::io::{Cursor, Write};

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

pub async fn ls(find: Option<String>, filter_found: bool) -> Result<()> {
    let tapes_grouped_by_video = db::get_nist_tapes_grouped_by_video().await?;
    for (video, tapes) in tapes_grouped_by_video.iter() {
        if filter_found && tapes.iter().any(|t| !t.release_files.is_empty()) {
            continue;
        }

        let mut s = String::new();
        s.push_str(&format!("{}: {}", video.video_id, video.video_title));
        if let Some(date) = video.broadcast_date {
            s.push_str(&format!(" ({})", date));
        }
        let c = if video.is_missing {
            s.push_str(" [MISSING]");
            s.red()
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
            println!("* {}", notes.red());
        }
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
