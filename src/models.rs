use crate::cumulus::CumulusImage;
use chrono::{NaiveDate, NaiveDateTime};
use color_eyre::{eyre::eyre, Result};
use image::GenericImageView;
use magick_rust::{magick_wand_genesis, MagickWand};
use sqlx::FromRow;
use std::path::PathBuf;
use std::process::Command;

#[derive(FromRow)]
pub struct Release {
    pub id: i32,
    pub date: NaiveDate,
    pub name: String,
    pub directory_name: Option<String>,
    pub file_count: Option<i16>,
    pub size: Option<i64>,
    pub torrent_url: Option<String>,
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "content_type", rename_all = "lowercase")]
pub enum ContentType {
    Image,
    Video,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Image => "image",
            ContentType::Video => "video",
        }
    }
}

#[derive(FromRow)]
pub struct Content {
    pub id: i32,
    pub content_type: ContentType,
    pub file_path: Option<String>,
    pub release_id: Option<i32>,
}

#[derive(FromRow)]
pub struct Photographer {
    pub id: i32,
    pub name: String,
}

#[derive(FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

pub struct Image {
    pub id: i32,
    pub album: Option<String>,
    pub caption: Option<String>,
    pub date_recorded: Option<NaiveDateTime>,
    pub file_metadata: String,
    pub file_size: i64,
    pub horizontal_pixels: i16,
    pub name: String,
    pub notes: Option<String>,
    pub photographers: Option<Vec<Photographer>>,
    pub received_from: Option<String>,
    pub shot_from: Option<String>,
    pub tags: Option<Vec<Tag>>,
    pub vertical_pixels: i16,
}

impl Image {
    pub fn new(
        name: &str,
        file_size: i64,
        file_metadata: &str,
        horizontal_pixels: i16,
        vertical_pixels: i16,
    ) -> Self {
        Image {
            id: 0,
            name: name.to_string(),
            file_metadata: file_metadata.to_string(),
            file_size,
            horizontal_pixels,
            vertical_pixels,
            album: None,
            caption: None,
            date_recorded: None,
            notes: None,
            photographers: None,
            received_from: None,
            shot_from: None,
            tags: None,
        }
    }

    pub fn add_photographer(&mut self, photographer: Photographer) {
        if let Some(ref mut photographers) = self.photographers {
            photographers.push(photographer);
        }
    }

    pub fn add_tag(&mut self, tag: Tag) {
        if let Some(ref mut tags) = self.tags {
            tags.push(tag);
        }
    }

    pub fn try_from_path_with_cumulus_image(
        path: &PathBuf,
        cumulus_image: CumulusImage,
    ) -> Result<Image> {
        let (width, height) = Self::get_image_dimensions(path)?;
        Ok(Image {
            id: 0,
            album: cumulus_image.album,
            caption: cumulus_image.caption,
            date_recorded: cumulus_image.date_recorded,
            file_metadata: Self::get_file_metadata(path)?,
            file_size: cumulus_image.file_size as i64,
            horizontal_pixels: width as i16,
            name: cumulus_image.name,
            notes: cumulus_image.notes,
            photographers: Some(
                cumulus_image
                    .photographers
                    .into_iter()
                    .map(|name| Photographer { id: 0, name })
                    .collect(),
            ),
            received_from: cumulus_image.received_from,
            shot_from: cumulus_image.shot_from,
            tags: Some(
                cumulus_image
                    .tags
                    .into_iter()
                    .map(|name| Tag { id: 0, name })
                    .collect(),
            ),
            vertical_pixels: height as i16,
        })
    }

    fn get_file_metadata(path: &PathBuf) -> Result<String> {
        let output = Command::new("file").arg("--brief").arg(path).output();
        match output {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    Err(eyre!(String::from_utf8_lossy(&output.stderr)
                        .trim()
                        .to_string()))
                }
            }
            Err(e) => Err(eyre!(e.to_string())),
        }
    }

    fn get_image_dimensions(path: &PathBuf) -> Result<(i16, i16)> {
        match image::open(path) {
            Ok(img) => {
                let (width, height) = img.dimensions();
                Ok((width as i16, height as i16))
            }
            Err(_) => {
                // There are some images the `image` crate can't read
                // Fall back to using ImageMagick
                magick_wand_genesis();
                let wand = MagickWand::new();
                wand.read_image(
                    path.to_str()
                        .ok_or_else(|| eyre!("Could not obtain path"))?,
                )?;
                let width = wand.get_image_width();
                let height = wand.get_image_height();
                Ok((width as i16, height as i16))
            }
        }
    }
}

impl TryFrom<PathBuf> for Image {
    type Error = String;

    fn try_from(path: PathBuf) -> std::result::Result<Self, Self::Error> {
        let file_info = Self::get_file_metadata(&path).map_err(|e| e.to_string())?;
        let (width, height) = Self::get_image_dimensions(&path).map_err(|e| e.to_string())?;
        let metadata = std::fs::metadata(path.clone()).map_err(|e| e.to_string())?;
        let file_name = path
            .file_name()
            .ok_or_else(|| "Could not obtain file name".to_string())?
            .to_string_lossy();

        let image = Self::new(
            &file_name,
            metadata.len() as i64,
            &file_info,
            width as i16,
            height as i16,
        );
        Ok(image)
    }
}
