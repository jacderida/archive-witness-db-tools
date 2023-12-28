use crate::cumulus::CumulusImage;
use crate::models::{Content, ContentType, Image};
use crate::releases::get_torrent_tree;
use color_eyre::{eyre::eyre, Result};
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn import_images(
    release_id: i32,
    cumulus_image_export: HashMap<String, CumulusImage>,
    releases_base_path: &PathBuf,
    torrent_path: &PathBuf,
) -> Result<()> {
    let tree = get_torrent_tree(torrent_path)?;
    for (path, size) in tree {
        let file_path = releases_base_path.join(path.clone());
        let file_name = file_path
            .file_name()
            .ok_or_else(|| eyre!("Unable to obtain file name"))?
            .to_string_lossy();
        let id = CumulusImage::generate_id(&file_name, size);

        let content = Content {
            id: 0,
            content_type: ContentType::Image,
            file_path: Some(path.to_string_lossy().to_string()),
            release_id: Some(release_id),
        };
        if let Some(cumulus_image) = cumulus_image_export.get(&id) {
            print!(
                "{} present in Cumulus export...",
                file_path.to_string_lossy()
            );
            match Image::try_from_path_with_cumulus_image(&file_path, cumulus_image.clone()) {
                Ok(image) => {
                    crate::db::save_image(content, image).await?;
                    print!("saved");
                    println!();
                }
                Err(e) => {
                    print!("not importing");
                    println!();
                    println!("{}", e);
                }
            }
        } else {
            print!(
                "{} not present in Cumulus export...",
                file_path.to_string_lossy()
            );
            match Image::try_from(file_path).map_err(|e| eyre!(e)) {
                Ok(image) => {
                    crate::db::save_image(content, image).await?;
                    print!("saved");
                    println!();
                }
                Err(e) => {
                    print!("not importing");
                    println!();
                    println!("{}", e);
                }
            }
        }
    }
    Ok(())
}
