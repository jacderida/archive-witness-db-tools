use super::*;
use crate::editing::{forms::Form, masters::master_video_from_form};
use chrono::NaiveDate;
use db::models::{Category, MasterVideo, NewsAffiliate, NewsBroadcast, NewsNetwork, Video};

#[test]
fn form_as_string_from_default_master_video_should_have_expected_fields() {
    let expected_form = std::fs::read_to_string("../resources/master_form_empty")
        .expect("Failed to read test form");

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
    let mut form = Form::from(&master_video);
    form.add_choices(
        "News Broadcasts",
        news_broadcasts.iter().map(|b| b.to_string()).collect(),
    )
    .unwrap();
    assert_eq!(form.as_string(), expected_form.trim());
}

#[test]
fn form_from_str_should_parse_a_form_from_a_string() {
    let form_input = std::fs::read_to_string("../resources/master_form_completed")
        .expect("Failed to read test form");

    let form = Form::from_master_video_str(&form_input).unwrap();
    assert_eq!(form.as_string(), form_input.trim());
}

#[test]
fn master_video_from_form_should_parse_a_master_video() {
    let form_input = std::fs::read_to_string("../resources/master_form_completed")
        .expect("Failed to read test form");
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

    let form = Form::from_master_video_str(&form_input).unwrap();
    let master_video = master_video_from_form(0, &form, &news_broadcasts, &people).unwrap();
    assert_eq!(master_video.news_broadcasts[0].id, 1);
    assert_eq!(
        master_video.title,
        "WABC-TV/ABC7: 9/11 Broadcast [0842 – 1042]"
    );
    assert_eq!(master_video.categories, vec![Category::News]);
    assert_eq!(
            master_video.description,
            "The source tape is about 2 hours of continuous footage, from approximately 0842 to 1042.\n\
                The picture has grainy artifacts and the audio is low and slightly muffled.".to_string()
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
fn master_video_from_form_should_parse_a_form_where_an_existing_person_has_a_type_added() {
    let form_input = std::fs::read_to_string("../resources/master_form_completed")
        .expect("Failed to read test form");
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

    let form = Form::from_master_video_str(&form_input).unwrap();
    let master_video = master_video_from_form(0, &form, &news_broadcasts, &people).unwrap();
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
fn master_video_from_form_should_parse_master_video_where_form_has_empty_fields() {
    let form_input = std::fs::read_to_string("../resources/master_form_with_empty_fields")
        .expect("Failed to read test form");

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

    let form = Form::from_master_video_str(&form_input).unwrap();
    let master_video = master_video_from_form(0, &form, &news_broadcasts, &people).unwrap();

    assert!(master_video.links.is_empty());
    assert!(master_video.timestamps.is_empty());
    assert!(master_video.nist_notes.is_none());
    assert!(master_video.people.is_empty());
    assert!(master_video.nist_files.is_empty());
}

#[test]
fn form_as_string_from_default_video_should_have_expected_fields() {
    let video_form =
        std::fs::read_to_string("../resources/video_form_empty").expect("Failed to read test form");
    let master_video_form = std::fs::read_to_string("../resources/master_form_completed")
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

    let master_form = Form::from_master_video_str(&master_video_form).unwrap();
    let master_video = master_video_from_form(0, &master_form, &news_broadcasts, &people).unwrap();

    let mut form = Form::from(&Video::default());
    form.add_choices("Master", vec![master_video.title])
        .unwrap();
    assert_eq!(form.as_string(), video_form.trim());
}
