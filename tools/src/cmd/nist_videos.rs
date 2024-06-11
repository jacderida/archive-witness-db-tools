use crate::editing::forms::Form;
use color_eyre::{eyre::eyre, Result};
use dialoguer::Editor;
use std::io::Write;

pub async fn edit(id: u32) -> Result<()> {
    let videos = db::get_nist_videos().await?;
    let video = videos
        .into_iter()
        .find(|v| v.video_id as u32 == id)
        .ok_or_else(|| eyre!("Could not find video with ID {id}"))?;

    println!("About to edit {}", video.video_title);
    println!("Proceed? [y/n]");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "y" {
        return Ok(());
    }

    let form = Form::from(&video);
    let (is_missing, additional_notes) = match Editor::new().edit(&form.as_string()) {
        Ok(completed_form) => {
            if let Some(cf) = completed_form {
                let form = Form::from_nist_video_str(&cf)?;
                crate::editing::nist_videos::get_missing_and_additional_notes_field(&form)?
            } else {
                println!("The video will not be saved");
                return Ok(());
            }
        }
        Err(_) => {
            return Err(eyre!("An unknown error occurred when editing the video"));
        }
    };

    let updated = db::save_nist_video(video.video_id, is_missing, &additional_notes).await?;
    println!("===============");
    println!("Saved NIST video");
    println!("===============");
    updated.print();
    Ok(())
}

pub async fn ls() -> Result<()> {
    let videos = db::get_nist_videos().await?;
    for video in videos.iter() {
        video.print_row();
    }
    Ok(())
}
