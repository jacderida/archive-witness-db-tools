use crate::{
    cumulus::CumulusImage,
    error::{Error, Result},
    helpers::{
        duration_to_string, human_readable_size, interval_to_duration, parse_duration,
        strip_first_two_directories,
    },
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use image::GenericImageView;
use magick_rust::{magick_wand_genesis, MagickWand};
use regex::Regex;
use sqlx::{
    postgres::{
        types::{PgHasArrayType, PgInterval},
        PgTypeInfo,
    },
    FromRow,
};
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
            horizontal_pixels: width,
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
            vertical_pixels: height,
        })
    }

    fn get_file_metadata(path: &PathBuf) -> Result<String> {
        let output = Command::new("file").arg("--brief").arg(path).output();
        match output {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    let error_output = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    Err(Error::FileCommandError(error_output))
                }
            }
            Err(e) => Err(e.into()),
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
                wand.read_image(path.to_str().ok_or_else(|| Error::PathNotObtained)?)?;
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

        let image = Self::new(&file_name, metadata.len() as i64, &file_info, width, height);
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

#[derive(Clone, Debug, PartialEq, sqlx::Type)]
#[sqlx(type_name = "category", rename_all = "lowercase")]
pub enum Category {
    AmateurFootage,
    Compilation,
    Documentary,
    News,
    ProfessionalFootage,
    SurvivorAccount,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let category_str = match self {
            Category::AmateurFootage => "amateur-footage",
            Category::Compilation => "compilation",
            Category::Documentary => "documentary",
            Category::News => "news",
            Category::ProfessionalFootage => "professional-footage",
            Category::SurvivorAccount => "survivor-account",
        };
        write!(f, "{}", category_str)
    }
}

impl From<&str> for Category {
    fn from(s: &str) -> Self {
        match s {
            "amateur-footage" => Category::AmateurFootage,
            "compilation" => Category::Compilation,
            "documentary" => Category::Documentary,
            "news" => Category::News,
            "professional-footage" => Category::ProfessionalFootage,
            _ => panic!("'{s}' is not a valid category"),
        }
    }
}

impl PgHasArrayType for Category {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_category")
    }
}

#[derive(Clone, Debug, sqlx::Type)]
#[sqlx(type_name = "event_type", rename_all = "lowercase")]
pub enum EventType {
    CameraSource,
    Jumper,
    Key,
    Normal,
    Person,
    PentagonAttack,
    Report,
    Wtc1Collapse,
    Wtc1Impact,
    Wtc2Collapse,
    Wtc2Impact,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let event_type_str = match self {
            EventType::CameraSource => "camera-source",
            EventType::Jumper => "jumper",
            EventType::Key => "key",
            EventType::Normal => "normal",
            EventType::Person => "person",
            EventType::PentagonAttack => "pentagon-attack",
            EventType::Report => "report",
            EventType::Wtc1Collapse => "wtc1-collapse",
            EventType::Wtc1Impact => "wtc1-impact",
            EventType::Wtc2Collapse => "wtc2-collapse",
            EventType::Wtc2Impact => "wtc2-impact",
        };
        write!(f, "{}", event_type_str)
    }
}

