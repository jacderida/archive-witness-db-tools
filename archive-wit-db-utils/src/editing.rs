use archive_wit_db::helpers::{duration_to_string, interval_to_duration, parse_duration};
use archive_wit_db::models::{
    Category, EventTimestamp, MasterVideo, NewsBroadcast, Person, PersonType, Video,
};
use color_eyre::{eyre::eyre, Result};
use sqlx::postgres::types::PgInterval;
use std::path::PathBuf;

pub fn build_master_video_editor_template(
    master_video: &MasterVideo,
    news_broadcasts: &Vec<NewsBroadcast>,
) -> String {
    let mut template = String::new();

    template.push_str("News Broadcasts:");
    if !master_video.news_broadcasts.is_empty() {
        template.push_str(" ");
        template.push_str(
            &master_video
                .news_broadcasts
                .iter()
                .map(|b| {
                    if let Some(network) = &b.news_network {
                        if let Some(date) = b.date {
                            format!("{} ({})", network.name, date.to_string())
                        } else {
                            network.name.clone()
                        }
                    } else if let Some(affiliate) = &b.news_affiliate {
                        if let Some(date) = b.date {
                            format!("{} ({})", affiliate.name, date.to_string())
                        } else {
                            affiliate.name.clone()
                        }
                    } else {
                        panic!("A broadcast must have a network or an affiliate");
                    }
                })
                .collect::<Vec<String>>()
                .join(";"),
        );
        template.push_str("\n");
    } else if master_video.id == 0 {
        template.push_str("\n## CHOOSE ONE OR DELETE ALL ##\n");
        for broadcast in news_broadcasts.iter() {
            template.push_str(&broadcast.to_string());
            template.push_str("\n");
        }
    }
    template.push_str("---\n");

    template.push_str("Title:");
    if !master_video.title.is_empty() {
        template.push_str(" ");
        template.push_str(&master_video.title);
    }
    template.push_str("\n---\n");

    template.push_str("Categories:");
    if !master_video.categories.is_empty() {
        template.push_str(" ");
        template.push_str(
            &master_video
                .categories
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(";"),
        );
    }
    template.push_str("\n---\n");

    template.push_str("Date:");
    if let Some(date) = master_video.date {
        template.push_str(" ");
        template.push_str(&date.to_string());
    }
    template.push_str("\n---\n");

    template.push_str("Description:");
    if let Some(desc) = &master_video.description {
        template.push_str("\n");
        template.push_str(desc);
    }
    template.push_str("\n---\n");

    template.push_str("Links:");
    if !master_video.links.is_empty() {
        template.push_str(" ");
        template.push_str(&master_video.links.join(";"));
    };
    template.push_str("\n---\n");

    template.push_str("Timestamps:");
    if !master_video.timestamps.is_empty() {
        template.push_str("\n");
        template.push_str(
            &master_video
                .timestamps
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
        );
    };
    template.push_str("\n---\n");

    template.push_str("NIST Notes:");
    if let Some(nist_nodes) = &master_video.nist_notes {
        template.push_str(" ");
        template.push_str(&nist_nodes);
    }
    template.push_str("\n---\n");

    template.push_str(&master_video.people_as_string("Eyewitnesses", PersonType::Eyewitness));
    template.push_str("\n---\n");
    template.push_str(&master_video.people_as_string("Fire", PersonType::Fire));
    template.push_str("\n---\n");
    template.push_str(&master_video.people_as_string("Police", PersonType::Police));
    template.push_str("\n---\n");
    template.push_str(&master_video.people_as_string("Port Authority", PersonType::PortAuthority));
    template.push_str("\n---\n");
    template.push_str(&master_video.people_as_string("Reporters", PersonType::Reporter));
    template.push_str("\n---\n");
    template.push_str(&master_video.people_as_string("Survivors", PersonType::Survivor));
    template.push_str("\n---\n");
    template.push_str(&master_video.people_as_string("Victims", PersonType::Victim));
    template.push_str("\n---\n");
    template.push_str(&master_video.people_as_string("Videographers", PersonType::Videographer));
    template.push_str("\n---\n");

    template.push_str("NIST Files:");
    if !master_video.nist_files.is_empty() {
        template.push_str("\n");
        for (path, _) in master_video.nist_files.iter() {
            template.push_str(&format!("{}\n", path.to_string_lossy()));
        }
    }

    template
}

