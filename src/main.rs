use std::fs;
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Write;
use std::env;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
struct Export {
    entries: Vec<JsonEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct JsonEntry {
    text: String,
    #[serde(rename(deserialize = "creationDate", serialize = "date"))]
    date: String,
    weather: Option<Weather>,
    location: Option<Location>,
    photos: Option<Vec<Photo>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Weather {
    weather_code: String,
    temperature_celsius: f64
}

impl Weather {
    fn display_string(&self) -> String {
        return format!("{} °C", self.temperature_celsius as i64)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Location {
    locality_name: Option<String>,
    country: Option<String>,
    longitude: f64,
    latitude: f64,
    place_name: Option<String>,
    administrativ_area: Option<String>
}

impl Location {
    fn display_string(&self) -> String {
        let mut values = Vec::new();

        if let Some(country) = &self.country {
            values.push(country.to_string());
        }

        if let Some(locality_name) = &self.locality_name {
            values.push(locality_name.to_string());
        }

        if let Some(place_name) = &self.place_name {
            values.push(place_name.to_string());
        }

        return values.join(", ").to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Photo {
    identifier: String,
    #[serde(rename = "type")]
    kind: String,
    md5: String
}

fn convert_entry(entry: &JsonEntry, content_folder: String, journal_folder: String) {
    let parsed_date = chrono::DateTime::parse_from_rfc3339(&entry.date.to_string()).expect("Date could not be converted");
    let date_string = parsed_date.format("%Y-%m-%dT%H:%M:%SZ");
    let post_folder = format!("{}/{}", content_folder, date_string.to_string());

    let _ = fs::create_dir_all(post_folder.to_string());

    let mut text = entry.text.to_string();


    if let Some(photos) = &entry.photos {
        for photo in photos {
            let day_one_moment = format!("dayone-moment://{}", photo.identifier);
            let file_name = &format!("{}.{}", photo.md5, photo.kind);

            fs::copy(
                format!("{}/photos/{}", journal_folder, file_name), 
                format!("{}/{}", post_folder, file_name)
            ).expect("Photo couldn't be moved");

            text = text.replace(&day_one_moment, file_name);
        }
    }

    let location_string;

    if let Some(location) = &entry.location {
        location_string = location.display_string();
    } else {
        location_string = "".to_string();
    }

    let weather_string;

    if let Some(weather) = &entry.weather {
        weather_string = weather.display_string()
    } else {
        weather_string = "".to_string();
    }

    let post_content = format!(
    r#"
+++
date = {}
+++
Location: {} - Weather: {}
{}

<!-- more -->
    "#
    , date_string.to_string(), location_string, weather_string, text.to_string());

    let mut f = File::create(format!("{}/index.md", post_folder.to_string())).expect("Unable to create file");
    f.write_all(post_content.as_bytes()).expect("Unable to write data");    
}

fn create_content_folder(export_path: String) -> String {
    let content_folder = format!("{}/content", export_path);
    fs::create_dir(content_folder.to_string()).expect("Folder couldnt be created");

    content_folder
}

fn convert_to_zola(
    export: Export, 
    journal_folder: String, 
    export_path: String,
) {
    let content_folder_path = create_content_folder(export_path.to_string());

    for entry in export.entries.iter() {
        convert_entry(entry, content_folder_path.to_string(), journal_folder.to_string());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let path_parameter = &args[1];

    let journal_path = Path::new(path_parameter);
    let journal_path_string = journal_path.to_str().expect("Journal path invalid").to_string();

    let journal_folder = journal_path.parent().expect("Paramter invalid");
    let jorunal_folder_string = journal_folder.to_str().expect("Journal folder invalid").to_string();

    println!("{}", journal_path_string);

    let data = fs::read_to_string(&journal_path_string).unwrap();
    let export: Export = serde_json::from_str(&data).expect("JSON couldnt be parsed");

    convert_to_zola(export.clone(), jorunal_folder_string.to_string(), jorunal_folder_string.to_string());

    println!("{:?}", export.clone());
}
