use super::fields::{
    ListField, MultilineTextField, OptionalChoiceListField, OptionalListField,
    OptionalMultilineListField, OptionalMultilineTextField, TextField,
};
use super::{
    forms::{Form, FormError},
    get_people_from_input,
};

use color_eyre::{eyre::eyre, Result};
use db::models::{Category, EventTimestamp, MasterVideo, NewsBroadcast, Person, PersonType};
use std::path::PathBuf;

impl Form {
    pub fn from_master_video_str(s: &str) -> Result<Self, FormError> {
        let parts: Vec<_> = s.split("---\n").collect();
        if parts.len() != 17 {
            return Err(FormError::MalformedForm);
        }

        let mut form = Form::new();

        form.add_field(Box::new(OptionalChoiceListField::from_input_str(
            "News Broadcasts",
            parts[0],
        )?));
        form.add_field(Box::new(TextField::from_input_str("Title", &parts[1])?));
        form.add_field(Box::new(ListField::from_input_str("Categories", parts[2])?));
        form.add_field(Box::new(TextField::from_input_str("Date", &parts[3])?));
        form.add_field(Box::new(MultilineTextField::from_input_str(
            "Description",
            &parts[4],
        )?));
        form.add_field(Box::new(OptionalListField::from_input_str(
            "Links", parts[5],
        )?));
        form.add_field(Box::new(OptionalMultilineListField::from_input_str(
            "Timestamps",
            parts[6],
        )?));
        form.add_field(Box::new(OptionalMultilineTextField::from_input_str(
            "NIST Notes",
            &parts[7],
        )?));
        form.add_field(Box::new(OptionalListField::from_input_str(
            "Eyewitnesses",
            parts[8],
        )?));
        form.add_field(Box::new(OptionalListField::from_input_str(
            "Fire", parts[9],
        )?));
        form.add_field(Box::new(OptionalListField::from_input_str(
            "Police", parts[10],
        )?));
        form.add_field(Box::new(OptionalListField::from_input_str(
            "Port Authority",
            parts[11],
        )?));
        form.add_field(Box::new(OptionalListField::from_input_str(
            "Reporters",
            parts[12],
        )?));
        form.add_field(Box::new(OptionalListField::from_input_str(
            "Survivors",
            parts[13],
        )?));
        form.add_field(Box::new(OptionalListField::from_input_str(
            "Victims", parts[14],
        )?));
        form.add_field(Box::new(OptionalListField::from_input_str(
            "Videographers",
            parts[15],
        )?));
        form.add_field(Box::new(OptionalMultilineListField::from_input_str(
            "NIST Files",
            parts[16],
        )?));

        Ok(form)
    }
}

