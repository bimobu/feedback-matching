use crate::structs::{matching_round::MatchingRound, participant::Gender};

pub fn print_messages_for_round(matching_round: &MatchingRound) {
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
        let er_sie_capitalized = match &match_pair.receiver.gender {
            Gender::Male => "Er",
            Gender::Female => "Sie",
        };
        let er_sie = match &match_pair.receiver.gender {
            Gender::Male => "er",
            Gender::Female => "sie",
        };
        println!(
"Hi {giver_first_name} 😊 Dein Feedbackempfänger für die nächsten zwei Wochen ist {receiver_full_name}. \
\n
Deine Aufgabe ist es, die nächsten zwei Wochen etwas auf {ihn_sie} zu achten und {ihm_ihr} am Ende dieser zwei Wochen Feedback zu geben. \
Das Feedback sollte im Idealfall so Sachen wie das Verhalten in und außerhalb von Meetings, Verhalten im Team, Code, Eigeninitiative etc. enthalten. \
Mache am Ende der zwei Wochen bitte selber einen Termin mit {ihm_ihr} aus um {ihm_ihr} das Feedback zu geben. {er_sie_capitalized} selber weiß ja nicht, wer {ihm_ihr} das Feedback geben wird. \
Deswegen solltest du den Termin jetzt noch nicht erstellen. Sonst weiß {er_sie}, wer {ihm_ihr} Feedback geben wird, was das Ergebnis verfälschen könnte. \
\n
Viel Spaß 😊
---"
      );
    }
}
