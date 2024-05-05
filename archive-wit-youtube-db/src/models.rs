use crate::error::Result;

use std::path::PathBuf;

#[derive(Clone)]
pub struct YouTubeVideo {
    pub channel_name: String,
    pub description: Option<String>,
    pub duration: Option<String>,
    pub id: String,
    pub saved_path: Option<PathBuf>,
    pub title: String,
}

impl YouTubeVideo {
    pub fn new(
        channel_name: &str,
        duration: Option<String>,
        id: &str,
        saved_path: Option<PathBuf>,
        title: &str,
    ) -> Result<Self> {
        let description = if let Some(mut path) = saved_path.clone() {
            path.pop();
            path.pop(); // Now at the channel level
            path.push("description");
            path.push(format!("{id}.description"));
            if path.exists() {
                let mut desc = std::fs::read_to_string(path)?;
                desc = desc.trim().to_string();
                if desc.is_empty() {
                    None
                } else {
                    Some(desc)
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(YouTubeVideo {
            channel_name: channel_name.to_string(),
            description,
            duration,
            id: id.to_string(),
            saved_path,
            title: title.to_string(),
        })
    }

    pub fn print(&self) {
        println!("ID: {}", self.id);
        println!("Title: {}", self.title);
        println!(
            "Description: {}",
            self.description.as_ref().unwrap_or(&"-".to_string())
        );
    }
}
