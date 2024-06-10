use crate::structs::matching_round::MatchingRound;
use crate::structs::participant::{map_participants_to_match_participants, MatchParticipant};
use crate::structs::participants_file::ParticipantsFile;
use crate::structs::r#match::Match;
use crate::{MAX_SCORE, NUMBER_OF_TRIES};

use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp;
use std::collections::{HashMap, HashSet};
use time::OffsetDateTime;

use super::complete_givers::get_complete_givers_per_group;
use super::last_match_map::{get_days_since_last_match, get_last_match_map};

#[derive(Debug, Clone)]
struct MatchingGroup {
    id: i32,
    givers: Vec<MatchParticipant>,
    receivers: Vec<MatchParticipant>,
}

pub fn match_participants(
    participants_file: &ParticipantsFile,
    past_matching_rounds: &Vec<MatchingRound>,
    cross_team_round: bool,
    rng: &mut impl Rng,
) -> (MatchingRound, Vec<(i32, i64)>) {
    let next_matching_round_id = get_next_matching_round_id(past_matching_rounds);

    let last_match_map = get_last_match_map(past_matching_rounds);

    let mut scores_by_group: Vec<(i32, i64)> = Vec::new();
    let mut matches: Vec<Match> = Vec::new();
    let groups = get_groups(participants_file, cross_team_round, &last_match_map);

    for group in &groups {
        let matches_and_score = get_good_matches(&group, &last_match_map, rng);

        match matches_and_score {
            Some((matches_for_group, score_for_group)) => {
                let mut mut_matches = matches_for_group;
                matches.append(&mut mut_matches);

                scores_by_group.push((group.id, score_for_group));
            }
            None => {
                panic!("Failed to match participants for group {}", group.id);
            }
        }
    }

    let matching_round = MatchingRound {
        id: next_matching_round_id,
        date: OffsetDateTime::now_utc().date(),
        matches,
    };

    (matching_round, scores_by_group)
}

fn get_groups(
    participants_file: &ParticipantsFile,
    cross_team_round: bool,
    last_match_map: &HashMap<(u32, u32), i64>,
) -> Vec<MatchingGroup> {
    if participants_file.groups.len() > 2 {
        panic!("There are more than 2 groups")
    }

    let mut matching_groups: Vec<MatchingGroup> = participants_file
        .groups
        .iter()
        .map(|g| MatchingGroup {
            id: g.id,
            givers: map_participants_to_match_participants(&g.participants, g.id), // they would have to be cloned anyway, so I'm just mapping twice here
            receivers: map_participants_to_match_participants(&g.participants, g.id),
        })
        .collect();

    // Rudimentary first version of cross-team-matching with only two groups
    if cross_team_round {
        let complete_givers_per_group =
            get_complete_givers_per_group(&participants_file.groups, last_match_map);

        let group_1 = matching_groups[0].clone();
        let group_2 = matching_groups[1].clone();

        let group_1_complete_givers_participants = complete_givers_per_group
            .get(&group_1.id)
            .expect("no complete givers for group 1");
        let group_1_complete_givers = map_participants_to_match_participants(
            group_1_complete_givers_participants,
            group_1.id,
        );

        let group_2_complete_givers_participants = complete_givers_per_group
            .get(&group_2.id)
            .expect("no complete givers for group 2");
        let group_2_complete_givers = map_participants_to_match_participants(
            group_2_complete_givers_participants,
            group_2.id,
        );

        let number_of_switched_participants =
            cmp::min(group_1_complete_givers.len(), group_2_complete_givers.len());

        let givers_to_be_switched_from_group_1 =
            group_1_complete_givers[0..number_of_switched_participants].to_vec();
        let givers_to_be_switched_from_group_2 =
            group_2_complete_givers[0..number_of_switched_participants].to_vec();

        let giver_ids_to_be_switched_from_group_1: HashSet<u32> =
            givers_to_be_switched_from_group_1
                .iter()
                .map(|g| g.id)
                .collect();
        let giver_ids_to_be_switched_from_group_2: HashSet<u32> =
            givers_to_be_switched_from_group_2
                .iter()
                .map(|g| g.id)
                .collect();

        let group_1_givers: Vec<MatchParticipant> = group_1
            .givers
            .iter()
            .cloned()
            .filter(|g| !giver_ids_to_be_switched_from_group_1.contains(&g.id))
            .chain(givers_to_be_switched_from_group_2.iter().cloned())
            .collect();
        let group_2_givers: Vec<MatchParticipant> = group_2
            .givers
            .iter()
            .cloned()
            .filter(|g| !giver_ids_to_be_switched_from_group_2.contains(&g.id))
            .chain(givers_to_be_switched_from_group_1.iter().cloned())
            .collect();

        matching_groups[0].givers = group_1_givers;
        matching_groups[1].givers = group_2_givers;
    }

    matching_groups
}

