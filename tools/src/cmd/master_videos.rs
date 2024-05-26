use crate::editing::forms::Form;
use color_eyre::{eyre::eyre, Result};
use db::models::MasterVideo;
use dialoguer::Editor;
use std::path::PathBuf;

pub async fn add(path: Option<PathBuf>) -> Result<()> {
    let news_broadcasts = db::get_news_broadcasts().await?;
    let people = db::get_people().await?;
    let video = if let Some(path) = path {
        let completed_form = std::fs::read_to_string(path)?;
        let form = Form::from_master_video_str(&completed_form)?;
        crate::editing::masters::master_video_from_form(0, &form, &news_broadcasts, &people)?
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
                    crate::editing::masters::master_video_from_form(
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

pub async fn edit(id: u32) -> Result<()> {
    let news_broadcasts = db::get_news_broadcasts().await?;
    let people = db::get_people().await?;
    let master_video = db::get_master_video(id as i32, None).await?;

    let form = Form::from(&master_video);
    let edited_master = match Editor::new().edit(&form.as_string()) {
        Ok(completed_form) => {
            if let Some(cf) = completed_form {
                let form = Form::from_master_video_str(&cf)?;
                crate::editing::masters::master_video_from_form(
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

pub async fn ls() -> Result<()> {
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

pub async fn print(id: u32) -> Result<()> {
    let master_video = db::get_master_video(id as i32, None).await?;
    master_video.print();
    Ok(())
}