pub fn build_video_editor_template(video: &Video, masters: &Vec<MasterVideo>) -> String {
    let mut template = String::new();

    template.push_str("Master:");
    if video.id != 0 {
        template.push_str(" ");
        template.push_str(&video.master.title);
        template.push_str("\n");
    } else {
        template.push_str("\n## CHOOSE ONE OR DELETE ALL ##\n");
        for master in masters.iter() {
            template.push_str(&master.title);
            template.push_str("\n");
        }
    }
    template.push_str("---\n");

    template.push_str("Title:");
    if video.id != 0 {
        template.push_str(" ");
        template.push_str(&video.master.title);
        template.push_str("\n");
    } else {
        template.push_str("\n");
    }
    template.push_str("---\n");

    template.push_str("Description:");
    if let Some(desc) = &video.description {
        template.push_str("\n");
        template.push_str(&desc);
        template.push_str("\n");
    } else {
        template.push_str("\n");
    }
    template.push_str("---\n");

    template.push_str("Link:");
    if video.id != 0 {
        template.push_str(" ");
        template.push_str(&video.link);
        template.push_str("\n");
    } else {
        template.push_str("\n");
    }
    template.push_str("---\n");

    template.push_str("Duration:");
    if video.id != 0 {
        template.push_str(" ");
        template.push_str(&duration_to_string(&interval_to_duration(&video.duration)));
        template.push_str("\n");
    } else {
        template.push_str("\n");
    }
    template.push_str("---\n");

    template.push_str("Primary:");
    if video.id != 0 {
        template.push_str(" ");
        if video.is_primary {
            template.push_str("Yes");
        } else {
            template.push_str("No");
        }
        template.push_str("\n");
    } else {
        template.push_str(" ");
        template.push_str("No");
    }

    template
}