fn get_good_matches(
    matching_group: &MatchingGroup,
    last_match_map: &HashMap<(u32, u32), i64>,
    rng: &mut impl Rng,
) -> Option<(Vec<Match>, i64)> {
    let mut best_score = i64::MIN;
    let mut best_match_and_score = None;

    for _ in 0..NUMBER_OF_TRIES {
        let mut cloned_unmatched_givers = matching_group.givers.clone();
        let mut cloned_unmatched_receivers = matching_group.receivers.clone();

        cloned_unmatched_givers.shuffle(rng);
        cloned_unmatched_receivers.shuffle(rng);

        let matches = get_matches(
            &last_match_map,
            cloned_unmatched_givers,
            cloned_unmatched_receivers,
        );

        match matches {
            Some(matches) => {
                let score = score_matches(&matches);

                if score == MAX_SCORE {
                    return Some((matches, score));
                }

                if score > best_score {
                    best_score = score;
                    best_match_and_score = Some((matches, score));
                }
            }
            None => continue,
        }
    }

    best_match_and_score
}

fn get_matches(
    last_match_map: &HashMap<(u32, u32), i64>,
    mut unmatched_givers: Vec<MatchParticipant>,
    mut unmatched_receivers: Vec<MatchParticipant>,
) -> Option<Vec<Match>> {
    let mut matches: Vec<Match> = Vec::new();

    while !unmatched_givers.is_empty() {
        let giver = &unmatched_givers[0];
        let giver_id = giver.id;

        let best_receiver_index_option =
            get_optimal_receiver_index_and_score(last_match_map, giver_id, &unmatched_receivers);

        match best_receiver_index_option {
            Some((index, score)) => {
                let giver = unmatched_givers.swap_remove(0);
                let receiver = unmatched_receivers.swap_remove(index);
                matches.push(create_match(giver, receiver, score));
            }
            None => {
                return None;
            }
        }
    }

    return Some(matches);
}

fn get_optimal_receiver_index_and_score(
    last_match_map: &HashMap<(u32, u32), i64>,
    giver_id: u32,
    unmatched_receivers: &Vec<MatchParticipant>,
) -> Option<(usize, i64)> {
    let mut best_receiver_score = i64::MIN;
    let mut best_receiver_index = None;

    for (i, receiver) in unmatched_receivers.iter().enumerate() {
        if giver_id == receiver.id {
            continue;
        }

        let score = get_days_since_last_match(last_match_map, giver_id, receiver.id);

        if best_receiver_score < score {
            best_receiver_score = score;
            best_receiver_index = Some((i, score));
        }
    }

    best_receiver_index
}

fn create_match(giver: MatchParticipant, receiver: MatchParticipant, score: i64) -> Match {
    return Match {
        giver,
        receiver,
        score,
    };
}

fn score_matches(matches: &Vec<Match>) -> i64 {
    let days_since_last_matches: Vec<i64> = matches.iter().map(|m| m.score).collect();
    let sum: i64 = days_since_last_matches.iter().sum();
    let number_of_matches = matches.len() as i64;

    if number_of_matches == 0 {
        0
    } else {
        sum / number_of_matches
    }
}

fn get_next_matching_round_id(past_matching_rounds: &Vec<MatchingRound>) -> i32 {
    let last_matching_round = past_matching_rounds.last();

    return match last_matching_round {
        Some(matching_round) => matching_round.id + 1,
        None => 1,
    };
}

// TODO fix tests
// Might make sense to split this up in modules to make it easier to test
// #[cfg(test)]
// mod tests {
//     use std::vec;

//     use crate::structs::{participant::Gender, participants_file::ParticipantsGroup};

//     use super::*;
//     use pretty_assertions::assert_eq;
//     use rand::SeedableRng;
//     use rand_chacha::ChaCha8Rng;
//     use time::Date;

