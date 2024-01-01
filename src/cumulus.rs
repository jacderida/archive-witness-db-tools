use crate::static_data::FIELD_NAME_TYPE_MAP;
use chrono::NaiveDateTime;
use color_eyre::{eyre::eyre, Result};
use encoding_rs::*;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use uuid::Uuid;

pub struct Header {
    pub file_type: String,
    pub field_names: Vec<String>,
    pub field_uids: Vec<Uuid>,
}

#[derive(Clone, Debug)]
pub struct RawAsset {
    pub fields: Vec<(String, String)>,
    pub tags: Vec<String>,
}

impl RawAsset {
    pub fn print(&self) {
        println!("{} fields", self.fields.len());
        for (name, value) in self.fields.iter() {
            println!("{name}: {value}");
        }
    }
}

pub trait Identifiable {
    fn id(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct CumulusImage {
    pub id: String,
    pub caption: Option<String>,
    pub date_recorded: Option<NaiveDateTime>,
    pub file_size: u64,
    pub horizontal_pixels: Option<u16>,
    pub name: String,
    pub notes: Option<String>,
    pub photographers: Vec<String>,
    pub received_from: Option<String>,
    pub shot_from: Option<String>,
    pub tags: Vec<String>,
    pub vertical_pixels: Option<u16>,
}

impl Identifiable for CumulusImage {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl CumulusImage {
    fn to_csv_row(&self) -> String {
        // The fields with enclosing quotes can have commas in them or can span multiple lines.
        // For larger fields, like notes, those can have quote characters, so they need to be
        // enclosed again in quotes.
        format!(
            "\"{}\",{},\"{}\",{},{},{},{},{},\"{}\",\"{}\",{}",
            self.name.clone().replace("\"", "\"\""),
            self.photographers.join(";"),
            self.shot_from
                .clone()
                .unwrap_or_default()
                .replace("\"", "\"\""),
            self.date_recorded.map_or(String::new(), |d| d.to_string()),
            self.file_size,
            self.horizontal_pixels
                .map_or(String::new(), |p| p.to_string()),
            self.vertical_pixels
                .map_or(String::new(), |p| p.to_string()),
            self.received_from.clone().unwrap_or_default(),
            self.caption
                .clone()
                .unwrap_or_default()
                .replace("\"", "\"\""), // Encapsulate in quotes and escape existing quotes
            self.notes.clone().unwrap_or_default().replace("\"", "\"\""),
            self.tags.join(";"),
        )
    }
}

impl From<RawAsset> for CumulusImage {
    fn from(value: RawAsset) -> Self {
        let caption = value
            .fields
            .iter()
            .find(|a| a.0 == "Caption")
            .and_then(|a| {
                if a.1.is_empty() {
                    None
                } else {
                    Some(a.1.clone())
                }
            });
        let date_recorded = value
            .fields
            .iter()
            .find(|a| a.0 == "Date Recorded")
            .and_then(|a| {
                if a.1.is_empty() {
                    None
                } else {
                    NaiveDateTime::parse_from_str(&a.1, "%Y-%m-%d %H:%M:%S").ok()
                }
            });
        let file_size = value
            .fields
            .iter()
            .find(|a| a.0 == "File Data Size")
            .map(|a| {
                let f: u64 = a.1.parse().unwrap();
                f
            })
            .unwrap();
        let horizontal_pixels = value
            .fields
            .iter()
            .find(|a| a.0 == "Horizontal Pixels")
            .map(|a| {
                if a.1.is_empty() {
                    None
                } else {
                    let h: u16 = a.1.parse().unwrap();
                    Some(h)
                }
            })
            .unwrap();
        let name = value
            .fields
            .iter()
            .find(|a| a.0 == "Asset Name")
            .map(|a| a.1.clone())
            .unwrap();
        let notes = value.fields.iter().find(|a| a.0 == "Notes").and_then(|a| {
            if a.1.is_empty() {
                None
            } else {
                Some(a.1.clone())
            }
        });
        let photographers: Vec<String> = value
            .fields
            .iter()
            .find(|a| a.0 == "Photographer")
            .map(|a| a.1.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        let received_from = value
            .fields
            .iter()
            .find(|a| a.0 == "Received From")
            .and_then(|a| {
                if a.1.is_empty() {
                    None
                } else {
                    Some(a.1.clone())
                }
            });
        let shot_from = value
            .fields
            .iter()
            .find(|a| a.0 == "Shot From")
            .and_then(|a| {
                if a.1.is_empty() {
                    None
                } else {
                    Some(a.1.clone())
                }
            });
        let vertical_pixels = value
            .fields
            .iter()
            .find(|a| a.0 == "Vertical Pixels")
            .and_then(|a| {
                if a.1.is_empty() {
                    None
                } else {
                    let v: u16 = a.1.parse().unwrap();
                    Some(v)
                }
            });

        CumulusImage {
            id: generate_asset_id(&name, file_size),
            caption,
            date_recorded,
            file_size,
            horizontal_pixels,
            name,
            notes,
            photographers,
            received_from,
            shot_from,
            tags: value.tags.clone(),
            vertical_pixels,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CumulusVideo {
    pub id: String,
    pub caption: Option<String>,
    pub date_recorded: Option<NaiveDateTime>,
    pub duration: Option<String>,
    pub file_size: u64,
    pub horizontal_pixels: Option<u16>,
    pub name: String,
    pub notes: Option<String>,
    pub videographers: Vec<String>,
    pub shot_from: Option<String>,
    pub tags: Vec<String>,
    pub vertical_pixels: Option<u16>,
}

impl Identifiable for CumulusVideo {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl From<RawAsset> for CumulusVideo {
    fn from(value: RawAsset) -> Self {
        let caption = value
            .fields
            .iter()
            .find(|a| a.0 == "Caption")
            .and_then(|a| {
                if a.1.is_empty() {
                    None
                } else {
                    Some(a.1.clone())
                }
            });
        let date_recorded = value
            .fields
            .iter()
            .find(|a| a.0 == "Date Recorded")
            .and_then(|a| {
                if a.1.is_empty() {
                    None
                } else {
                    NaiveDateTime::parse_from_str(&a.1, "%Y-%m-%d %H:%M:%S").ok()
                }
            });
        let duration = value
            .fields
            .iter()
            .find(|a| a.0 == "Duration")
            .and_then(|a| {
                if a.1.is_empty() {
                    None
                } else {
                    Some(a.1.clone())
                }
            });
        let file_size = value
            .fields
            .iter()
            .find(|a| a.0 == "File Data Size")
            .map(|a| {
                let f: u64 = a.1.parse().unwrap_or(0);
                f
            })
            .unwrap();
        let horizontal_pixels = value
            .fields
            .iter()
            .find(|a| a.0 == "Horizontal Pixels")
            .map(|a| {
                if a.1.is_empty() {
                    None
                } else {
                    let h: u16 = a.1.parse().unwrap();
                    Some(h)
                }
            })
            .unwrap();
        let name = value
            .fields
            .iter()
            .find(|a| a.0 == "Asset Name")
            .map(|a| a.1.clone())
            .unwrap();
        let notes = value.fields.iter().find(|a| a.0 == "Notes").and_then(|a| {
            if a.1.is_empty() {
                None
            } else {
                Some(a.1.clone())
            }
        });
        let videographers: Vec<String> = value
            .fields
            .iter()
            .find(|a| a.0 == "Photographer")
            .map(|a| a.1.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        let shot_from = value
            .fields
            .iter()
            .find(|a| a.0 == "Shot From")
            .and_then(|a| {
                if a.1.is_empty() {
                    None
                } else {
                    Some(a.1.clone())
                }
            });
        let vertical_pixels = value
            .fields
            .iter()
            .find(|a| a.0 == "Vertical Pixels")
            .and_then(|a| {
                if a.1.is_empty() {
                    None
                } else {
                    let v: u16 = a.1.parse().unwrap();
                    Some(v)
                }
            });

        CumulusVideo {
            id: generate_asset_id(&name, file_size),
            caption,
            date_recorded,
            duration,
            file_size,
            horizontal_pixels,
            name,
            notes,
            videographers,
            shot_from,
            tags: value.tags.clone(),
            vertical_pixels,
        }
    }
}

impl CumulusVideo {
    fn to_csv_row(&self) -> String {
        // The fields with enclosing quotes can have commas in them or can span multiple lines.
        // For larger fields, like notes, those can have quote characters, so they need to be
        // enclosed again in quotes.
        format!(
            "\"{}\",{},\"{}\",{},{},{},{},{},\"{}\",{}",
            self.name.clone().replace("\"", "\"\""),
            self.videographers.join(";"),
            self.shot_from
                .clone()
                .unwrap_or_default()
                .replace("\"", "\"\""),
            self.duration
                .clone()
                .map_or(String::new(), |d| d.to_string()),
            self.date_recorded.map_or(String::new(), |d| d.to_string()),
            self.file_size,
            self.horizontal_pixels
                .map_or(String::new(), |p| p.to_string()),
            self.vertical_pixels
                .map_or(String::new(), |p| p.to_string()),
            self.notes.clone().unwrap_or_default().replace("\"", "\"\""),
            self.tags.join(";"),
        )
    }
}

pub fn generate_asset_id(name: &str, file_size: u64) -> String {
    let mut hasher = Sha1::new();
    hasher.update(name.as_bytes());
    hasher.update(file_size.to_be_bytes());
    let hash = hasher.finalize();
    format!("{:x}", hash)
}

pub fn convert_images_to_csv<P>(cumulus_export_path: P, out_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let images = read_cumulus_export::<_, CumulusImage>(cumulus_export_path)?;
    let mut file = File::create(out_path)?;
    writeln!(file, "name,photographers,shot_from,date_recorded,file_size,horizontal_pixels,vertical_pixels,received_from,caption,notes,tags")?;
    for (_, image) in images {
        writeln!(file, "{}", image.to_csv_row())?;
    }
    Ok(())
}

pub fn convert_videos_to_csv<P>(cumulus_export_path: P, out_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let videos = read_cumulus_export::<_, CumulusVideo>(cumulus_export_path)?;
    let mut file = File::create(out_path)?;
    writeln!(file, "name,videographers,shot_from,duration,date_recorded,file_size,horizontal_pixels,vertical_pixels,notes,tags")?;
    for (_, video) in videos {
        writeln!(file, "{}", video.to_csv_row())?;
    }
    Ok(())
}

pub fn read_cumulus_export<P, T>(file_path: P) -> Result<HashMap<String, T>>
where
    P: AsRef<Path>,
    T: From<RawAsset> + Identifiable + Clone,
{
    let mut file = File::open(file_path)?;
    file.seek(SeekFrom::Start(0))?;
    let header = read_header(&mut file)?;

    let mut items = HashMap::new();
    loop {
        match read_asset_data(&mut file, header.field_names.clone()) {
            Ok(asset) => {
                let item = T::from(asset.clone());
                items.insert(item.id().clone(), item.clone());
            }
            Err(err) => {
                if err.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                }
                return Err(err.into());
            }
        }
    }

    Ok(items)
}

pub fn get_asset<P>(file_path: P, name: &str) -> Result<Vec<RawAsset>>
where
    P: AsRef<Path>,
{
    let mut file = File::open(file_path)?;
    file.seek(SeekFrom::Start(0))?;
    let header = read_header(&mut file)?;

    let mut assets = Vec::new();
    loop {
        match read_asset_data(&mut file, header.field_names.clone()) {
            Ok(asset) => {
                let (_, value) = asset
                    .fields
                    .iter()
                    .find(|a| a.0 == "Asset Name")
                    .ok_or_else(|| eyre!("Could not obtain 'Asset Name' field"))?;
                if value == name {
                    assets.push(asset);
                }
            }
            Err(err) => {
                if err.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                }
                return Err(err.into());
            }
        }
    }

    Ok(assets)
}

pub fn get_fields<P>(file_path: P) -> Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let mut file = File::open(file_path)?;
    file.seek(SeekFrom::Start(0))?;
    let header = read_header(&mut file)?;
    Ok(header.field_names.clone())
}

///
/// Private Helpers
///

fn read_header(file: &mut File) -> Result<Header> {
    let mut buffer = [0; 1];
    let mut current_section = Vec::new();
    let mut header = Header {
        file_type: String::new(),
        field_names: Vec::new(),
        field_uids: Vec::new(),
    };

    // The file type is always from byte 0 to the first 0x0D character.
    while file.read(&mut buffer)? == 1 {
        if buffer[0] == 0x0D {
            break;
        }
        header.file_type.push(buffer[0] as char);
    }

    // Now read the sections.
    //
    // Each section consists of a name and set of fields. The name is in the form %Name0x0D,
    // which needs to be parsed first, then the section has fields that are separated by 0x09
    // (tab), terminating with a 0x0D. The fact that both the section name and the fields are
    // terminated by the same 0x0D makes the code awkward.
    let mut section_name = String::new();
    let mut reading_section_name = true;
    while let Ok(1) = file.read(&mut buffer) {
        if buffer[0] == 0x0D {
            if reading_section_name {
                reading_section_name = false;
                if section_name == "%Data" {
                    break;
                }
            } else {
                let section = String::from_utf8_lossy(&current_section);
                if section_name == "%Fieldnames" {
                    header.field_names = section
                        .split('\t')
                        .map(String::from)
                        .map(|s| s.trim().to_string())
                        .collect();
                } else if section_name == "%FieldUIDs" {
                    header.field_uids = section
                        .split('\t')
                        .filter_map(|s| Some(s.trim_matches(|c| c == '{' || c == '}')))
                        .filter_map(|s| Uuid::parse_str(s).ok())
                        .collect();
                }

                current_section.clear();
                reading_section_name = true;
                section_name = String::new();
            }
        } else {
            if reading_section_name {
                section_name.push(buffer[0] as char);
            } else {
                current_section.push(buffer[0]);
            }
        }
    }

    Ok(header)
}

fn read_asset_data(file: &mut File, field_names: Vec<String>) -> std::io::Result<RawAsset> {
    let mut asset_fields = Vec::new();
    let mut asset_tags = Vec::new();

    let mut buffer = [0; 1];
    let mut field_data = Vec::new();
    let mut field_index = 0;

    loop {
        match file.read_exact(&mut buffer) {
            Ok(_) => {
                match buffer[0] {
                    0x09 => {
                        if field_index == field_names.len() {
                            continue;
                        }

                        let field_name = field_names[field_index].clone();
                        let field_type = *FIELD_NAME_TYPE_MAP.get(&field_name.as_str()).unwrap();
                        match field_type {
                            "String" => {
                                let (field_value, _, had_errors) = WINDOWS_1252.decode(&field_data);
                                if had_errors {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        format!("Could not read '{field_name}' field"),
                                    ));
                                }
                                asset_fields.push((field_name, field_value.to_string()));
                            }
                            "Tag" => {
                                if !field_data.is_empty() {
                                    // They use the character '1' to set the field to true
                                    if field_data[0] == 0x31 {
                                        asset_tags.push(field_name);
                                    }
                                }
                            }
                            "Binary" => {
                                // Nothing to do in this case
                                // The binary data does not seem to represent an image
                                // Or perhaps it's compressed
                            }
                            _ => panic!("Field type '{}' not supported", field_type),
                        }
                        field_index += 1;
                        field_data.clear();
                    }
                    0x0D => break,
                    _ => field_data.push(buffer[0]),
                }
            }
            Err(e) => {
                // This will occur when the end of the file reached.
                // Returning the error causes the loop outside of this function to break.
                return Err(e.into());
            }
        }
    }

    let asset = RawAsset {
        fields: asset_fields,
        tags: asset_tags,
    };
    Ok(asset)
}