impl From<&str> for EventType {
    fn from(s: &str) -> Self {
        match s {
            "camera-source" => EventType::CameraSource,
            "jumper" => EventType::Jumper,
            "key" => EventType::Key,
            "normal" => EventType::Normal,
            "person" => EventType::Person,
            "pentagon-attack" => EventType::PentagonAttack,
            "report" => EventType::Report,
            "wtc1-collapse" => EventType::Wtc1Collapse,
            "wtc1-impact" => EventType::Wtc1Impact,
            "wtc2-collapse" => EventType::Wtc2Collapse,
            "wtc2-impact" => EventType::Wtc2Impact,
            _ => panic!("'{s}' is not a valid event type"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EventTimestamp {
    pub id: i32,
    pub description: String,
    pub timestamp: PgInterval,
    pub event_type: EventType,
    pub time_of_day: Option<NaiveTime>,
}

impl TryFrom<&str> for EventTimestamp {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let time_and_desc_regex = Regex::new(
            r"^(?P<time>\d{2}:\d{2}:\d{2}): (?P<description>.+?) \[(?P<time_of_day>\d{4})?\] \[(?P<event_type>[\w-]+)\]$"
        ).map_err(|_| "Regex compilation failed")?;

        let time_and_desc_simple_regex = Regex::new(
            r"^(?P<time>\d{2}:\d{2}:\d{2}): (?P<description>.+?) \[(?P<event_type>[\w-]+)\]$",
        )
        .map_err(|_| "Regex compilation failed")?;

        let caps = time_and_desc_regex
            .captures(s)
            .or_else(|| time_and_desc_simple_regex.captures(s))
            .ok_or("Input string format is incorrect")?;

        let time = caps.name("time").unwrap().as_str();
        let description = caps.name("description").unwrap().as_str().trim();

        let timestamp = match PgInterval::try_from(parse_duration(time)) {
            Ok(timestamp) => timestamp,
            Err(_) => return Err("Invalid timestamp format"),
        };

        let event_type_str = caps.name("event_type").unwrap().as_str();
        let event_type = EventType::from(event_type_str);

        let time_of_day = caps.name("time_of_day").and_then(|tod| {
            let hours = tod.as_str()[0..2].parse::<u32>().ok()?;
            let minutes = tod.as_str()[2..4].parse::<u32>().ok()?;
            NaiveTime::from_hms_opt(hours, minutes, 0)
        });

        Ok(Self {
            id: 0,
            description: description.to_string(),
            timestamp,
            event_type,
            time_of_day,
        })
    }
}

impl std::fmt::Display for EventTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = format!(
            "{}: {}",
            duration_to_string(&interval_to_duration(&self.timestamp)),
            self.description
        );
        if let Some(time) = self.time_of_day {
            s.push_str(&format!(" [{}]", time.format("%H%M")))
        }
        s.push_str(&format!(" [{}]", self.event_type));
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, Default, PartialEq, FromRow)]
pub struct NewsNetwork {
    pub id: i32,
    pub name: String,
    pub description: String,
}

impl NewsNetwork {
    pub fn print(&self) {
        println!("ID: {}", self.id);
        println!("---");
        println!("Name: {}", self.name);
        println!("---");
        println!("Description:\n{}", self.description);
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NewsAffiliate {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub region: String,
    pub network: NewsNetwork,
}

impl NewsAffiliate {
    pub fn print(&self) {
        println!("ID: {}", self.id);
        println!("---");
        println!("Name: {}", self.name);
        println!("---");
        println!("Description:\n{}", self.description);
        println!("---");
        println!("Region:\n{}", self.region);
        println!("---");
        println!("Network:\n{}", self.network.name);
    }
}

#[derive(Clone, Debug)]
pub struct NewsBroadcast {
    pub id: i32,
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
    pub news_network: Option<NewsNetwork>,
    pub news_affiliate: Option<NewsAffiliate>,
}

impl std::fmt::Display for NewsBroadcast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut title = String::new();
        if let Some(network) = &self.news_network {
            title.push_str(&network.name);
        } else if let Some(affiliate) = &self.news_affiliate {
            title.push_str(&affiliate.name);
        }
        if let Some(date) = self.date {
            title.push_str(&format!(" ({})", date.format("%Y-%m-%d")));
        }
        write!(f, "{}", title)
    }
}

#[derive(Clone, Debug, PartialEq, sqlx::Type)]
#[sqlx(type_name = "person_type", rename_all = "lowercase")]
pub enum PersonType {
    Eyewitness,
    Fire,
    Police,
    PortAuthority,
    Reporter,
    Survivor,
    Victim,
    Videographer,
}

impl std::fmt::Display for PersonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let event_type_str = match self {
            PersonType::Eyewitness => "Eyewitness",
            PersonType::Fire => "Fire",
            PersonType::Police => "Police",
            PersonType::PortAuthority => "Port Authority",
            PersonType::Reporter => "Reporter",
            PersonType::Survivor => "Survivor",
            PersonType::Victim => "Victim",
            PersonType::Videographer => "Videographer",
        };
        write!(f, "{}", event_type_str)
    }
}

impl From<&str> for PersonType {
    fn from(s: &str) -> Self {
        match s {
            "eyewitness" => PersonType::Eyewitness,
            "fire" => PersonType::Fire,
            "police" => PersonType::Police,
            "portauthority" => PersonType::PortAuthority,
            "reporter" => PersonType::Reporter,
            "survivor" => PersonType::Survivor,
            "victim" => PersonType::Victim,
            "videographer" => PersonType::Videographer,
            _ => panic!("'{s}' is not a valid person"),
        }
    }
}

impl PgHasArrayType for PersonType {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_person_type")
    }
}

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub historical_title: Option<String>,
    pub description: Option<String>,
    pub types: Vec<PersonType>,
}

#[derive(Clone, Default)]
pub struct MasterVideo {
    pub categories: Vec<Category>,
    pub date: Option<NaiveDate>,
    pub description: String,
    pub id: i32,
    pub links: Vec<String>,
    pub news_broadcasts: Vec<NewsBroadcast>,
    pub nist_files: Vec<(PathBuf, u64)>,
    pub nist_notes: Option<String>,
    pub people: Vec<Person>,
    pub timestamps: Vec<EventTimestamp>,
    pub title: String,
}

