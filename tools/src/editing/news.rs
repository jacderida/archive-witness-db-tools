use super::fields::{ChoiceField, MultilineTextField, TextField};
use super::forms::{Form, FormError};

use color_eyre::{eyre::eyre, Result};
use db::models::{NewsAffiliate, NewsNetwork};

impl Form {
    pub fn from_news_network_str(s: &str) -> Result<Self, FormError> {
        let parts: Vec<_> = s.split("---\n").collect();
        if parts.len() != 2 {
            return Err(FormError::MalformedForm);
        }

        let mut form = Form::new();
        form.add_field(Box::new(TextField::from_input_str("Name", &parts[0])?));
        form.add_field(Box::new(MultilineTextField::from_input_str(
            "Description",
            &parts[1],
        )?));

        Ok(form)
    }

    pub fn from_news_affiliate_str(s: &str) -> Result<Self, FormError> {
        let parts: Vec<_> = s.split("---\n").collect();
        if parts.len() != 4 {
            return Err(FormError::MalformedForm);
        }

        let mut form = Form::new();
        form.add_field(Box::new(TextField::from_input_str("Name", &parts[0])?));
        form.add_field(Box::new(MultilineTextField::from_input_str(
            "Description",
            &parts[1],
        )?));
        form.add_field(Box::new(TextField::from_input_str("Region", &parts[2])?));
        form.add_field(Box::new(ChoiceField::from_input_str("Network", parts[3])?));

        Ok(form)
    }
}

impl From<&NewsNetwork> for Form {
    fn from(model: &NewsNetwork) -> Self {
        let mut form = Form::new();
        form.add_field(Box::new(TextField::new("Name", &model.name)));
        form.add_field(Box::new(MultilineTextField::new(
            "Description",
            &model.description,
        )));
        form
    }
}

pub fn news_network_from_form(id: i32, form: &Form) -> Result<NewsNetwork> {
    let name = form.get_field("Name")?.value();
    let description = form.get_field("Description")?.value();
    let network = NewsNetwork {
        description,
        id,
        name,
    };
    Ok(network)
}

impl From<&NewsAffiliate> for Form {
    fn from(model: &NewsAffiliate) -> Self {
        let mut form = Form::new();
        form.add_field(Box::new(TextField::new("Name", &model.name)));
        form.add_field(Box::new(MultilineTextField::new(
            "Description",
            &model.description,
        )));
        form.add_field(Box::new(TextField::new("Region", &model.region)));
        form.add_field(Box::new(TextField::new("Network", &model.network.name)));
        form
    }
}

pub fn news_affiliate_from_form(
    id: i32,
    form: &Form,
    networks: &[NewsNetwork],
) -> Result<NewsAffiliate> {
    let name = form.get_field("Name")?.value();
    let description = form.get_field("Description")?.value();
    let region = form.get_field("Region")?.value();
    let network_name = form.get_field("Network")?.value();
    let network = networks
        .iter()
        .find(|m| m.name == network_name)
        .ok_or_else(|| eyre!("{network_name} is not in the networks list"))?;

    let affiliate = NewsAffiliate {
        description,
        id,
        name,
        region,
        network: network.clone(),
    };
    Ok(affiliate)
}
