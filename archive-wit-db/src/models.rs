use crate::{
    cumulus::CumulusImage,
    helpers::{
        duration_to_string, human_readable_size, interval_to_duration, parse_duration,
        strip_first_two_directories,
    },
};
use chrono::{NaiveDate, NaiveDateTime};
use color_eyre::{eyre::eyre, Result};
use image::GenericImageView;
use magick_rust::{magick_wand_genesis, MagickWand};
use sqlx::{postgres::types::PgInterval, FromRow};
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Clone, FromRow)]
pub struct ReleaseFile {
    pub id: i32,
    pub path: PathBuf,
    pub size: i64,
}

#[derive(Clone)]
pub struct Release {
    pub id: i32,
    pub date: NaiveDate,
    pub name: String,
    pub directory_name: Option<String>,
    pub file_count: Option<i16>,
    pub size: Option<i64>,
    pub torrent_url: Option<String>,
    pub files: Vec<ReleaseFile>,
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
    pub network: Option<String>,
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
            network: Some(network),
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

#[derive(Clone, FromRow)]
pub struct Category {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, FromRow)]
pub struct Network {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Default)]
pub struct MasterVideo {
    pub id: i32,
    pub title: String,
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
    pub categories: Vec<Category>,
    pub networks: Vec<Network>,
    pub links: Vec<String>,
}

impl MasterVideo {
    pub fn print(&self) {
        println!("ID: {}", self.id);
        println!("Title: {}", self.title);
        println!(
            "Date: {}",
            self.date.map_or(String::new(), |d| d.to_string())
        );
        if let Some(description) = &self.description {
            println!("Description: {}", description);
        } else {
            println!("Description:");
        }
        if self.categories.is_empty() {
            println!("Categories:");
        } else {
            println!(
                "Categories: {}",
                self.categories
                    .iter()
                    .map(|c| c.name.clone())
                    .collect::<Vec<String>>()
                    .join(";")
            );
        }
        if self.networks.is_empty() {
            println!("Networks:");
        } else {
            println!(
                "Networks: {}",
                self.networks
                    .iter()
                    .map(|n| n.name.clone())
                    .collect::<Vec<String>>()
                    .join(";")
            );
        }
        if self.links.is_empty() {
            println!("Links:");
        } else {
            println!("Links: {}", self.links.join(";"));
        }
    }

    pub fn to_editor(&self) -> String {
        let categories = if self.categories.is_empty() {
            "".to_string()
        } else {
            self.categories
                .iter()
                .map(|c| c.name.clone())
                .collect::<Vec<String>>()
                .join(";")
        };

        let networks = if self.networks.is_empty() {
            "".to_string()
        } else {
            self.networks
                .iter()
                .map(|n| n.name.clone())
                .collect::<Vec<String>>()
                .join(";")
        };

        let links = if self.links.is_empty() {
            "".to_string()
        } else {
            self.links.join(";")
        };

        format!(
            "Title: {}\n---\nDate: {}\n---\nDescription: {}\n---\nCategories: {}\n---\nNetworks: {}\n---\nLinks: {}",
            self.title,
            self.date.map_or(String::new(), |d| d.to_string()),
            self.description.as_ref().unwrap_or(&String::new()),
            categories,
            networks,
            links
        )
    }

    pub fn update_from_editor(&mut self, edited: &str) -> Result<()> {
        let parts: Vec<_> = edited.split("---\n").collect();
        if parts.len() != 6 {
            return Err(eyre!("Input string does not match expected format"));
        }

        self.title = parts[0].trim_start_matches("Title: ").trim().to_string();

        let date = parts[1].trim_start_matches("Date: ").trim();
        self.date = if date.is_empty() {
            None
        } else {
            Some(date.parse()?)
        };

        let description = parts[2]
            .trim_start_matches("Description: ")
            .trim()
            .to_string();
        self.description = if description.is_empty() {
            None
        } else {
            Some(description)
        };

        let categories = parts[3].trim_start_matches("Categories: ").trim();
        for category in categories.split(';').map(|c| c.trim()) {
            if let None = self.categories.iter().find(|c| c.name == category) {
                self.categories.push(Category {
                    id: 0, // The ID will be applied when the video is saved.
                    name: category.to_string(),
                });
            }
        }

        let networks = parts[4].trim_start_matches("Networks: ").trim();
        for network in networks.split(';').map(|n| n.trim()) {
            if let None = self.networks.iter().find(|n| n.name == network) {
                self.networks.push(Network {
                    id: 0, // The ID will be applied when the video is saved.
                    name: network.to_string(),
                });
            }
        }

        let links = parts[5].trim_start_matches("Links: ").trim();
        for link in links.split(';').map(|u| u.trim()) {
            if !self.links.contains(&link.to_string()) && !link.is_empty() {
                self.links.push(link.to_string());
            }
        }

        Ok(())
    }
}

#[derive(Clone, FromRow)]
pub struct Videographer {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, FromRow)]
pub struct Reporter {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, FromRow)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub historical_title: Option<String>,
}

#[derive(Clone, FromRow)]
pub struct JumperTimestamp {
    pub id: i32,
    pub timestamp: PgInterval,
}

