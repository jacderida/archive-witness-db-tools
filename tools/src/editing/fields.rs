use crate::editing::forms::FormError;
use color_eyre::Result;
use std::any::Any;

pub trait FormField: Any {
    fn name(&self) -> &str;
    fn value(&self) -> String;
    fn as_string(&self) -> String;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct TextField {
    pub name: String,
    pub value: String,
}

impl TextField {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    pub fn from_input_str(name: &str, input: &str) -> Result<Self, FormError> {
        if !input.contains(name) {
            return Err(FormError::MalformedField(name.to_string()));
        }
        let val = input
            .trim_start_matches(&format!("{name}: "))
            .trim()
            .to_string();
        if val.is_empty() {
            return Err(FormError::RequiredFieldEmpty(name.to_string()));
        }
        Ok(TextField::new(name, &val))
    }
}

impl FormField for TextField {
    fn value(&self) -> String {
        self.value.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn as_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("{}:", self.name()));
        let val = self.value();
        if !val.is_empty() {
            s.push_str(&format!(" {}", val));
        }
        s
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct MultilineTextField {
    pub name: String,
    pub value: String,
}

impl MultilineTextField {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    pub fn from_input_str(name: &str, input: &str) -> Result<Self, FormError> {
        if !input.contains(name) {
            return Err(FormError::MalformedField(name.to_string()));
        }
        let val = input
            .trim_start_matches(&format!("{name}:"))
            .trim_start_matches(':')
            .trim()
            .to_string();
        if val.is_empty() {
            return Err(FormError::RequiredFieldEmpty(name.to_string()));
        }
        Ok(MultilineTextField::new(name, &val))
    }
}

impl FormField for MultilineTextField {
    fn name(&self) -> &str {
        &self.name
    }

    fn value(&self) -> String {
        self.value.clone()
    }