pub fn parse_master_video_editor_template(
    id: i32,
    edited_template: &str,
    news_broadcasts: &Vec<NewsBroadcast>,
    people: &Vec<Person>,
) -> Result<MasterVideo> {
    let parts: Vec<_> = edited_template.split("---\n").collect();
    if parts.len() != 17 {
        return Err(eyre!("Edited template was not in expected format"));
    }

    let mut video_news_broadcasts = Vec::new();
    let news_broadcasts_input = parts[0]
        .trim_start_matches("News Broadcasts: ")
        .trim()
        .to_string();
    if !news_broadcasts_input.is_empty() {
        for broadcast_input in news_broadcasts_input.split(';').map(|v| v.trim()) {
            if let Some(broadcast) = news_broadcasts
                .iter()
                .find(|b| b.to_string() == broadcast_input)
            {
                video_news_broadcasts.push(broadcast.clone());
            } else {
                return Err(eyre!("{} is not a valid broadcast", broadcast_input));
            }
        }
    }

    let title = parts[1].trim_start_matches("Title: ").trim().to_string();

    let mut categories = Vec::new();
    let categories_input = parts[2].trim_start_matches("Categories: ").trim();
    if !categories_input.is_empty() {
        for category in categories_input.split(';').map(|v| v.trim()) {
            categories.push(Category::from(category));
        }
    }

    let date = parts[3].trim_start_matches("Date: ").trim();

    let description = parts[4]
        .trim_start_matches("Description:")
        .trim_start_matches(':')
        .trim()
        .to_string();
    let description = if description.is_empty() {
        None
    } else {
        Some(description)
    };

    // I don't quite understand why this field needs to be treated differently from the others, but
    // the same logic does not have the intended effect and does not correctly support an empty
    // field.
    let links_input = parts[5].strip_prefix("Links: ").unwrap_or("").trim();
    let links = if links_input.is_empty() {
        Vec::new()
    } else {
        links_input
            .trim_matches(':')
            .split(';')
            .map(|l| l.trim().to_string())
            .collect::<Vec<String>>()
    };

    let timestamps_input = parts[6]
        .trim_start_matches("Timestamps:")
        .trim()
        .trim_start_matches(':');
    let timestamps = if timestamps_input.is_empty() {
        Vec::new()
    } else {
        timestamps_input
            .split('\n')
            .map(|t| EventTimestamp::try_from(t).unwrap())
            .collect::<Vec<EventTimestamp>>()
    };

    let nist_notes = parts[7].strip_prefix("NIST Notes: ").unwrap_or("").trim();
    let nist_notes = if nist_notes.is_empty() {
        None
    } else {
        Some(nist_notes.to_string())
    };

    let pairs = vec![
        ("Eyewitnesses: ", PersonType::Eyewitness),
        ("Fire: ", PersonType::Fire),
        ("Police: ", PersonType::Police),
        ("Port Authority: ", PersonType::PortAuthority),
        ("Reporters: ", PersonType::Reporter),
        ("Survivors: ", PersonType::Survivor),
        ("Victims: ", PersonType::Victim),
        ("Videographers: ", PersonType::Videographer),
    ];

    let mut video_people: Vec<Person> = Vec::new();
    let mut i = 8;
    for (prefix, person_type) in pairs.iter() {
        let people_input = get_people_from_input(&parts[i], prefix, &people, person_type.clone());
        for person in people_input.iter() {
            if video_people
                .iter()
                .find(|p| p.name == person.name)
                .is_none()
            {
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
        i += 1;
    }

    let mut nist_files = Vec::new();
    let nist_files_input = parts[16]
        .trim_start_matches("NIST Files:")
        .trim_start_matches(':')
        .trim();
    if !nist_files_input.is_empty() {
        for path in nist_files_input.split('\n').map(|p| PathBuf::from(p)) {
            nist_files.push((path, 0));
        }
    }

    let master = MasterVideo {
        categories,
        date: Some(date.parse()?),
        description,
        id,
        links,
        news_broadcasts: video_news_broadcasts,
        nist_files,
        nist_notes,
        people: video_people,
        timestamps,
        title,
    };
    Ok(master)
}

pub fn parse_video_editor_template(
    id: i32,
    edited_template: &str,
    masters: &Vec<MasterVideo>,
) -> Result<Video> {
    let parts: Vec<_> = edited_template.split("---\n").collect();
    if parts.len() != 6 {
        return Err(eyre!("Edited template was not in expected format"));
    }

    let master_title = parts[0].trim_start_matches("Master: ").trim().to_string();
    let master = masters
        .iter()
        .find(|m| m.title == master_title)
        .ok_or_else(|| eyre!("{master_title} is not in the master list"))?;
    let title = parts[1].trim_start_matches("Title: ").trim().to_string();

    let description = parts[2]
        .trim_start_matches("Description:")
        .trim_start_matches(':')
        .trim()
        .to_string();
    let description = if description.is_empty() {
        None
    } else {
        Some(description)
    };

    let link = parts[3].trim_start_matches("Link: ").trim().to_string();
    let duration = parts[4].trim_start_matches("Duration: ").trim().to_string();
    let duration = PgInterval::try_from(parse_duration(&duration))
        .map_err(|_| eyre!("Could not convert duration string"))?;

    let is_primary = parts[5].trim_start_matches("Primary: ").trim().to_string();
    let is_primary = if is_primary.to_lowercase() == "yes" {
        true
    } else {
        false
    };

    let video = Video {
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

fn get_people_from_input(
    input: &str,
    prefix: &str,
    people: &[Person],
    person_type: PersonType,
) -> Vec<Person> {
    let mut result = Vec::new();
    let trimmed_input = input.strip_prefix(prefix).unwrap_or("").trim().to_string();
    if !trimmed_input.is_empty() {
        for name in trimmed_input.split(';').map(|v| v.trim()) {
            let (id, description, historical_title) =
                if let Some(person) = people.iter().find(|p| p.name == name) {
                    (
                        person.id,
                        person.description.clone(),
                        person.historical_title.clone(),
                    )
                } else {
                    (0, None, None)
                };
            result.push(Person {
                id,
                name: name.to_string(),
                description,
                historical_title,
                types: vec![person_type.clone()],
            });
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use archive_wit_db::models::{Category, NewsAffiliate, NewsNetwork};
    use chrono::NaiveDate;

    #[test]
    fn build_master_video_editor_template_from_default() {
        let expected_template = std::fs::read_to_string("../resources/master_template_empty")
            .expect("Failed to read expected template file");

        let news_broadcasts = vec![
            NewsBroadcast {
                id: 1,
                date: Some(NaiveDate::from_ymd_opt(2001, 9, 11).unwrap()),
                description: None,
                news_network: Some(NewsNetwork {
                    id: 1,
                    name: String::from("WABC-TV"),
                    description: String::new(),
                }),
                news_affiliate: None,
            },
            NewsBroadcast {
                id: 2,
                date: Some(NaiveDate::from_ymd_opt(2001, 9, 11).unwrap()),
                description: None,
                news_network: Some(NewsNetwork {
                    id: 2,
                    name: String::from("ABC News"),
                    description: String::new(),
                }),
                news_affiliate: None,
            },
        ];

        let master_video = MasterVideo::default();
        let generated_template =
            build_master_video_editor_template(&master_video, &news_broadcasts);
        assert_eq!(generated_template.trim(), expected_template.trim());
    }

    #[test]
    fn parse_master_video_from_populated_template() {
        let populated_template = std::fs::read_to_string("../resources/master_template_completed")
            .expect("Failed to read template file");
        let news_network = NewsNetwork {
            id: 1,
            name: "ABC News".to_string(),
            description: "National ABC News coverage".to_string(),
        };

        let news_broadcasts = vec![
            NewsBroadcast {
                id: 1,
                date: Some(NaiveDate::from_ymd_opt(2001, 9, 11).unwrap()),
                description: None,
                news_network: None,
                news_affiliate: Some(NewsAffiliate {
                    id: 1,
                    name: String::from("WABC-TV"),
                    description: String::new(),
                    region: "NYC".to_string(),
                    network: news_network.clone(),
                }),
            },
            NewsBroadcast {
                id: 2,
                date: Some(NaiveDate::from_ymd_opt(2001, 9, 11).unwrap()),
                description: None,
                news_network: Some(news_network),
                news_affiliate: None,
            },
        ];

        let people = vec![
            Person {
                description: None,
                historical_title: None,
                id: 1,
                name: "Steve Bartelstein".to_string(),
                types: vec![PersonType::Reporter],
            },
            Person {
                description: None,
                historical_title: None,
                id: 2,
                name: "Lori Stokes".to_string(),
                types: vec![PersonType::Reporter],
            },
            Person {
                description: None,
                historical_title: None,
                id: 3,
                name: "Steve Silva".to_string(),
                types: vec![PersonType::Eyewitness],
            },
            Person {
                description: None,
                historical_title: None,
                id: 4,
                name: "Sandra Rodriguez".to_string(),
                types: vec![PersonType::Eyewitness],
            },
        ];

        let master_video =
            parse_master_video_editor_template(0, &populated_template, &news_broadcasts, &people)
                .unwrap();
        assert_eq!(master_video.news_broadcasts[0].id, 1);
        assert_eq!(
            master_video.title,
            "WABC-TV/ABC7: 9/11 Broadcast [0842 – 1042]"
        );
        assert_eq!(master_video.categories, vec![Category::News]);
        assert_eq!(
            master_video.description,
            Some("The source tape is about 2 hours of continuous footage, from approximately 0842 to 1042.\n\
                The picture has grainy artifacts and the audio is low and slightly muffled.".to_string())
        );
        assert_eq!(
            master_video.links,
            vec!["https://www.google.com".to_string()]
        );
        assert_eq!(master_video.timestamps.len(), 16);
        assert_eq!(
            master_video.nist_notes,
            Some("Silverstein copy has about 8 minutes more than NCM copy - not recorded on mini-DV, but contains a couple more replays of collapses".to_string())
        );

        let existing_person = master_video
            .people
            .iter()
            .find(|p| p.name == "Steve Bartelstein")
            .unwrap();
        assert_eq!(existing_person.id, 1);
        let new_person = master_video
            .people
            .iter()
            .find(|p| p.name == "Jay Jonas")
            .unwrap();
        assert_eq!(new_person.id, 0);
        let person_with_multiple_types = master_video
            .people
            .iter()
            .find(|p| p.name == "John DelGiorno")
            .unwrap();
        assert!(person_with_multiple_types
            .types
            .contains(&PersonType::Videographer));
        assert!(person_with_multiple_types
            .types
            .contains(&PersonType::Reporter));

        assert_eq!(
            master_video
                .people
                .iter()
                .filter(|p| p.types.contains(&PersonType::Eyewitness))
                .count(),
            2
        );
        assert_eq!(
            master_video
                .people
                .iter()
                .filter(|p| p.types.contains(&PersonType::Fire))
                .count(),
            1
        );
        assert_eq!(
            master_video
                .people
                .iter()
                .filter(|p| p.types.contains(&PersonType::Police))
                .count(),
            1
        );
        assert_eq!(
            master_video
                .people
                .iter()
                .filter(|p| p.types.contains(&PersonType::PortAuthority))
                .count(),
            1
        );
        assert_eq!(
            master_video
                .people
                .iter()
                .filter(|p| p.types.contains(&PersonType::Reporter))
                .count(),
            3
        );
        assert_eq!(
            master_video
                .people
                .iter()
                .filter(|p| p.types.contains(&PersonType::Victim))
                .count(),
            1
        );
        assert_eq!(
            master_video
                .people
                .iter()
                .filter(|p| p.types.contains(&PersonType::Videographer))
                .count(),
            1
        );
        assert_eq!(master_video.nist_files.len(), 10);
    }

    #[test]
    fn parse_master_video_where_an_existing_person_has_a_type_added() {
        let populated_template = std::fs::read_to_string("../resources/master_template_completed")
            .expect("Failed to read template file");
        let news_network = NewsNetwork {
            id: 1,
            name: "ABC News".to_string(),
            description: "National ABC News coverage".to_string(),
        };

        let news_broadcasts = vec![
            NewsBroadcast {
                id: 1,
                date: Some(NaiveDate::from_ymd_opt(2001, 9, 11).unwrap()),
                description: None,
                news_network: None,
                news_affiliate: Some(NewsAffiliate {
                    id: 1,
                    name: String::from("WABC-TV"),
                    description: String::new(),
                    region: "NYC".to_string(),
                    network: news_network.clone(),
                }),
            },
            NewsBroadcast {
                id: 2,
                date: Some(NaiveDate::from_ymd_opt(2001, 9, 11).unwrap()),
                description: None,
                news_network: Some(news_network),
                news_affiliate: None,
            },
        ];

        let people = vec![
            Person {
                description: None,
                historical_title: None,
                id: 1,
                name: "John DelGiorno".to_string(),
                types: vec![PersonType::Reporter],
            },
            Person {
                description: None,
                historical_title: None,
                id: 2,
                name: "Lori Stokes".to_string(),
                types: vec![PersonType::Reporter],
            },
            Person {
                description: None,
                historical_title: None,
                id: 3,
                name: "Steve Silva".to_string(),
                types: vec![PersonType::Eyewitness],
            },
            Person {
                description: None,
                historical_title: None,
                id: 4,
                name: "Sandra Rodriguez".to_string(),
                types: vec![PersonType::Eyewitness],
            },
        ];

        let master_video =
            parse_master_video_editor_template(0, &populated_template, &news_broadcasts, &people)
                .unwrap();
        let person_with_multiple_types = master_video
            .people
            .iter()
            .find(|p| p.name == "John DelGiorno")
            .unwrap();
        assert!(person_with_multiple_types
            .types
            .contains(&PersonType::Videographer));
        assert!(person_with_multiple_types
            .types
            .contains(&PersonType::Reporter));

        assert_eq!(master_video.nist_files.len(), 10);
    }

    #[test]
    fn parse_master_video_where_template_has_empty_fields() {
        let populated_template =
            std::fs::read_to_string("../resources/master_template_with_empty_fields")
                .expect("Failed to read template file");

        let news_network = NewsNetwork {
            id: 1,
            name: "ABC News".to_string(),
            description: "National ABC News coverage".to_string(),
        };

        let news_broadcasts = vec![
            NewsBroadcast {
                id: 1,
                date: Some(NaiveDate::from_ymd_opt(2001, 9, 11).unwrap()),
                description: None,
                news_network: None,
                news_affiliate: Some(NewsAffiliate {
                    id: 1,
                    name: String::from("WABC-TV"),
                    description: String::new(),
                    region: "NYC".to_string(),
                    network: news_network.clone(),
                }),
            },
            NewsBroadcast {
                id: 2,
                date: Some(NaiveDate::from_ymd_opt(2001, 9, 11).unwrap()),
                description: None,
                news_network: Some(news_network),
                news_affiliate: None,
            },
        ];

        let people = vec![
            Person {
                description: None,
                historical_title: None,
                id: 1,
                name: "John DelGiorno".to_string(),
                types: vec![PersonType::Reporter],
            },
            Person {
                description: None,
                historical_title: None,
                id: 2,
                name: "Lori Stokes".to_string(),
                types: vec![PersonType::Reporter],
            },
            Person {
                description: None,
                historical_title: None,
                id: 3,
                name: "Steve Silva".to_string(),
                types: vec![PersonType::Eyewitness],
            },
            Person {
                description: None,
                historical_title: None,
                id: 4,
                name: "Sandra Rodriguez".to_string(),
                types: vec![PersonType::Eyewitness],
            },
        ];

        let master_video =
            parse_master_video_editor_template(0, &populated_template, &news_broadcasts, &people)
                .unwrap();

        assert!(master_video.description.is_none());
        assert!(master_video.links.is_empty());
        assert!(master_video.timestamps.is_empty());
        assert!(master_video.nist_notes.is_none());
        assert!(master_video.people.is_empty());
        assert!(master_video.nist_files.is_empty());
    }

    #[test]
    fn build_video_editor_template_from_default() {
        let expected_template = std::fs::read_to_string("../resources/video_template_empty")
            .expect("Failed to read expected template file");
        let populated_template = std::fs::read_to_string("../resources/master_template_completed")
            .expect("Failed to read template file");

        let news_network = NewsNetwork {
            id: 1,
            name: "ABC News".to_string(),
            description: "National ABC News coverage".to_string(),
        };

        let news_broadcasts = vec![
            NewsBroadcast {
                id: 1,
                date: Some(NaiveDate::from_ymd_opt(2001, 9, 11).unwrap()),
                description: None,
                news_network: None,
                news_affiliate: Some(NewsAffiliate {
                    id: 1,
                    name: String::from("WABC-TV"),
                    description: String::new(),
                    region: "NYC".to_string(),
                    network: news_network.clone(),
                }),
            },
            NewsBroadcast {
                id: 2,
                date: Some(NaiveDate::from_ymd_opt(2001, 9, 11).unwrap()),
                description: None,
                news_network: Some(news_network),
                news_affiliate: None,
            },
        ];

        let people = vec![
            Person {
                description: None,
                historical_title: None,
                id: 1,
                name: "Steve Bartelstein".to_string(),
                types: vec![PersonType::Reporter],
            },
            Person {
                description: None,
                historical_title: None,
                id: 2,
                name: "Lori Stokes".to_string(),
                types: vec![PersonType::Reporter],
            },
            Person {
                description: None,
                historical_title: None,
                id: 3,
                name: "Steve Silva".to_string(),
                types: vec![PersonType::Eyewitness],
            },
            Person {
                description: None,
                historical_title: None,
                id: 4,
                name: "Sandra Rodriguez".to_string(),
                types: vec![PersonType::Eyewitness],
            },
        ];

        let master_video =
            parse_master_video_editor_template(0, &populated_template, &news_broadcasts, &people)
                .unwrap();

        let video = Video::default();
        let generated_template = build_video_editor_template(&video, &vec![master_video]);

        assert_eq!(generated_template.trim(), expected_template.trim());
    }
}
