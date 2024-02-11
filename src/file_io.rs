use crate::structs::matching_round::MatchingRound;
use crate::structs::participants_file::ParticipantsFile;

use jsonschema::JSONSchema;
use serde::de::DeserializeOwned;
use serde_json::{from_reader, Value};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};

pub fn read_participants(file_path: &str) -> ParticipantsFile {
    read::<ParticipantsFile>(file_path, "./data/schema/participants_schema.json")
}

pub fn read_matching_rounds(file_path: &str) -> Vec<MatchingRound> {
    read::<Vec<MatchingRound>>(file_path, "./data/schema/matches_schema.json")
}

fn read<T>(file_path: &str, schema_path: &str) -> T
where
    T: DeserializeOwned,
{
    let data_string = read_string(file_path);
    let schema_string = read_string(schema_path);

    let schema: Value =
        serde_json::from_str(schema_string.as_str()).expect("Failed to parse JSON Schema");
    let json_data: Value =
        serde_json::from_str(data_string.as_str()).expect("Failed to parse JSON data");

    let compiled_schema = JSONSchema::compile(&schema).expect("Failed to compile JSON Schema");
    let validation_result = compiled_schema.validate(&json_data);

    if let Err(errors) = validation_result {
        println!("\n### JSON Validation Errors ###\n");

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

// TODO add JSON validation when writing
pub fn write_matches(file_path: &str, round: MatchingRound) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)
        .expect("Failed to open matches file");

    let mut existing_rounds: Vec<MatchingRound> = from_reader(&file).unwrap_or_else(|_| Vec::new());

    // Append a new MatchingRound to the "matches.json" array
    existing_rounds.push(round);

    // Serialize and write the updated matches array to the matches JSON file
    file.seek(SeekFrom::Start(0))
        .expect("Failed to seek to the beginning of the file");
    serde_json::to_writer_pretty(&mut file, &existing_rounds)
        .expect("Failed to write matching rounds to file");
}
