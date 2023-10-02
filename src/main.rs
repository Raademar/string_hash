mod test;

use regex::Regex;
use sha256::digest;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, prelude::*};
use std::path::PathBuf;

// Replace with current dir when running
const CLIENT_PATH: &'static str = "C:/projects/kcrp_string_hash/test_data/client_index.js";
const SERVER_PATH: &'static str = "C:/projects/kcrp_string_hash/test_data/server_index.js";
const CEF_PATH: &'static str = "C:/projects/kcrp_string_hash/test_data/cef_index.js";

struct EventHash {
    event: String,
    hash: String,
}

impl EventHash {
    fn new(event: &str, hay: &HashMap<String, String>) -> Self {
        Self {
            event: event.to_string(),
            hash: get_value_or_digest_sha256(event, hay),
        }
    }
}

fn main() {
    let client_content = fs::read_to_string(CLIENT_PATH).expect("Unable to read the file");
    let server_content = fs::read_to_string(SERVER_PATH).expect("Unable to read the file");
    let cef_content = fs::read_to_string(CEF_PATH).expect("Unable to read the file");

    let re = Regex::new(r#"("server:[a-zA-Z-0-9:-_]*\")|(\"client:[a-zA-Z-0-9:-_]*\")"#).unwrap();

    let mut event_hashes: HashMap<String, String> = HashMap::new();

    fill_hash_map(
        client_content,
        server_content,
        cef_content,
        &mut event_hashes,
        &re,
    );

    let paths: Vec<PathBuf> = vec![
        PathBuf::from(CLIENT_PATH),
        PathBuf::from(SERVER_PATH),
        PathBuf::from(CEF_PATH),
    ];
    for path in paths {
        let _ = replace_event_names_in_files_with_hashes(&event_hashes, path.as_os_str());
    }
}

fn replace_event_names_in_files_with_hashes(
    event_hashes: &HashMap<String, String>,
    file_path: &OsStr,
) -> Result<(), io::Error> {
    let mut file = File::open(file_path)?;

    let mut data = String::new();
    file.read_to_string(&mut data)?;
    drop(file);

    let mut new_data = data.clone();
    let mut destination = File::create(&file_path)?;

    for (key, value) in event_hashes.iter() {
        new_data = new_data.replace(key, value);
    }
    destination.write(new_data.as_bytes())?;

    Ok(())
}

fn fill_hash_map<'a>(
    client_content: String,
    server_content: String,
    cef_content: String,
    hash_map: &mut HashMap<String, String>,
    re: &'a Regex,
) {
    let client_matches = return_matches_from_lines(&client_content, &re);
    let server_matches = return_matches_from_lines(&server_content, &re);
    let cef_matches = return_matches_from_lines(&cef_content, &re);

    read_matches_and_insert_to_hash_map([client_matches, server_matches, cef_matches], hash_map);
}

fn return_matches_from_lines<'a>(lines: &'a str, re: &Regex) -> Vec<&'a str> {
    let matches: Vec<&'a str> = re.find_iter(&lines).map(|m| m.as_str()).collect();
    matches
}

fn read_matches_and_insert_to_hash_map(
    matches: [Vec<&str>; 3],
    event_hashes: &mut HashMap<String, String>,
) {
    for m in matches {
        for line in m {
            let parsed_event: String = line.chars().filter(|&c| c != '"').collect();
            let event_hash = EventHash::new(&parsed_event, &event_hashes);

            event_hashes.insert(event_hash.event, event_hash.hash);
        }
    }
}

fn get_value_or_digest_sha256(event: &str, hay: &HashMap<String, String>) -> String {
    match hay.get(event) {
        None => digest(event),
        Some(e) => e.to_string(),
    }
}
