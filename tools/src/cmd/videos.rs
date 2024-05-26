use crate::{editing::forms::Form, export_master_videos, releases::export_video_list};
use color_eyre::{eyre::eyre, Result};
use db::{cumulus::convert_videos_to_csv, helpers::parse_duration, models::Video};
use dialoguer::Editor;
use sqlx::postgres::types::PgInterval;
use std::path::{Path, PathBuf};

pub async fn add(
    master_id: Option<u32>,
    path: Option<PathBuf>,
    youtube_id: Option<String>,
) -> Result<()> {
    let video = if let Some(youtube_id) = youtube_id {
        let master = db::get_master_video(
            master_id
                .ok_or_else(|| eyre!("A master ID must be supplied along with a YouTube ID"))?
                as i32,
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
        crate::editing::videos::video_from_form(0, &form, &masters)?
    } else {
        let masters = db::get_master_videos().await?;
        let mut form = Form::from(&Video::default());
        form.add_choices("Master", masters.iter().map(|m| m.title.clone()).collect())?;
        match Editor::new().edit(&form.as_string()) {
            Ok(completed_form) => {
                if let Some(cf) = completed_form {
                    let form = Form::from_video_str(&cf)?;
                    crate::editing::videos::video_from_form(0, &form, &masters)?
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

pub async fn convert(cumulus_export_path: &Path, out_path: &Path) -> Result<()> {
    println!(
        "Converting {} to {}",
        cumulus_export_path.to_string_lossy(),
        out_path.to_string_lossy()
    );
    println!("This can take 30 to 60 seconds...");
    convert_videos_to_csv(cumulus_export_path, out_path)?;
    Ok(())
}

pub async fn edit(id: u32) -> Result<()> {
    let masters = db::get_master_videos().await?;
    let video = db::get_video(id as i32, None).await?;

    let form = Form::from(&video);
    let edited_video = match Editor::new().edit(&form.as_string()) {
        Ok(completed_form) => {
            if let Some(cf) = completed_form {
                let form = Form::from_video_str(&cf)?;
                crate::editing::videos::video_from_form(video.id, &form, &masters)?
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

pub async fn export(end_release_id: u32, out_path: &Path, start_release_id: u32) -> Result<()> {
    export_video_list(start_release_id as i32, end_release_id as i32, out_path).await?;
    Ok(())
}

pub async fn export_master(out_path: &Path) -> Result<()> {
    println!(
        "Exporting master video list to {}",
        out_path.to_string_lossy()
    );
    export_master_videos(out_path).await?;
    Ok(())
}

pub async fn ls() -> Result<()> {
    let videos = db::get_videos().await?;
    for video in videos.iter() {
        video.print_row();
    }
    Ok(())
}

pub async fn print(id: u32) -> Result<()> {
    let video = db::get_video(id as i32, None).await?;
    video.print();
    Ok(())
}