impl From<&MasterVideo> for Form {
    fn from(model: &MasterVideo) -> Self {
        let mut form = Form::new();

        form.add_field(Box::new(OptionalChoiceListField::new(
            "News Broadcasts",
            &model
                .news_broadcasts
                .iter()
                .map(|b| b.to_string())
                .collect(),
        )));
        form.add_field(Box::new(TextField::new("Title", &model.title)));
        form.add_field(Box::new(ListField::new(
            "Categories",
            &model.categories.iter().map(|c| c.to_string()).collect(),
        )));
        form.add_field(Box::new(TextField::new(
            "Date",
            &model.date.map_or("".to_string(), |d| d.to_string()),
        )));
        form.add_field(Box::new(MultilineTextField::new(
            "Description",
            &model.description,
        )));
        form.add_field(Box::new(OptionalListField::new(
            "Links",
            &model.categories.iter().map(|c| c.to_string()).collect(),
        )));
        form.add_field(Box::new(OptionalMultilineListField::new(
            "Timestamps",
            &model.timestamps.iter().map(|c| c.to_string()).collect(),
        )));
        form.add_field(Box::new(OptionalMultilineTextField::new(
            "NIST Notes",
            &model.nist_notes.as_ref().unwrap_or(&"".to_string()),
        )));
        form.add_field(Box::new(ListField::new(
            "Eyewitnesses",
            &model
                .people
                .iter()
                .filter_map(|p| {
                    if p.types.contains(&PersonType::Eyewitness) {
                        Some(p.name.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        )));
        form.add_field(Box::new(ListField::new(
            "Fire",
            &model
                .people
                .iter()
                .filter_map(|p| {
                    if p.types.contains(&PersonType::Fire) {
                        Some(p.name.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        )));
        form.add_field(Box::new(ListField::new(
            "Police",
            &model
                .people
                .iter()
                .filter_map(|p| {
                    if p.types.contains(&PersonType::Police) {
                        Some(p.name.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        )));
        form.add_field(Box::new(ListField::new(
            "Port Authority",
            &model
                .people
                .iter()
                .filter_map(|p| {
                    if p.types.contains(&PersonType::PortAuthority) {
                        Some(p.name.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        )));
        form.add_field(Box::new(ListField::new(
            "Reporters",
            &model
                .people
                .iter()
                .filter_map(|p| {
                    if p.types.contains(&PersonType::Reporter) {
                        Some(p.name.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        )));
        form.add_field(Box::new(ListField::new(
            "Survivors",
            &model
                .people
                .iter()
                .filter_map(|p| {
                    if p.types.contains(&PersonType::Survivor) {
                        Some(p.name.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        )));
        form.add_field(Box::new(ListField::new(
            "Victims",
            &model
                .people
                .iter()
                .filter_map(|p| {
                    if p.types.contains(&PersonType::Victim) {
                        Some(p.name.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        )));
        form.add_field(Box::new(ListField::new(
            "Videographers",
            &model
                .people
                .iter()
                .filter_map(|p| {
                    if p.types.contains(&PersonType::Videographer) {
                        Some(p.name.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        )));
        form.add_field(Box::new(OptionalMultilineListField::new(
            "NIST Files",
            &model.timestamps.iter().map(|c| c.to_string()).collect(),
        )));

        form
    }
}

pub fn master_video_from_form(
    id: i32,
    form: &Form,
    news_broadcasts: &[NewsBroadcast],
    people: &[Person],
) -> Result<MasterVideo> {
    let broadcasts_input = form
        .get_field_as::<OptionalChoiceListField>("News Broadcasts")?
        .values
        .clone();
    let news_broadcasts = if broadcasts_input.is_empty() {
        Vec::new()
    } else {
        let mut broadcasts = Vec::new();
        for broadcast in broadcasts_input {
            if let Some(broadcast) = news_broadcasts.iter().find(|b| b.to_string() == broadcast) {
                broadcasts.push(broadcast.clone());
            } else {
                return Err(eyre!("{} is not a valid broadcast", broadcast));
            }
        }
        broadcasts
    };

    let title = form.get_field("Title")?.value();

    let categories_input = form.get_field_as::<ListField>("Categories")?.values.clone();
    let categories = categories_input
        .iter()
        .map(|c| Category::from(c.as_str()))
        .collect();

    let date = form.get_field("Date")?.value();
    let description = form.get_field("Description")?.value();
    let links = form
        .get_field_as::<OptionalListField>("Links")?
        .values
        .clone();

    let timestamps_input = form
        .get_field_as::<OptionalMultilineListField>("Timestamps")?
        .values
        .clone();
    let timestamps = timestamps_input
        .iter()
        .map(|t| EventTimestamp::try_from(t.as_str()).unwrap())
        .collect::<Vec<EventTimestamp>>();

    let nist_notes = form.get_field("NIST Notes")?.value();
    let nist_notes = if nist_notes.is_empty() {
        None
    } else {
        Some(nist_notes)
    };

    let pairs = [
        ("Eyewitnesses", PersonType::Eyewitness),
        ("Fire", PersonType::Fire),
        ("Police", PersonType::Police),
        ("Port Authority", PersonType::PortAuthority),
        ("Reporters", PersonType::Reporter),
        ("Survivors", PersonType::Survivor),
        ("Victims", PersonType::Victim),
        ("Videographers", PersonType::Videographer),
    ];

    let mut video_people: Vec<Person> = Vec::new();
    for (prefix, person_type) in pairs.iter() {
        let people_input = form
            .get_field_as::<OptionalListField>(prefix)?
            .values
            .clone();
        let people_input = get_people_from_input(&people_input, people, person_type.clone());
        for person in people_input.iter() {
            if !video_people.iter().any(|p| p.name == person.name) {
                video_people.push(person.clone());
            }
        }
        // This part accommodates the addition of a new type to person without creating a duplicate
        // person with different types.
        for person in video_people.iter_mut() {
            if let Some(p) = people_input.iter().find(|p| p.name == person.name) {
                if !person.types.contains(&p.types[0]) {
                    person.types.push(p.types[0].clone());
                }
            }
        }
    }

    let nist_files_input = form
        .get_field_as::<OptionalMultilineListField>("NIST Files")?
        .values
        .clone();
    let nist_files = nist_files_input
        .iter()
        .map(|n| (PathBuf::from(n), 0))
        .collect();

    let master = MasterVideo {
        categories,
        date: Some(date.parse()?),
        description,
        id,
        links,
        news_broadcasts,
        nist_files,
        nist_notes,
        people: video_people,
        timestamps,
        title,
    };
    Ok(master)
}
