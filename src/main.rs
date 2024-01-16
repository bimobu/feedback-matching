mod file_io;
mod matching;
mod structs;

use clap::Parser;
use file_io::{read_participants, write_matches};
use matching::match_participants;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Whether the output should be saved to matches.json
    #[arg(short, long, default_value_t = false)]
    silent: bool,
}

fn main() {
    let args = Args::parse();
    let Args { silent } = args;

    // Read participants JSON file
    let participants_file = read_participants("./data/participants.json");

    // Match participants
    let mut rng = ChaCha8Rng::from_entropy();
    let matching_round = match_participants(&participants_file, &mut rng);

    // Print messages
    println!("\n");
    for match_pair in &matching_round.matches {
        let giver_first_name = &match_pair.giver.first_name;
        let receiver_full_name = &match_pair.receiver.full_name();
        println!(
            "Hi {giver_first_name} 😊 Dein Feedbackempfänger für die nächsten zwei Wochen ist {receiver_full_name}. \n
Deine Aufgabe ist es, die nächsten zwei Wochen etwas auf ihn zu achten und ihm am Ende dieser zwei Wochen Feedback zu geben. \
Das Feedback sollte im Idealfall so Sachen wie das Verhalten in und außerhalb von Meetings, Verhalten im Team, Code, Eigeninitiative etc. enthalten. \
Mache am Ende der zwei Wochen bitte selber einen Termin mit ihm aus um ihm das Feedback zu geben. Er selber weiß ja nicht, wer ihm das Feedback geben wird. \n
Viel Spaß 😊
---"
        );
    }

    // Save matches to JSON file
    if !silent {
        write_matches("./data/matches.json", matching_round);
    }
}
