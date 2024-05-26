use color_eyre::Result;
use db::cumulus::{get_asset, get_fields};
use std::{collections::HashSet, path::Path};

pub async fn diff_fields(
    first_cumulus_export_path: &Path,
    second_cumulus_export_path: &Path,
) -> Result<()> {
    let fields1 = get_fields(first_cumulus_export_path)?;
    let fields2 = get_fields(second_cumulus_export_path)?;
    let set1: HashSet<_> = fields1.iter().collect();
    let set2: HashSet<_> = fields2.iter().collect();
    for item in set1.symmetric_difference(&set2) {
        println!("{item}");
    }
    Ok(())
}

pub async fn get(cumulus_export_path: &Path, name: &str) -> Result<()> {
    println!("Searching for assets named {name}...");
    let assets = get_asset(cumulus_export_path, name)?;
    if assets.is_empty() {
        println!("Not assets found");
        return Ok(());
    }
    for asset in assets.iter() {
        asset.print();
    }
    Ok(())
}

pub async fn ls_fields(cumulus_export_path: &Path) -> Result<()> {
    let mut fields = get_fields(cumulus_export_path)?;
    fields.sort();
    println!("{} fields:", fields.len());
    println!("{}", fields.join(", "));
    Ok(())
}
