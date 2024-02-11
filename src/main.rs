mod file_io;
mod matching;
mod messages;
mod structs;

use clap::{Parser, Subcommand};
use file_io::{read_matching_rounds, read_participants, write_matches};
use matching::match_participants;
use messages::print_messages_for_round;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    /// The path to the directory where matches and participants are saved
    #[arg(short, long, default_value_t = {"./data/".to_string()})]
    data_path: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new match
    CreateMatch {
        /// Not save the output to matches.json
        #[arg(short, long)]
        json_save: bool,
        /// Print the messages for each match
        #[arg(short, long)]
        messages_generate: bool,
    },
    /// Print the messages for a past matching round
    PastMatchMessages {
        /// The matching round id to print messages for
        matching_round_id: Option<i32>,
    },
}

fn main() {
    let Args { command, data_path } = Args::parse();

    match command {
        Commands::PastMatchMessages { matching_round_id } => {
            print_messages_for_past_round(matching_round_id, &data_path)
        }
        Commands::CreateMatch {
            json_save: save_json,
            messages_generate: generate_messages,
        } => create_match(generate_messages, save_json, &data_path),
    }
}

fn matches_file_path(data_path: &String) -> String {
    format!("{}/matches.json", data_path)
}

fn participants_file_path(data_path: &String) -> String {
    format!("{}/participants.json", data_path)
}

fn print_messages_for_past_round(matching_round_id: Option<i32>, data_path: &String) {
    let past_matching_rounds = read_matching_rounds(&matches_file_path(data_path));

    match matching_round_id {
        None => {
            let last_matching_round = past_matching_rounds.last();

            match last_matching_round {
                Some(matching_round) => {
                    print_messages_for_round(matching_round);
                }
                None => {
                    println!("No matches have been created yet.");
                }
            };
        }
        Some(matching_round_id) => {
            let matching_round = past_matching_rounds
                .iter()
                .find(|r| r.id == matching_round_id);

            match matching_round {
                Some(matching_round) => {
                    print_messages_for_round(matching_round);
                }
                None => {
                    println!("No matching round with id {matching_round_id} has been found");
                }
            };
        }
    }
}

fn create_match(generate_messages: bool, save_json: bool, data_path: &String) {
    // Read JSON Data
    let participants_file = read_participants(&participants_file_path(data_path));
    let past_matching_rounds = read_matching_rounds(&matches_file_path(data_path));

    // Match participants
    let mut rng = ChaCha8Rng::from_entropy();
    let (matching_round, scores_by_group) =
        match_participants(&participants_file, &past_matching_rounds, &mut rng);

    // Print messages
    if generate_messages {
        print_messages_for_round(&matching_round);
    }

    // Save matches to JSON file
    if save_json {
        write_matches(&matches_file_path(data_path), matching_round);
    }

    for (group_id, score) in scores_by_group {
        println!("Group {} has a score of {}", group_id, score);
    }
}
