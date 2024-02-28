use time::{format_description, Duration};

use crate::structs::{matching_round::MatchingRound, participant::Gender};

pub fn print_messages_for_round(matching_round: &MatchingRound) {
    let feedback_date_string = feedback_date(matching_round);
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
        println!(
"Hi {giver_first_name} 😊 Dein Feedbackempfänger für die nächsten zwei Wochen ist {receiver_full_name}. \
\n
Deine Aufgabe ist es, die nächsten zwei Wochen etwas auf {ihn_sie} zu achten und {ihm_ihr} am Ende dieser zwei Wochen (z.B. am {feedback_date_string}) Feedback zu geben. \
Das Feedback sollte im Idealfall so Sachen wie das Verhalten in und außerhalb von Meetings, Verhalten im Team, Code, Eigeninitiative etc. enthalten. \
Mache am Ende der zwei Wochen bitte selber einen Termin mit {ihm_ihr} aus um {ihm_ihr} das Feedback zu geben. \
Es gibt Donnerstags einen Blocker-Termin, den ihr dafür nutzen könnt. \
\n
Viel Spaß 😊
---"
      );
    }
}

fn feedback_date(matching_round: &MatchingRound) -> String {
    let feedback_weekday = 4;
    let match_weekday = matching_round.date.weekday().number_days_from_sunday() as i32;
    let days_to_add = 14 + (feedback_weekday - match_weekday) as i64;
    let feedback_date = matching_round.date + Duration::days(days_to_add);
    let format = format_description::parse("[day].[month].").unwrap();
    feedback_date.format(&format).unwrap()
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;
    use crate::structs::matching_round::MatchingRound;

    #[test]
    fn test_feedback_date_match_on_thursday() {
        let date = date!(2024 - 02 - 29);
        let matching_round = MatchingRound {
            id: 2,
            date,
            matches: vec![],
        };
        assert_eq!(feedback_date(&matching_round), "14.03.");
    }

    #[test]
    fn test_feedback_date_match_on_wednesday() {
        let date = date!(2024 - 02 - 28);
        let matching_round = MatchingRound {
            id: 2,
            date,
            matches: vec![],
        };
        assert_eq!(feedback_date(&matching_round), "14.03.");
    }

    #[test]
    fn test_feedback_date_match_on_friday() {
        let date = date!(2024 - 03 - 01);
        let matching_round = MatchingRound {
            id: 2,
            date,
            matches: vec![],
        };
        assert_eq!(feedback_date(&matching_round), "14.03.");
    }
}
