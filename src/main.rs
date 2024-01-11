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
    write_matches("./data/matches.json", matching_round);
}
