mod file_io;
mod matching;
mod structs;

use clap::{Parser, Subcommand};
use file_io::{read, write_matches};
use matching::match_participants;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::structs::{
    matching_round::MatchingRound, participant::Gender, participants_file::ParticipantsFile,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new match
    CreateMatch {
        /// Not save the output to matches.json
        #[arg(short, long)]
        json_silent: bool,
        /// Print the messages for each match
        #[arg(short, long)]
        messages_silent: bool,
    },
    /// Print the messages for a past matching round
    PastMatchMessages {
        /// The matching round id to print messages for
        matching_round_id: Option<i32>,
    },
}

fn main() {
    let Args { command } = Args::parse();

    match command {
        Commands::PastMatchMessages { matching_round_id } => {
            print_messages_for_past_round(matching_round_id)
        }
        Commands::CreateMatch {
            json_silent: silent_json,
            messages_silent: silent_messages,
        } => create_match(silent_messages, silent_json),
    }
}

fn print_messages_for_past_round(matching_round_id: Option<i32>) {
    let past_matching_rounds = read::<Vec<MatchingRound>>("./data/matches.json");

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
                    println!("No matches have been created yet.");
                }
            };
        }
    }
}

fn create_match(silent_messages: bool, silent_json: bool) {
    // Read JSON Data
    let participants_file = read::<ParticipantsFile>("./data/participants.json");
    let past_matching_rounds = read::<Vec<MatchingRound>>("./data/matches.json");

    // Match participants
    let mut rng = ChaCha8Rng::from_entropy();
    let matching_round = match_participants(&participants_file, &past_matching_rounds, &mut rng);

    // Print messages
    if !silent_messages {
        print_messages_for_round(&matching_round);
    }

    // Save matches to JSON file
    if !silent_json {
        write_matches("./data/matches.json", matching_round);
    }
}

fn print_messages_for_round(matching_round: &MatchingRound) {
    println!("\n");

    for match_pair in &matching_round.matches {
        let giver_first_name = &match_pair.giver.first_name;
        let receiver_full_name = &match_pair.receiver.full_name();
        let ihn_sie = match &match_pair.receiver.gender {
            Gender::Male => "ihn",
            Gender::Female => "sie",
        };
        let ihm_ihr = match &match_pair.receiver.gender {
            Gender::Male => "ihm",
            Gender::Female => "ihr",
        };
        let er_sie = match &match_pair.receiver.gender {
            Gender::Male => "Er",
            Gender::Female => "Sie",
        };
        println!(
            "Hi {giver_first_name} 😊 Dein Feedbackempfänger für die nächsten zwei Wochen ist {receiver_full_name}. \n
Deine Aufgabe ist es, die nächsten zwei Wochen etwas auf {ihn_sie} zu achten und {ihm_ihr} am Ende dieser zwei Wochen Feedback zu geben. \
Das Feedback sollte im Idealfall so Sachen wie das Verhalten in und außerhalb von Meetings, Verhalten im Team, Code, Eigeninitiative etc. enthalten. \
Mache am Ende der zwei Wochen bitte selber einen Termin mit {ihm_ihr} aus um {ihm_ihr} das Feedback zu geben. {er_sie} selber weiß ja nicht, wer {ihm_ihr} das Feedback geben wird. \n
Viel Spaß 😊
---"
        );
    }
}