    fn as_string(&self) -> String {
        format!("{}:\n{}", self.name(), self.value())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct OptionalMultilineTextField {
    pub name: String,
    pub value: String,
}

impl OptionalMultilineTextField {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    pub fn from_input_str(name: &str, input: &str) -> Result<Self, FormError> {
        if !input.contains(name) {
            return Err(FormError::MalformedField(name.to_string()));
        }
        let val = input
            .trim_start_matches(&format!("{name}:"))
            .trim()
            .to_string();
        Ok(OptionalMultilineTextField::new(name, &val))
    }
}

impl FormField for OptionalMultilineTextField {
    fn name(&self) -> &str {
        &self.name
    }

    fn value(&self) -> String {
        self.value.clone()
    }

    fn as_string(&self) -> String {
        format!("{}:\n{}", self.name(), self.value())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct OptionalChoiceListField {
    pub name: String,
    pub values: Vec<String>,
    pub choices: Vec<String>,
}

impl FormField for OptionalChoiceListField {
    fn value(&self) -> String {
        self.values
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(";")
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn as_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("{}:", self.name()));
        let val = self.value();
        if val.is_empty() {
            s.push('\n');
            s.push_str("## CHOOSE ONE OR DELETE ALL ##\n");
            for choice in self.choices.iter() {
                s.push_str(choice);
                s.push('\n');
            }
        } else {
            s.push_str(&format!(" {val}"));
        }
        s.trim().to_string()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl OptionalChoiceListField {
    pub fn new(name: &str, values: &Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            values: values.clone(),
            choices: Vec::new(),
        }
    }

    pub fn add_choices(&mut self, choices: Vec<String>) {
        self.choices.extend(choices);
    }

    pub fn from_input_str(name: &str, input: &str) -> Result<Self, FormError> {
        if !input.contains(name) {
            return Err(FormError::MalformedField(name.to_string()));
        }

        let name_stripped = input
            .trim_start_matches(&format!("{}: ", name))
            .trim()
            .to_string();
        let values = if name_stripped.is_empty() {
            Vec::new()
        } else {
            let mut values = Vec::new();
            for item in name_stripped.split(';').map(|v| v.trim()) {
                values.push(item.to_string());
            }
            values
        };
        Ok(OptionalChoiceListField::new(name, &values))
    }
}

pub struct ListField {
    pub name: String,
    pub values: Vec<String>,
}

impl ListField {
    pub fn new(name: &str, values: &Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            values: values.clone(),
        }
    }

    pub fn from_input_str(name: &str, input: &str) -> Result<Self, FormError> {
        if !input.contains(name) {
            return Err(FormError::MalformedField(name.to_string()));
        }

        let name_stripped = input
            .trim_start_matches(&format!("{}: ", name))
            .trim()
            .to_string();
        let values = if name_stripped.is_empty() {
            Vec::new()
        } else {
            let mut values = Vec::new();
            for item in name_stripped.split(';').map(|v| v.trim()) {
                values.push(item.to_string());
            }
            values
        };

        if values.is_empty() {
            return Err(FormError::RequiredFieldEmpty(name.to_string()));
        }

        Ok(ListField::new(name, &values))
    }
}

impl FormField for ListField {
    fn name(&self) -> &str {
        &self.name
    }

    fn value(&self) -> String {
        self.values
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("; ")
    }

    fn as_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("{}:", self.name()));
        let val = self.value();
        if !val.is_empty() {
            s.push_str(&format!(" {}", val));
        }
        s
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct OptionalListField {
    pub name: String,
    pub values: Vec<String>,
}

impl OptionalListField {
    pub fn new(name: &str, values: &Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            values: values.clone(),
        }
    }

    pub fn from_input_str(name: &str, input: &str) -> Result<Self, FormError> {
        if !input.contains(name) {
            return Err(FormError::MalformedField(name.to_string()));
        }

        let name_stripped = input
            .trim_start_matches(&format!("{name}:"))
            .trim()
            .to_string();
        let values = if name_stripped.is_empty() {
            Vec::new()
        } else {
            let mut values = Vec::new();
            for item in name_stripped.split(';').map(|v| v.trim()) {
                values.push(item.to_string());
            }
            values
        };

        Ok(OptionalListField::new(name, &values))
    }
}

impl FormField for OptionalListField {
    fn name(&self) -> &str {
        &self.name
    }

    fn value(&self) -> String {
        self.values
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("; ")
    }

    fn as_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("{}:", self.name()));
        let val = self.value();
        if !val.is_empty() {
            s.push_str(&format!(" {}", val));
        }
        s
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct MultilineListField {
    pub name: String,
    pub values: Vec<String>,
}

impl FormField for MultilineListField {
    fn name(&self) -> &str {
        &self.name
    }

    fn value(&self) -> String {
        self.values
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn as_string(&self) -> String {
        format!("{}:\n{}", self.name(), self.value())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl MultilineListField {
    fn new(name: &str, values: &Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            values: values.clone(),
        }
    }

    pub fn from_input_str(name: &str, input: &str) -> Result<Self, FormError> {
        if !input.contains(name) {
            return Err(FormError::MalformedField(name.to_string()));
        }

        let name_stripped = input.trim_start_matches(&format!("{name}:")).trim();
        let values = if name_stripped.is_empty() {
            Vec::new()
        } else {
            name_stripped.split('\n').map(|s| s.to_string()).collect()
        };

        if values.is_empty() {
            return Err(FormError::RequiredFieldEmpty(name.to_string()));
        }

        Ok(MultilineListField::new(name, &values))
    }
}

pub struct OptionalMultilineListField {
    pub name: String,
    pub values: Vec<String>,
}

impl FormField for OptionalMultilineListField {
    fn name(&self) -> &str {
        &self.name
    }

    fn value(&self) -> String {
        self.values
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn as_string(&self) -> String {
        format!("{}:\n{}", self.name(), self.value())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl OptionalMultilineListField {
    pub fn new(name: &str, values: &Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            values: values.clone(),
        }
    }

    pub fn from_input_str(name: &str, input: &str) -> Result<Self, FormError> {
        if !input.contains(name) {
            return Err(FormError::MalformedField(name.to_string()));
        }

        let name_stripped = input.trim_start_matches(&format!("{name}:")).trim();
        let values = if name_stripped.is_empty() {
            Vec::new()
        } else {
            name_stripped.split('\n').map(|s| s.to_string()).collect()
        };

        Ok(OptionalMultilineListField::new(name, &values))
    }
}

pub struct ChoiceField {
    pub name: String,
    pub value: String,
    pub choices: Vec<String>,
}

impl FormField for ChoiceField {
    fn value(&self) -> String {
        self.value.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn as_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("{}:", self.name()));
        let val = self.value();
        if val.is_empty() {
            s.push('\n');
            s.push_str("## CHOOSE ONE ##\n");
            for choice in self.choices.iter() {
                s.push_str(choice);
                s.push('\n');
            }
        } else {
            s.push_str(&format!(" {val}"));
        }
        s.trim().to_string()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ChoiceField {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            choices: Vec::new(),
        }
    }

    pub fn add_choices(&mut self, choices: Vec<String>) {
        self.choices.extend(choices);
    }

    pub fn from_input_str(name: &str, input: &str) -> Result<Self, FormError> {
        if !input.contains(name) {
            return Err(FormError::MalformedField(name.to_string()));
        }
        let val = input
            .trim_start_matches(&format!("{name}: "))
            .trim()
            .to_string();
        if val.is_empty() {
            return Err(FormError::RequiredFieldEmpty(name.to_string()));
        }
        Ok(ChoiceField::new(name, &val))
    }
}

pub struct OptionalChoiceField {
    pub name: String,
    pub value: String,
    pub choices: Vec<String>,
}

impl FormField for OptionalChoiceField {
    fn value(&self) -> String {
        self.value.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn as_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("{}:", self.name()));
        let val = self.value();
        if val.is_empty() {
            s.push('\n');
            // The check here is to support edits, where the initial choice has already been made.
            // On edit commands, the choices don't get populated.
            if !self.choices.is_empty() {
                s.push_str("## CHOOSE ONE OR NONE ##\n");
                for choice in self.choices.iter() {
                    s.push_str(choice);
                    s.push('\n');
                }
            }
        } else {
            s.push_str(&format!(" {val}"));
        }
        s.trim().to_string()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl OptionalChoiceField {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            choices: Vec::new(),
        }
    }

    pub fn add_choices(&mut self, choices: Vec<String>) {
        self.choices.extend(choices);
    }

    pub fn from_input_str(name: &str, input: &str) -> Result<Self, FormError> {
        if !input.contains(name) {
            return Err(FormError::MalformedField(name.to_string()));
        }
        let val = input
            .trim_start_matches(&format!("{name}:"))
            .trim()
            .to_string();
        Ok(OptionalChoiceField::new(name, &val))
    }
}

pub struct BooleanField {
    pub name: String,
    pub value: bool,
}

impl FormField for BooleanField {
    fn value(&self) -> String {
        if self.value {
            "Yes".to_string()
        } else {
            "No".to_string()
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn as_string(&self) -> String {
        format!("{}: {}", self.name(), self.value())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl BooleanField {
    pub fn new(name: &str, value: bool) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }

    pub fn from_input_str(name: &str, input: &str) -> Result<Self, FormError> {
        if !input.contains(name) {
            return Err(FormError::MalformedField(name.to_string()));
        }

        let val = input
            .trim_start_matches(&format!("{name}: "))
            .trim()
            .to_string();
        if val.is_empty() {
            return Err(FormError::RequiredFieldEmpty(name.to_string()));
        }
        if val.to_lowercase() == "yes" || val.to_lowercase() == "no" {
            return Err(FormError::MalformedField(name.to_string()));
        }

        Ok(BooleanField::new(name, val.to_lowercase() == "yes"))
    }
}
