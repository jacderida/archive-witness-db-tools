use super::fields::{ChoiceField, FormField, OptionalChoiceListField};
use color_eyre::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormError {
    #[error("There are no choice fields named {0}")]
    ChoiceFieldNotFound(String),
    #[error("Field {0} not found")]
    FieldNotFound(String),
    #[error("Field {0} is of incorrect type")]
    IncorrectType(String),
    #[error("Invalid value for field {0}")]
    InvalidValue(String),
    #[error("The {0} field is not in the expected form")]
    MalformedField(String),
    #[error("The form string is not in the expected form")]
    MalformedForm,
    #[error("The {0} field requires at least one value")]
    RequiredFieldEmpty(String),
}

pub struct Form {
    pub fields: Vec<Box<dyn FormField>>,
}

impl Form {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn add_field(&mut self, field: Box<dyn FormField>) {
        self.fields.push(field);
    }

    pub fn get_field(&self, name: &str) -> Result<&dyn FormField, FormError> {
        self.fields
            .iter()
            .find(|f| f.name() == name)
            .map(|f| f.as_ref())
            .ok_or_else(|| FormError::FieldNotFound(name.to_string()))
    }

    pub fn get_field_mut(&mut self, name: &str) -> Result<&mut dyn FormField, FormError> {
        self.fields
            .iter_mut()
            .find(|f| f.name() == name)
            .map(|f| f.as_mut())
            .ok_or_else(|| FormError::FieldNotFound(name.to_string()))
    }

    pub fn get_field_as<T: 'static>(&self, name: &str) -> Result<&T, FormError> {
        let field = self.get_field(name)?;
        field
            .as_any()
            .downcast_ref::<T>()
            .ok_or_else(|| FormError::IncorrectType(name.to_string()))
    }

    pub fn get_field_as_mut<T: 'static>(&mut self, name: &str) -> Result<&mut T, FormError> {
        let field = self.get_field_mut(name)?;
        field
            .as_any_mut()
            .downcast_mut::<T>()
            .ok_or_else(|| FormError::IncorrectType(name.to_string()))
    }

    pub fn add_choices(&mut self, field_name: &str, choices: Vec<String>) -> Result<(), FormError> {
        match self.get_field_as_mut::<OptionalChoiceListField>(field_name) {
            Ok(field) => {
                field.add_choices(choices.clone());
                return Ok(());
            }
            Err(_) => {}
        }
        match self.get_field_as_mut::<ChoiceField>(field_name) {
            Ok(field) => {
                field.add_choices(choices.clone());
                return Ok(());
            }
            Err(_) => {}
        }

        Err(FormError::ChoiceFieldNotFound(field_name.to_string()))
    }

    pub fn print(&self) {
        let s = self.as_string();
        println!("{s}");
    }

    pub fn as_string(&self) -> String {
        let mut s = String::new();
        for i in 0..self.fields.len() {
            s.push_str(&self.fields[i].as_string());
            if i < self.fields.len() - 1 {
                s.push_str("\n---\n");
            }
        }
        s.trim().to_string()
    }
}
