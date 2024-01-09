use crate::structs::matching_round::MatchingRound;
use crate::structs::participants_file::ParticipantsFile;

use serde_json::from_reader;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};

pub fn read_participants(file_path: &str) -> ParticipantsFile {
    let mut file = File::open(file_path).expect("Failed to open participants file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read participants file");

    from_reader(contents.as_bytes()).expect("Failed to parse participants JSON")
}

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