//     fn get_seeded_rng() -> ChaCha8Rng {
//         let rng = ChaCha8Rng::seed_from_u64(14);
//         return rng;
//     }

//     #[test]
//     fn test_match_participants() {
//         // Arrange
//         let participants_data = ParticipantsFile {
//             groups: vec![ParticipantsGroup {
//                 id: 1,
//                 participants: vec![
//                     Participant {
//                         id: 1,
//                         first_name: "John".to_string(),
//                         last_name: "Doe".to_string(),
//                         gender: Gender::Male,
//                     },
//                     Participant {
//                         id: 2,
//                         first_name: "Jane".to_string(),
//                         last_name: "Smith".to_string(),
//                         gender: Gender::Female,
//                     },
//                     Participant {
//                         id: 3,
//                         first_name: "Bob".to_string(),
//                         last_name: "Johnson".to_string(),
//                         gender: Gender::Male,
//                     },
//                 ],
//                 excluded_participants: vec![],
//             }],
//         };
//         let past_matching_rounds: Vec<MatchingRound> = vec![MatchingRound {
//             id: 1,
//             date: Date::from_calendar_date(2024, time::Month::January, 11).expect(""),
//             matches: vec![
//                 Match {
//                     giver: Participant {
//                         id: 1,
//                         first_name: "John".to_string(),
//                         last_name: "Doe".to_string(),
//                         gender: Gender::Male,
//                     },
//                     receiver: Participant {
//                         id: 3,
//                         first_name: "Bob".to_string(),
//                         last_name: "Johnson".to_string(),
//                         gender: Gender::Male,
//                     },
//                 },
//                 Match {
//                     giver: Participant {
//                         id: 3,
//                         first_name: "Bob".to_string(),
//                         last_name: "Johnson".to_string(),
//                         gender: Gender::Male,
//                     },
//                     receiver: Participant {
//                         id: 2,
//                         first_name: "Jane".to_string(),
//                         last_name: "Smith".to_string(),
//                         gender: Gender::Female,
//                     },
//                 },
//                 Match {
//                     giver: Participant {
//                         id: 2,
//                         first_name: "Jane".to_string(),
//                         last_name: "Smith".to_string(),
//                         gender: Gender::Female,
//                     },
//                     receiver: Participant {
//                         id: 1,
//                         first_name: "John".to_string(),
//                         last_name: "Doe".to_string(),
//                         gender: Gender::Male,
//                     },
//                 },
//             ],
//         }];
//         let mut rng = get_seeded_rng();

//         // Act
//         let (matching_round, _) =
//             match_participants(&participants_data, &past_matching_rounds, &mut rng);

//         // Assert
//         assert_eq!(
//             matching_round.matches.len(),
//             participants_data.groups[0].participants.len()
//         );

//         for matched_pair in matching_round.matches.iter() {
//             assert_ne!(matched_pair.giver.id, matched_pair.receiver.id);
//         }
//     }

//     #[test]
//     fn test_match_participants_empty() {
//         // Arrange
//         let participants_data = ParticipantsFile {
//             groups: vec![ParticipantsGroup {
//                 id: 1,
//                 participants: vec![],
//                 excluded_participants: vec![],
//             }],
//         };
//         let past_matching_rounds = vec![];
//         let mut rng = get_seeded_rng();

//         // Act
//         let (matching_round, _) =
//             match_participants(&participants_data, &past_matching_rounds, &mut rng);

//         // Assert
//         assert!(matching_round.matches.is_empty());
//     }

//     #[test]
//     fn test_match_participants_single_participant() {
//         // Arrange
//         let participants_data = ParticipantsFile {
//             groups: vec![ParticipantsGroup {
//                 id: 1,
//                 participants: vec![Participant {
//                     id: 1,
//                     first_name: "John".to_string(),
//                     last_name: "Doe".to_string(),
//                     gender: crate::structs::participant::Gender::Male,
//                 }],
//                 excluded_participants: vec![],
//             }],
//         };
//         let past_matching_rounds = vec![];
//         let mut rng = get_seeded_rng();

//         // Act
//         let (matching_round, _) =
//             match_participants(&participants_data, &past_matching_rounds, &mut rng);

//         // Assert
//         assert!(matching_round.matches.is_empty());
//     }
// }
