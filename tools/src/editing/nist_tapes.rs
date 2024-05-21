use super::fields::OptionalMultilineListField;
use super::forms::{Form, FormError};

use color_eyre::Result;
use db::models::NistTape;
use std::path::PathBuf;

impl Form {
    pub fn from_nist_tape_str(s: &str) -> Result<Self, FormError> {
        let mut form = Form::default();
        form.add_field(Box::new(OptionalMultilineListField::from_input_str(
            "NIST Files",
            s,
        )?));
        Ok(form)
    }
}

impl From<&NistTape> for Form {
    fn from(model: &NistTape) -> Self {
        let mut form = Form::default();
        form.add_field(Box::new(OptionalMultilineListField::new(
            "NIST Files",
            &model
                .release_files
                .iter()
                .map(|c| c.0.to_string_lossy().to_string())
                .collect::<Vec<String>>(),
        )));
        form
    }
}

pub fn get_release_files_from_form(form: &Form) -> Result<Vec<(PathBuf, u64)>> {
    let nist_files_input = form
        .get_field_as::<OptionalMultilineListField>("NIST Files")?
        .values
        .clone();
    let nist_files = nist_files_input
        .iter()
        .map(|n| (PathBuf::from(n), 0))
        .collect();
    Ok(nist_files)
}
