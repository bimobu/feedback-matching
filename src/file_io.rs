use crate::structs::matching_round::MatchingRound;
use crate::structs::participants_file::ParticipantsFile;

use jsonschema::JSONSchema;
use serde::de::DeserializeOwned;
use serde_json::{from_reader, Value};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};

const MATCHES_SCHEMA: &[u8] = include_bytes!("../data/schema/matches_schema.json");
const PARTICIPANTS_SCHEMA: &[u8] = include_bytes!("../data/schema/participants_schema.json");

pub fn read_participants(file_path: &str) -> ParticipantsFile {
    read::<ParticipantsFile>(file_path, PARTICIPANTS_SCHEMA)
}

pub fn read_matching_rounds(file_path: &str) -> Vec<MatchingRound> {
    read::<Vec<MatchingRound>>(file_path, MATCHES_SCHEMA)
}

fn read<T>(file_path: &str, schema: &[u8]) -> T
where
    T: DeserializeOwned,
{
    let compiled_schema = load_schema(schema);

    let data_string = read_string(file_path);
    let json_data: Value =
        serde_json::from_str(data_string.as_str()).expect("Failed to parse JSON data");

    let validation_result = compiled_schema.validate(&json_data);

    if let Err(errors) = validation_result {
        println!("\n### Read JSON Validation Errors ###\n");

        for (i, error) in errors.enumerate() {
            println!(
                "{}.Validation error: {}\nInstance path: {}\n",
                i + 1,
                error,
                error.instance_path
            );
        }

        print!("###\n\n")
    }

    from_reader(data_string.as_bytes()).expect("Failed to parse JSON")
}

fn read_string(file_path: &str) -> String {
    let mut file = File::open(file_path).expect("Failed to open JSON file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read JSON file");

    contents
}

fn load_schema(schema: &[u8]) -> JSONSchema {
    let schema: Value = serde_json::from_slice(schema).expect("Failed to parse Schema");
    JSONSchema::compile(&schema).expect("Failed to compile Schema")
}

pub fn save_matching_round(file_path: &str, round: MatchingRound) {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)
        .expect("Failed to open matches file");

    let mut existing_rounds: Vec<MatchingRound> = from_reader(&file).unwrap_or_else(|_| Vec::new());

    existing_rounds.push(round);

    update_all_existing_rounds(file_path, &existing_rounds)
}

pub fn update_all_existing_rounds(file_path: &str, existing_rounds: &Vec<MatchingRound>) {
    let schema = load_schema(MATCHES_SCHEMA);

    let value = serde_json::to_value(&existing_rounds).expect("Failed to serialize MatchingRound");
    let validation_result = schema.validate(&value);

    if let Err(errors) = validation_result {
        println!("\n### Write JSON Validation Errors ###\n");

        for (i, error) in errors.enumerate() {
            println!(
                "{}.Validation error: {}\nInstance path: {}\n",
                i + 1,
                error,
                error.instance_path
            );
        }

        print!("###\n\n");

        return;
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)
        .expect("Failed to open matches file");

    file.seek(SeekFrom::Start(0))
        .expect("Failed to seek to the beginning of the file");
    serde_json::to_writer_pretty(&mut file, &existing_rounds)
        .expect("Failed to write matching rounds to file");
}
