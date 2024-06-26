mod file_io;
mod matching;
mod migrations;
mod structs;

use clap::{Parser, Subcommand};
use file_io::{
    read_matching_rounds, read_participants, save_matching_round, update_all_existing_rounds,
};
use matching::{
    complete_givers::get_complete_givers, matching::match_participants,
    messages::print_messages_for_round,
};
use migrations::{calculate_scores, update_matching_rounds_with_group_ids};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use structs::matching_round::MatchingRound;

pub const NUMBER_OF_TRIES: i32 = 50;
pub const MAX_SCORE: i64 = 1000000;

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
    /// Print the messages for a past matching round
    PastMatchMessages {
        /// The matching round id to print messages for
        matching_round_id: Option<i32>,
        /// The number of weeks to separate the matches
        #[arg(short, long, default_value_t = 4)]
        intervall_weeks: i32,
    },
    /// Print the messages for a past matching round
    PastRoundSummary {
        /// The matching round id to print messages for
        matching_round_id: Option<i32>,
    },
    /// Print the complete givers for every group
    CompleteGivers {},
    /// Create a new match
    CreateMatch {
        /// Save the output to matches.json
        #[arg(short, long)]
        json_save: bool,
        /// Print the messages for each match
        #[arg(short, long)]
        messages_generate: bool,
        /// The number of weeks to separate the matches (only really relevant for the messages)
        #[arg(short, long, default_value_t = 4)]
        intervall_weeks: i32,
        /// Match cross-teams
        #[arg(short, long)]
        cross_team_round: bool,
    },
    /// Deletes the last match
    DeleteMatch {},
    /// Execute data migrations
    CalculateAndSaveScores {},
    /// Execute data migrations
    AddGroupIdsToPastMatchParticipants {},
}

fn main() {
    let Args { command, data_path } = Args::parse();

    match command {
        Commands::PastMatchMessages {
            matching_round_id,
            intervall_weeks,
        } => print_messages_for_past_round(matching_round_id, intervall_weeks, &data_path),
        Commands::PastRoundSummary { matching_round_id } => {
            print_summary_for_past_round(matching_round_id, &data_path)
        }
        Commands::CompleteGivers {} => print_complete_givers(&data_path),
        Commands::CreateMatch {
            json_save: save_json,
            messages_generate: generate_messages,
            intervall_weeks,
            cross_team_round,
        } => create_match(
            generate_messages,
            save_json,
            intervall_weeks,
            cross_team_round,
            &data_path,
        ),
        Commands::DeleteMatch {} => delete_match(&data_path),
        Commands::CalculateAndSaveScores {} => calculate_and_save_scores(&data_path),
        Commands::AddGroupIdsToPastMatchParticipants {} => {
            add_group_ids_to_past_match_participants(&data_path)
        }
    }
}

fn matches_file_path(data_path: &String) -> String {
    format!("{}/matches.json", data_path)
}

fn participants_file_path(data_path: &String) -> String {
    format!("{}/participants.json", data_path)
}

fn print_messages_for_past_round(
    matching_round_id: Option<i32>,
    intervall_weeks: i32,
    data_path: &String,
) {
    let past_matching_rounds = read_matching_rounds(&matches_file_path(data_path));

    match matching_round_id {
        None => {
            let last_matching_round = past_matching_rounds.last();

            match last_matching_round {
                Some(matching_round) => {
                    print_messages_for_round(matching_round, intervall_weeks);
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
                    print_messages_for_round(matching_round, intervall_weeks);
                }
                None => {
                    println!("No matching round with id {matching_round_id} has been found");
                }
            };
        }
    }
}

fn print_summary_for_past_round(matching_round_id: Option<i32>, data_path: &String) {
    let past_matching_rounds = read_matching_rounds(&matches_file_path(data_path));

    match matching_round_id {
        None => {
            let last_matching_round = past_matching_rounds.last();

            match last_matching_round {
                Some(matching_round) => {
                    print_result(matching_round);
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
                    print_result(matching_round);
                }
                None => {
                    println!("No matching round with id {matching_round_id} has been found");
                }
            };
        }
    }
}

fn print_complete_givers(data_path: &String) {
    let participants_file = read_participants(&participants_file_path(data_path));
    let past_matching_rounds = read_matching_rounds(&matches_file_path(data_path));

    let complete_givers_by_group = get_complete_givers(&participants_file, &past_matching_rounds);

    for group_id in complete_givers_by_group.keys() {
        let complete_givers = complete_givers_by_group
            .get(group_id)
            .expect("Complete givers not found");
        let complete_giver_names: Vec<String> =
            complete_givers.iter().map(|g| g.full_name()).collect();

        println!(
            "Complete givers for group {group_id} {:#?}",
            complete_giver_names
        );
    }

    // println!("{:#?}", complete_givers_by_group);
}

fn create_match(
    generate_messages: bool,
    save_json: bool,
    intervall_weeks: i32,
    cross_team_round: bool,
    data_path: &String,
) {
    // Read JSON Data
    let participants_file = read_participants(&participants_file_path(data_path));
    let past_matching_rounds = read_matching_rounds(&matches_file_path(data_path));

    // Match participants
    let mut rng = ChaCha8Rng::from_entropy();
    let matching_round = match_participants(
        &participants_file,
        &past_matching_rounds,
        cross_team_round,
        &mut rng,
    );

    // Print messages
    if generate_messages {
        println!("\n### Messages: ###");
        print_messages_for_round(&matching_round, intervall_weeks);
    }

    print_result(&matching_round);

    // Save matches to JSON file
    if save_json {
        save_matching_round(&matches_file_path(data_path), matching_round);
    }
}

fn print_result(matching_round: &MatchingRound) {
    println!("\n### Result: ###\n");

    for group_match in &matching_round.matches {
        let giver_name = group_match.giver.full_name();
        let giver_group = group_match.giver.group_id;
        let receiver_name = group_match.receiver.full_name();
        let receiver_group = group_match.receiver.group_id;
        let score = group_match.score;
        let switch_info = match giver_group != receiver_group {
            true => {
                format!("Cross-Team Match between {giver_group} and {receiver_group}")
            }
            false => format!("In-Team Match in group {giver_group}"),
        };
        println!("{switch_info}: {giver_name} => {receiver_name}, score: {score}");
    }
}

fn delete_match(data_path: &String) {
    let mut past_matching_rounds = read_matching_rounds(&matches_file_path(data_path));
    past_matching_rounds.pop();
    update_all_existing_rounds(&matches_file_path(data_path), &past_matching_rounds);
}

fn calculate_and_save_scores(data_path: &String) {
    let past_matching_rounds = read_matching_rounds(&matches_file_path(data_path));
    let new_matching_rounds = calculate_scores(&past_matching_rounds);
    update_all_existing_rounds(&matches_file_path(data_path), &new_matching_rounds);
}

fn add_group_ids_to_past_match_participants(data_path: &String) {
    let past_matching_rounds = read_matching_rounds(&matches_file_path(data_path));
    let participants_file = read_participants(&participants_file_path(data_path));

    let new_matching_rounds =
        update_matching_rounds_with_group_ids(&past_matching_rounds, &participants_file);

    update_all_existing_rounds(&matches_file_path(data_path), &new_matching_rounds);
}