#[derive(Clone, Default)]
pub struct Video {
    pub id: i32,
    pub master: MasterVideo,
    pub title: String,
    pub description: Option<String>,
    pub timestamps: Option<String>,
    pub duration: Option<PgInterval>,
    pub link: Option<String>,
    pub nist_notes: Option<String>,
    pub videographers: Vec<Videographer>,
    pub reporters: Vec<Reporter>,
    pub people: Vec<Person>,
    pub jumper_timestamps: Vec<JumperTimestamp>,
    pub nist_files: Vec<(PathBuf, u64)>,
}

impl Video {
    pub fn print(&self) {
        println!("ID: {}", self.id);
        println!("---");
        println!("Master: {}", self.master.title);
        println!("---");
        println!("Title: {}", self.title);
        println!("---");
        println!(
            "Description:\n{}",
            self.description.as_ref().unwrap_or(&String::new())
        );
        println!("---");
        println!(
            "Timestamps:\n{}",
            self.timestamps.as_ref().unwrap_or(&String::new())
        );
        println!("---");

        if let Some(duration) = &self.duration {
            let d = interval_to_duration(&duration);
            println!(
                "Duration: {:02}:{:02}:{:02}",
                d.num_hours(),
                d.num_minutes() % 60,
                d.num_seconds() % 60
            );
        } else {
            println!("Duration:");
        }
        println!("---");

        println!("Link: {}", self.link.as_ref().unwrap_or(&String::new()));
        println!("---");
        println!(
            "NIST Notes: {}",
            self.nist_notes.as_ref().unwrap_or(&String::new())
        );
        println!("---");

        if self.videographers.is_empty() {
            println!("Videographers:");
        } else {
            println!(
                "Videographers: {}",
                self.videographers
                    .iter()
                    .map(|v| v.name.clone())
                    .collect::<Vec<String>>()
                    .join(";")
            );
        }
        println!("---");

        if self.reporters.is_empty() {
            println!("Reporters:");
        } else {
            println!(
                "Reporters: {}",
                self.reporters
                    .iter()
                    .map(|v| v.name.clone())
                    .collect::<Vec<String>>()
                    .join(";")
            );
        }
        println!("---");

        if self.people.is_empty() {
            println!("People:");
        } else {
            println!(
                "People: {}",
                self.people
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<String>>()
                    .join(";")
            );
        }
        println!("---");

        if self.jumper_timestamps.is_empty() {
            println!("Jumpers:");
        } else {
            println!(
                "Jumpers: {}",
                self.jumper_timestamps
                    .iter()
                    .map(|t| interval_to_duration(&t.timestamp).to_string())
                    .collect::<Vec<String>>()
                    .join(";")
            );
        }
        println!("---");

        if self.nist_files.is_empty() {
            println!("NIST Files:");
        } else {
            println!("NIST Files:");
            for (path, size) in self.nist_files.iter() {
                println!(
                    "{} ({})",
                    strip_first_two_directories(path).to_string_lossy(),
                    human_readable_size(*size)
                );
            }
        }
    }

    pub fn to_editor(&self, master_videos: &Vec<MasterVideo>) -> String {
        let mut template = String::new();

        template.push_str("Master Video: ");
        if self.master.id == 0 {
            template.push_str("\n## SELECT ONE AND DELETE THE OTHERS ##\n");
            for video in master_videos.iter() {
                template.push_str(&format!("{}\n", video.title));
            }
        } else {
            template.push_str(&self.master.title);
        }
        template.push_str("\n---\n");

        template.push_str("Title: ");
        if !self.title.is_empty() {
            template.push_str(&self.title);
        }
        template.push_str("\n---\n");

        // The description field is very likely to be a multiline string, so the actual value can
        // be on a new line.
        template.push_str("Description:\n");
        if let Some(description) = &self.description {
            template.push_str(&description);
        }
        template.push_str("\n---\n");

        // The timestamps field is very likely to be a multiline string, so the actual value can
        // be on a new line.
        template.push_str("Timestamps:\n");
        if let Some(description) = &self.description {
            template.push_str(&description);
        }
        template.push_str("\n---\n");

        template.push_str("Duration: ");
        if let Some(duration) = &self.duration {
            template.push_str(&duration_to_string(&interval_to_duration(duration)));
        }
        template.push_str("\n---\n");

        template.push_str("Link: ");
        if let Some(link) = &self.link {
            template.push_str(&link);
        }
        template.push_str("\n---\n");

        template.push_str("NIST Notes: ");
        if let Some(nist_nodes) = &self.nist_notes {
            template.push_str(&nist_nodes);
        }
        template.push_str("\n---\n");

        template.push_str("Videographers: ");
        if !self.videographers.is_empty() {
            template.push_str(
                &self
                    .videographers
                    .iter()
                    .map(|v| v.name.clone())
                    .collect::<Vec<String>>()
                    .join(";"),
            );
        }
        template.push_str("\n---\n");

        template.push_str("Reporters: ");
        if !self.reporters.is_empty() {
            template.push_str(
                &self
                    .reporters
                    .iter()
                    .map(|r| r.name.clone())
                    .collect::<Vec<String>>()
                    .join(";"),
            );
        }
        template.push_str("\n---\n");

        template.push_str("People: ");
        if !self.people.is_empty() {
            template.push_str(
                &self
                    .people
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<String>>()
                    .join(";"),
            );
        };
        template.push_str("\n---\n");

        template.push_str("Jumpers: ");
        if !self.jumper_timestamps.is_empty() {
            template.push_str(
                &self
                    .jumper_timestamps
                    .iter()
                    .map(|r| interval_to_duration(&r.timestamp).to_string())
                    .collect::<Vec<String>>()
                    .join(";"),
            );
        }
        template.push_str("\n---\n");

        template.push_str("NIST Files: ");
        if !self.nist_files.is_empty() {
            for (path, _) in self.nist_files.iter() {
                template.push_str(&format!("{}\n", path.to_string_lossy()));
            }
        }

        template
    }

