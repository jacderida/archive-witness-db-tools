use crate::editing::forms::Form;
use color_eyre::{eyre::eyre, Result};
use db::models::NewsAffiliate;
use dialoguer::Editor;
use std::path::PathBuf;

pub async fn add(path: Option<PathBuf>) -> Result<()> {
    let networks = db::get_news_networks(None).await?;
    let affiliate = if let Some(path) = path {
        let completed_form = std::fs::read_to_string(path)?;
        let form = Form::from_news_affiliate_str(&completed_form)?;
        crate::editing::news::news_affiliate_from_form(0, &form, &networks)?
    } else {
        let form = Form::from(&NewsAffiliate::default());
        match Editor::new().edit(&form.as_string()) {
            Ok(completed_form) => {
                if let Some(cf) = completed_form {
                    let form = Form::from_news_affiliate_str(&cf)?;
                    crate::editing::news::news_affiliate_from_form(0, &form, &networks)?
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

    let updated = db::save_news_affiliate(affiliate).await?;
    println!("===============");
    println!("Saved affiliate");
    println!("===============");
    updated.print();
    Ok(())
}

pub async fn edit(id: u32) -> Result<()> {
    let networks = db::get_news_networks(None).await?;
    let affiliate = db::get_news_affiliate(id as i32, None).await?;
    let form = Form::from(&affiliate);
    let affiliate = match Editor::new().edit(&form.as_string()) {
        Ok(completed_form) => {
            if let Some(cf) = completed_form {
                let form = Form::from_news_affiliate_str(&cf)?;
                crate::editing::news::news_affiliate_from_form(affiliate.id, &form, &networks)?
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

pub async fn ls() -> Result<()> {
    let affiliates = db::get_news_affiliates(None).await?;
    for affiliate in affiliates.iter() {
        affiliate.print_row();
    }
    Ok(())
}

pub async fn print(id: u32) -> Result<()> {
    let affiliate = db::get_news_affiliate(id as i32, None).await?;
    affiliate.print();
    Ok(())
}