impl MasterVideo {
    pub fn print(&self) {
        println!("ID: {}", self.id);
        println!("---");

        if self.news_broadcasts.is_empty() {
            println!("News Broadcasts:");
        } else {
            println!(
                "News Broadcasts: {}",
                self.news_broadcasts
                    .iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<String>>()
                    .join("; ")
            );
        }
        println!("---");

        println!("Title: {}", self.title);
        println!("---");
        println!(
            "Date: {}",
            self.date.map_or(String::new(), |d| d.to_string())
        );
        println!("---");

        println!("Description:\n{}", self.description);
        println!("---");

        if self.categories.is_empty() {
            println!("Categories:");
        } else {
            println!(
                "Categories: {}",
                self.categories
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join("; ")
            );
        }
        println!("---");

        if self.links.is_empty() {
            println!("Links:");
        } else {
            println!("Links: {}", self.links.join("; "));
        }
        println!("---");

        println!(
            "NIST Notes:\n{}",
            self.nist_notes.as_ref().unwrap_or(&String::new())
        );
        println!("---");

        self.print_people("Eyewitnesses", PersonType::Eyewitness);
        println!("---");
        self.print_people("Fire", PersonType::Fire);
        println!("---");
        self.print_people("Police", PersonType::Police);
        println!("---");
        self.print_people("Port Authority", PersonType::PortAuthority);
        println!("---");
        self.print_people("Reporters", PersonType::Reporter);
        println!("---");
        self.print_people("Videographers", PersonType::Videographer);
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

    pub fn people_as_string(&self, prefix: &str, person_type: PersonType) -> String {
        let mut s = String::new();
        s.push_str(prefix);
        s.push(':');
        let people = &self
            .people
            .iter()
            .filter_map(|p| {
                if p.types.contains(&person_type) {
                    Some(p.name.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
        if !people.is_empty() {
            s.push_str(&people.join("; "));
        }
        s
    }

    fn print_people(&self, title: &str, person_type: PersonType) {
        if self.people.iter().any(|p| p.types.contains(&person_type)) {
            println!(
                "{}: {}",
                title,
                self.people
                    .iter()
                    .filter_map(|p| {
                        if p.types.contains(&person_type) {
                            Some(p.name.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("; ")
            );
        } else {
            println!("{title}:");
        }
    }
}

#[derive(Clone)]
pub struct Video {
    pub description: Option<String>,
    pub duration: PgInterval,
    pub id: i32,
    pub is_primary: bool,
    pub link: String,
    pub master: MasterVideo,
    pub title: String,
}

use std::convert::TryFrom;

impl Default for Video {
    fn default() -> Self {
        Self {
            description: None,
            duration: PgInterval::try_from(parse_duration("0")).unwrap(),
            id: 0,
            is_primary: false,
            link: String::default(),
            master: MasterVideo::default(),
            title: String::default(),
        }
    }
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

        let d = interval_to_duration(&self.duration);
        println!(
            "Duration: {:02}:{:02}:{:02}",
            d.num_hours(),
            d.num_minutes() % 60,
            d.num_seconds() % 60
        );
        println!("---");

        println!("Link: {}", self.link);
        println!("---");
        println!("Primary: {}", self.is_primary);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_should_parse_timestamp_without_time_of_day() {
        let input_str =
            "00:08:05: Local coverage commences with an ‘Eyewitness News Special Report’. [normal]";
        let event_timestamp = EventTimestamp::try_from(input_str).unwrap();
        matches!(event_timestamp.event_type, EventType::Normal);
        assert_eq!(
            event_timestamp.description,
            "Local coverage commences with an ‘Eyewitness News Special Report’."
        );
        assert_eq!(event_timestamp.time_of_day, None);
        assert_eq!(
            event_timestamp.timestamp,
            PgInterval::try_from(parse_duration("00:08:05")).unwrap()
        );
    }

    #[test]
    fn try_from_should_parse_timestamp_with_time_of_day() {
        let input_str = "00:20:00: UA175 hits the South Tower during a call with eyewitness Winston Mitchell. [0903] [wtc2-impact]";
        let event_timestamp = EventTimestamp::try_from(input_str).unwrap();
        matches!(event_timestamp.event_type, EventType::Wtc2Impact);
        assert_eq!(
            event_timestamp.description,
            "UA175 hits the South Tower during a call with eyewitness Winston Mitchell."
        );
        assert_eq!(
            event_timestamp.time_of_day,
            NaiveTime::from_hms_opt(9, 3, 0)
        );
        assert_eq!(
            event_timestamp.timestamp,
            PgInterval::try_from(parse_duration("00:20:00")).unwrap()
        );
    }

    #[test]
    fn to_string_should_print_timestamp_without_time_of_day() {
        let timestamp = EventTimestamp {
            id: 1,
            description: "Local coverage commences with an ‘Eyewitness News Special Report’."
                .to_string(),
            event_type: EventType::Normal,
            timestamp: PgInterval::try_from(parse_duration("00:08:05")).unwrap(),
            time_of_day: None,
        };

        let timestamp = timestamp.to_string();
        assert_eq!(
            timestamp,
            "00:08:05: Local coverage commences with an ‘Eyewitness News Special Report’. [normal]"
        );
    }

    #[test]
    fn to_string_should_print_timestamp_with_time_of_day() {
        let timestamp = EventTimestamp {
            id: 1,
            description:
                "UA175 hits the South Tower during a call with eyewitness Winston Mitchell."
                    .to_string(),
            event_type: EventType::Wtc2Impact,
            timestamp: PgInterval::try_from(parse_duration("00:20:00")).unwrap(),
            time_of_day: NaiveTime::from_hms_opt(9, 3, 0),
        };

        let timestamp = timestamp.to_string();
        assert_eq!(
            timestamp,
            "00:20:00: UA175 hits the South Tower during a call with eyewitness Winston Mitchell. [0903] [wtc2-impact]"
        );
    }
}
