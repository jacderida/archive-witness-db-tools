use super::fields::{
    BooleanField, ChoiceField, OptionalMultilineListField, OptionalMultilineTextField, TextField,
};
use super::forms::{Form, FormError};
use color_eyre::{eyre::eyre, Result};
use db::helpers::{duration_to_string, interval_to_duration, parse_duration};
use db::models::{MasterVideo, Video};
use sqlx::postgres::types::PgInterval;

impl Form {
    pub fn from_video_str(s: &str) -> Result<Self, FormError> {
        let parts: Vec<_> = s.split("---\n").collect();
        if parts.len() != 7 {
            return Err(FormError::MalformedForm);
        }

        let mut form = Form::new();

        form.add_field(Box::new(ChoiceField::from_input_str("Master", parts[0])?));
        form.add_field(Box::new(TextField::from_input_str("Title", &parts[1])?));
        form.add_field(Box::new(TextField::from_input_str("Channel", &parts[2])?));
        form.add_field(Box::new(OptionalMultilineListField::from_input_str(
            "Description",
            parts[3],
        )?));
        form.add_field(Box::new(TextField::from_input_str("Link", &parts[4])?));
        form.add_field(Box::new(TextField::from_input_str("Duration", &parts[5])?));
        form.add_field(Box::new(BooleanField::from_input_str(
            "Primary", &parts[6],
        )?));

        Ok(form)
    }
}

impl From<&Video> for Form {
    fn from(video: &Video) -> Self {
        let mut form = Form::new();

        form.add_field(Box::new(ChoiceField::new("Master", &video.master.title)));
        form.add_field(Box::new(TextField::new("Title", &video.title)));
        form.add_field(Box::new(TextField::new("Channel", &video.channel_username)));
        form.add_field(Box::new(OptionalMultilineTextField::new(
            "Description",
            &video.description.as_ref().unwrap_or(&"".to_string()),
        )));
        form.add_field(Box::new(TextField::new("Link", &video.title)));
        form.add_field(Box::new(TextField::new(
            "Duration",
            &duration_to_string(&interval_to_duration(&video.duration)),
        )));
        form.add_field(Box::new(BooleanField::new("Primary", video.is_primary)));

        form
    }
}

pub fn video_from_form(id: i32, form: &Form, masters: &[MasterVideo]) -> Result<Video> {
    let master_title = form.get_field("Master")?.value();
    let master = masters
        .iter()
        .find(|m| m.title == master_title)
        .ok_or_else(|| eyre!("{master_title} is not in the master list"))?;

    let title = form.get_field("Title")?.value();
    let channel_username = form.get_field("Channel")?.value();
    let description = form.get_field("Description")?.value();
    let description = if description.is_empty() {
        None
    } else {
        Some(description)
    };

    let link = form.get_field("Link")?.value();
    let duration = form.get_field("Duration")?.value();
    let duration = PgInterval::try_from(parse_duration(&duration))
        .map_err(|_| eyre!("Could not convert duration string"))?;

    let is_primary = form.get_field("Primary")?.value();
    let is_primary = is_primary.to_lowercase() == "yes";

    let video = Video {
        channel_username,
        description,
        duration,
        id,
        is_primary,
        link,
        master: master.clone(),
        title,
    };
    Ok(video)
}
