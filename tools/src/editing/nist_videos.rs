use super::fields::{BooleanField, OptionalMultilineTextField};
use super::forms::{Form, FormError};
use color_eyre::Result;
use db::models::NistVideo;

impl Form {
    pub fn from_nist_video_str(s: &str) -> Result<Self, FormError> {
        let parts: Vec<_> = s.split("---\n").collect();
        if parts.len() != 2 {
            return Err(FormError::MalformedForm);
        }

        let mut form = Form::default();
        form.add_field(Box::new(BooleanField::from_input_str(
            "Missing?", parts[0],
        )?));
        form.add_field(Box::new(OptionalMultilineTextField::from_input_str(
            "Additional Notes",
            parts[1],
        )?));
        Ok(form)
    }
}

impl From<&NistVideo> for Form {
    fn from(model: &NistVideo) -> Self {
        let mut form = Form::default();
        form.add_field(Box::new(BooleanField::new("Missing?", model.is_missing)));
        form.add_field(Box::new(OptionalMultilineTextField::new(
            "Additional Notes",
            model.additional_notes.as_ref().unwrap_or(&"".to_string()),
        )));
        form
    }
}

pub fn get_missing_and_additional_notes_field(form: &Form) -> Result<(bool, String)> {
    let is_missing = form.get_field_as::<BooleanField>("Missing?")?.value;
    let additional_notes = form
        .get_field_as::<OptionalMultilineTextField>("Additional Notes")?
        .value
        .clone();
    Ok((is_missing, additional_notes))
}
