use std::collections::HashMap;

use crate::last_match_map::{self, get_days_since_last_match};
use crate::structs::matching_round::MatchingRound;
use crate::structs::participant::MatchParticipant;
use crate::structs::participants_file::ParticipantsFile;
use crate::structs::r#match::Match;

use last_match_map::get_last_match_map;

pub fn calculate_scores(past_matching_rounds: &Vec<MatchingRound>) -> Vec<MatchingRound> {
    let mut new_matching_rounds = past_matching_rounds.clone();
    let mut passed_matching_rounds = Vec::<MatchingRound>::new();

    for matching_round in &mut new_matching_rounds {
        let last_match_map = get_last_match_map(&passed_matching_rounds);
        for group_match in &mut matching_round.matches {
            let score = get_days_since_last_match(
                &last_match_map,
                group_match.giver.id,
                group_match.receiver.id,
            );

            group_match.score = score;
        }

        passed_matching_rounds.push(matching_round.clone())
    }

    new_matching_rounds
}

pub fn update_matching_rounds_with_group_ids(
    past_matching_rounds: &Vec<MatchingRound>,
    participants_file: &ParticipantsFile,
) -> Vec<MatchingRound> {
    let participant_group_map = create_participant_group_map(participants_file);

    past_matching_rounds
        .iter()
        .map(|round| {
            let updated_matches = round
                .matches
                .iter()
                .map(|m| {
                    let giver_group_id =
                        get_group_id_for_participant(participant_group_map.clone(), &m.giver.id);
                    let receiver_group_id =
                        get_group_id_for_participant(participant_group_map.clone(), &m.receiver.id);

                    Match {
                        giver: MatchParticipant {
                            group_id: giver_group_id,
                            ..m.giver.clone()
                        },
                        receiver: MatchParticipant {
                            group_id: receiver_group_id,
                            ..m.receiver.clone()
                        },
                        ..m.clone()
                    }
                })
                .collect();

            MatchingRound {
                matches: updated_matches,
                ..round.clone()
            }
        })
        .collect()
}

fn get_group_id_for_participant(map: HashMap<u32, i32>, participant_id: &u32) -> i32 {
    *map.get(&participant_id)
        .expect("Could not find group for participant")
}

fn create_participant_group_map(participants_file: &ParticipantsFile) -> HashMap<u32, i32> {
    participants_file
        .groups
        .iter()
        .fold(HashMap::new(), |mut acc, group| {
            group
                .participants
                .iter()
                .chain(group.excluded_participants.iter())
                .for_each(|participant| {
                    acc.insert(participant.id, group.id);
                });
            acc
        })
}
