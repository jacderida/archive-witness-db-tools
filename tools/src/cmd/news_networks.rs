use crate::editing::forms::Form;
use color_eyre::{eyre::eyre, Result};
use db::models::NewsNetwork;
use dialoguer::Editor;
use std::path::PathBuf;

pub async fn add(path: Option<PathBuf>) -> Result<()> {
    let network = if let Some(path) = path {
        let completed_form = std::fs::read_to_string(path)?;
        let form = Form::from_news_network_str(&completed_form)?;
        crate::editing::news::news_network_from_form(0, &form)?
    } else {
        let form = Form::from(&NewsNetwork::default());
        match Editor::new().edit(&form.as_string()) {
            Ok(completed_form) => {
                if let Some(cf) = completed_form {
                    let form = Form::from_news_network_str(&cf)?;
                    crate::editing::news::news_network_from_form(0, &form)?
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

    let updated = db::save_news_network(network).await?;
    println!("=============");
    println!("Saved network");
    println!("=============");
    updated.print();
    Ok(())
}

pub async fn edit(id: u32) -> Result<()> {
    let network = db::get_news_network(id as i32, None).await?;
    let form = Form::from(&network);
    let network = match Editor::new().edit(&form.as_string()) {
        Ok(completed_form) => {
            if let Some(cf) = completed_form {
                let form = Form::from_news_network_str(&cf)?;
                crate::editing::news::news_network_from_form(network.id, &form)?
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

pub async fn ls() -> Result<()> {
    let networks = db::get_news_networks(None).await?;
    for network in networks.iter() {
        network.print_row();
    }
    Ok(())
}

pub async fn print(id: u32) -> Result<()> {
    let network = db::get_news_network(id as i32, None).await?;
    network.print();
    Ok(())
}
