mod file_io;
mod matching;
mod structs;

use file_io::{read_participants, write_matches};
use matching::match_participants;

fn main() {
    // Read participants JSON file
    let participants_data = read_participants("./data/participants.json");

    // Match participants
    let matching_round = match_participants(&participants_data);

    // Print matches
    for match_pair in &matching_round.matches {
        println!(
            "Giver: {} - Receiver: {}",
            match_pair.giver.name, match_pair.receiver.name
        );
    }

    // Save matches to JSON file
    write_matches("./data/matches.json", matching_round);
}
