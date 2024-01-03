use crate::cumulus::CumulusImage;
use chrono::{NaiveDate, NaiveDateTime};
use color_eyre::{eyre::eyre, Result};
use image::GenericImageView;
use magick_rust::{magick_wand_genesis, MagickWand};
use sqlx::FromRow;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error(transparent)]
    DateParsingError(#[from] chrono::ParseError),
    #[error("The source list must have {0} elements")]
    InvalidLength(u16),
    #[error(transparent)]
    ParseError(#[from] std::num::ParseIntError),
}

#[derive(sqlx::FromRow)]
pub struct NistVideo {
    pub video_id: i32,
    pub video_title: String,
    pub network: String,
    pub broadcast_date: Option<NaiveDate>,
    pub duration_min: i32,
    pub subject: Option<String>,
    pub notes: Option<String>,
}

impl TryFrom<Vec<String>> for NistVideo {
    type Error = ConversionError;

    fn try_from(values: Vec<String>) -> std::result::Result<Self, Self::Error> {
        if values.len() != 7 {
            return Err(ConversionError::InvalidLength(7));
        }

        let video_id: i32 = values[0].parse()?;
        let video_title = values[1].clone();
        let network = values[2].clone();
        let broadcast_date = if values[3].is_empty() {
            None
        } else {
            let date = NaiveDate::parse_from_str(&values[3].clone(), "%m/%d/%y 00:00:00")?;
            Some(date)
        };
        let duration_min: i32 = values[4].parse()?;
        let subject = if values[5].is_empty() {
            None
        } else {
            Some(values[5].clone())
        };
        let notes = if values[6].is_empty() {
            None
        } else {
            Some(values[6].clone())
        };
        Ok(Self {
            video_id,
            video_title,
            network,
            broadcast_date,
            duration_min,
            subject,
            notes,
        })
    }
}

#[derive(sqlx::FromRow)]
pub struct NistTape {
    pub tape_id: i32,
    pub video_id: i32,
    pub tape_name: String,
    pub tape_source: String,
    pub copy: i32,
    pub derived_from: i32,
    pub format: String,
    pub duration_min: i32,
    pub batch: bool,
    pub clips: bool,
    pub timecode: bool,
}

impl TryFrom<Vec<String>> for NistTape {
    type Error = ConversionError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        if value.len() != 11 {
            return Err(ConversionError::InvalidLength(11));
        }

        let tape_id = value[0].parse()?;
        let video_id = value[1].parse()?;
        let tape_name = value[2].clone();
        let tape_source = value[3].clone();
        let copy = value[4].parse()?;
        let derived_from = value[5].parse().unwrap_or(0);
        let format = value[6].clone();
        let duration_min = value[7].parse()?;
        let batch = value[8].parse::<i32>()? != 0; // Assuming 0 for false, non-zero for true
        let clips = value[9].parse::<i32>()? != 0;
        let timecode = value[10].parse::<i32>()? != 0;

        Ok(NistTape {
            tape_id,
            video_id,
            tape_name,
            tape_source,
            copy,
            derived_from,
            format,
            duration_min,
            batch,
            clips,
            timecode,
        })
    }
}
