use crate::error::{Error, Result};
use crate::static_data::FIELD_NAME_TYPE_MAP;
use crate::ImageContent;
use encoding_rs::*;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use uuid::Uuid;

pub struct Header {
    pub file_type: String,
    pub field_names: Vec<String>,
    pub field_uids: Vec<Uuid>,
}

pub struct Asset {
    pub fields: Vec<(String, String)>,
    pub tags: Vec<String>,
    pub thumbnail: Vec<u8>,
}

pub fn read_cumulus_photo_export<P>(file_path: P) -> Result<Vec<ImageContent>>
where
    P: AsRef<Path>,
{
    let mut file = File::open(file_path)?;
    file.seek(SeekFrom::Start(0))?;
    let header = read_header(&mut file)?;

    let images = Vec::new();
    let mut count = 1;
    loop {
        match read_asset_data(&mut file, header.field_names.clone()) {
            Ok(asset) => {
                // convert the asset to an image
                println!("Read asset {count}: {}", asset.fields.len());
                count += 1;
            }
            Err(err) => match err {
                Error::Io(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        break;
                    }
                }
                _ => return Err(err),
            },
        }
    }

    Ok(images)
}

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
                println!("Read section name: {section_name}");
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

fn read_asset_data(file: &mut File, field_names: Vec<String>) -> Result<Asset> {
    let mut asset_fields = Vec::new();
    let mut asset_tags = Vec::new();
    let mut thumbnail = Vec::new();

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
                                    return Err(Error::CouldNotReadStringField(field_name));
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
                                if field_name == "Thumbnail" {
                                    thumbnail = field_data.clone();
                                }
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

    let asset = Asset {
        fields: asset_fields,
        tags: asset_tags,
        thumbnail,
    };
    Ok(asset)
}