    pub fn update_from_editor(&mut self, edited: &str) -> Result<String> {
        let parts: Vec<_> = edited.split("---\n").collect();
        if parts.len() != 12 {
            return Err(eyre!("Input string does not match expected format"));
        }

        let master_title = parts[0]
            .trim_start_matches("Master Title: ")
            .trim_start_matches(':')
            .trim()
            .to_string();
        self.title = parts[1]
            .trim_start_matches("Title:")
            .trim_start_matches(':')
            .trim()
            .to_string();

        let description = parts[2]
            .trim_start_matches("Description:")
            .trim_start_matches(':')
            .trim();
        self.description = if description.is_empty() {
            None
        } else {
            Some(description.to_string())
        };

        let timestamps = parts[3]
            .trim_start_matches("Timestamps:")
            .trim_start_matches(':')
            .trim();
        self.timestamps = if timestamps.is_empty() {
            None
        } else {
            Some(timestamps.to_string())
        };

        let duration = parts[4]
            .trim_start_matches("Duration:")
            .trim_start_matches(':')
            .trim();
        self.duration = if duration.is_empty() {
            None
        } else {
            let duration = parse_duration(duration);
            let interval = PgInterval::try_from(duration)
                .map_err(|_| eyre!("Could not convert duration to PgInterval"))?;
            Some(interval)
        };

        let link = parts[5]
            .trim_start_matches("Link:")
            .trim_start_matches(':')
            .trim();
        self.link = if link.is_empty() {
            None
        } else {
            Some(link.to_string())
        };

        let nist_notes = parts[6]
            .trim_start_matches("NIST Notes:")
            .trim_start_matches(':')
            .trim();
        self.nist_notes = if nist_notes.is_empty() {
            None
        } else {
            Some(nist_notes.to_string())
        };

        let videographers = parts[7]
            .trim_start_matches("Videographers:")
            .trim_start_matches(':')
            .trim();
        if !videographers.is_empty() {
            for videographer in videographers.split(';').map(|v| v.trim()) {
                if let None = self.videographers.iter().find(|v| v.name == videographer) {
                    self.videographers.push(Videographer {
                        id: 0, // The ID will be applied when the video is saved.
                        name: videographer.to_string(),
                    });
                }
            }
        }

        let reporters = parts[8]
            .trim_start_matches("Reporters:")
            .trim_start_matches(':')
            .trim();
        if !reporters.is_empty() {
            for reporter in reporters.split(';').map(|r| r.trim()) {
                if let None = self.reporters.iter().find(|r| r.name == reporter) {
                    self.reporters.push(Reporter {
                        id: 0, // The ID will be applied when the video is saved.
                        name: reporter.to_string(),
                    });
                }
            }
        }

        let people = parts[9]
            .trim_start_matches("People:")
            .trim_start_matches(':')
            .trim();
        if !people.is_empty() {
            for person in people.split(';').map(|p| p.trim()) {
                if let None = self.people.iter().find(|p| p.name == person) {
                    self.people.push(Person {
                        id: 0, // The ID will be applied when the video is saved.
                        name: person.to_string(),
                        historical_title: Some(String::new()),
                    });
                }
            }
        }

        let jumper_timestamps = parts[10]
            .trim_start_matches("Jumpers:")
            .trim_start_matches(':')
            .trim();
        if !jumper_timestamps.is_empty() {
            for timestamp in jumper_timestamps.split(';').map(|t| {
                let time = t.trim();
                let duration = parse_duration(time);
                PgInterval::try_from(duration).unwrap()
            }) {
                if let None = self
                    .jumper_timestamps
                    .iter()
                    .find(|p| p.timestamp == timestamp)
                {
                    self.jumper_timestamps.push(JumperTimestamp {
                        id: 0, // The ID will be applied when the video is saved.
                        timestamp,
                    });
                }
            }
        }

        let nist_files = parts[11]
            .trim_start_matches("NIST Files:")
            .trim_start_matches(':')
            .trim();
        if !nist_files.is_empty() {
            for path in nist_files.split('\n').map(|p| PathBuf::from(p)) {
                self.nist_files.push((path, 0));
            }
        }

        Ok(master_title)
    }
}
