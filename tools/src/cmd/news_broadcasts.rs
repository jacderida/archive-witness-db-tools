use crate::editing::forms::Form;
use color_eyre::{eyre::eyre, Result};
use db::models::NewsBroadcast;
use dialoguer::Editor;
use std::path::PathBuf;

pub async fn add(path: Option<PathBuf>) -> Result<()> {
    let networks = db::get_news_networks(None).await?;
    let affiliates = db::get_news_affiliates(None).await?;

    let broadcast = if let Some(path) = path {
        let completed_form = std::fs::read_to_string(path)?;
        let form = Form::from_news_broadcast_str(&completed_form)?;
        crate::editing::news::news_broadcast_from_form(0, &form, &networks, &affiliates)?
    } else {
        let mut form = Form::from(&NewsBroadcast::default());
        form.add_choices("Network", networks.iter().map(|n| n.name.clone()).collect())?;
        form.add_choices(
            "Affiliate",
            affiliates.iter().map(|a| a.name.clone()).collect(),
        )?;
        match Editor::new().edit(&form.as_string()) {
            Ok(completed_form) => {
                if let Some(cf) = completed_form {
                    let form = Form::from_news_broadcast_str(&cf)?;
                    crate::editing::news::news_broadcast_from_form(
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
                return Err(eyre!("An unknown error occurred when editing the video"));
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

pub async fn edit(id: u32) -> Result<()> {
    let networks = db::get_news_networks(None).await?;
    let affiliates = db::get_news_affiliates(None).await?;
    let broadcast = db::get_news_broadcast(id as i32, None).await?;
    let form = Form::from(&broadcast);

    let broadcast = match Editor::new().edit(&form.as_string()) {
        Ok(completed_form) => {
            if let Some(cf) = completed_form {
                let form = Form::from_news_broadcast_str(&cf)?;
                crate::editing::news::news_broadcast_from_form(
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

pub async fn ls() -> Result<()> {
    let broadcasts = db::get_news_broadcasts().await?;
    for broadcast in broadcasts.iter() {
        broadcast.print_row();
    }
    Ok(())
}

pub async fn print(id: u32) -> Result<()> {
    let broadcast = db::get_news_broadcast(id as i32, None).await?;
    broadcast.print();
    Ok(())
}
