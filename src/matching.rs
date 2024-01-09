use crate::structs::matching_round::MatchingRound;
use crate::structs::participants_file::ParticipantsFile;
use crate::structs::r#match::Match;

use chrono::prelude::*;
use std::collections::HashSet;

pub fn match_participants(participants_data: &ParticipantsFile) -> MatchingRound {
    let mut matches: Vec<Match> = Vec::new();
    let mut unmatched_receivers: HashSet<u32> = participants_data
        .participants
        .iter()
        .map(|p| p.id)
        .collect();

    for giver in participants_data.participants.iter() {
        // Find an unmatched receiver with a different ID
        let receiver_id = unmatched_receivers
            .iter()
            .filter(|&&id| id != giver.id)
            .next()
            .cloned();

        if let Some(receiver_id) = receiver_id {
            unmatched_receivers.remove(&receiver_id);

            let receiver = participants_data
                .participants
                .iter()
                .find(|p| p.id == receiver_id)
                .unwrap();

            let new_match = Match {
                giver: giver.clone(),
                receiver: receiver.clone(),
            };

            matches.push(new_match);
        } else {
            // Handle the case where no unmatched receiver with a different ID is found
            println!("Error: No unmatched receiver found for giver {:?}", giver);
        }
    }

    // Create and serialize the MatchingRound struct
    let matching_round = MatchingRound {
        date: Utc::now().to_string(),
        matches,
    };

    matching_round
}
