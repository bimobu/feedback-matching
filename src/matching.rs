use crate::structs::matching_round::MatchingRound;
use crate::structs::participant::Participant;
use crate::structs::participants_file::{ParticipantsFile, ParticipantsGroup};
use crate::structs::r#match::Match;

use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use time::OffsetDateTime;

const NUMBER_OF_TRIES: i32 = 50;
const MAX_SCORE: i64 = 1000000;

// struct MatchingParticipant {
//     // id, first_name, last_name, gender, swapped_from
// }

#[derive(Debug, Clone)]
struct MatchingGroup {
    id: i32,
    givers: Vec<Participant>,
    receivers: Vec<Participant>,
}

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

pub fn get_complete_givers(
    participants_file: &ParticipantsFile,
    past_matching_rounds: &Vec<MatchingRound>,
) -> HashMap<i32, Vec<Participant>> {
    let last_match_map = get_last_match_map(past_matching_rounds);
    let groups = &participants_file.groups;

    let complete_givers_per_group = get_complete_givers_per_group(&groups, &last_match_map);

    complete_givers_per_group
}

pub fn match_participants(
    participants_file: &ParticipantsFile,
    past_matching_rounds: &Vec<MatchingRound>,
    rng: &mut impl Rng,
) -> (MatchingRound, Vec<(i32, i64)>) {
    let next_matching_round_id = get_next_matching_round_id(past_matching_rounds);

    let last_match_map = get_last_match_map(past_matching_rounds);

    let mut scores_by_group: Vec<(i32, i64)> = Vec::new();
    let mut matches: Vec<Match> = Vec::new();
    let groups = get_groups(participants_file, next_matching_round_id, &last_match_map);

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
    _next_matching_round_id: i32,
    _last_match_map: &HashMap<(u32, u32), i64>,
) -> Vec<MatchingGroup> {
    // let is_cross_team_round = next_matching_round_id % 2 == 0;
    let matching_groups: Vec<MatchingGroup> = participants_file
        .groups
        .iter()
        .map(|g| MatchingGroup {
            id: g.id,
            givers: g.participants.clone(),
            receivers: g.participants.clone(),
        })
        .collect();

    // if is_cross_team_round {
    //     let complete_givers_per_group =
    //         get_complete_givers_per_group(&participants_file.groups, last_match_map);

    //     for group in &matching_groups {
    //         let group_complete_givers = complete_givers_per_group
    //             .get(&group.id)
    //             .expect("No complete giver for group");
    //         let clone = matching_groups.clone(); // Warum muss ich das hier in eine eigene Variable packen?
    //         let other_groups = clone.iter().filter(|g| g.id != group.id);

    //         for other_group in other_groups {
    //             let other_group_complete_givers = complete_givers_per_group
    //                 .get(&other_group.id)
    //                 .expect("No complete giver for group");

    //             if group_complete_givers.len() == other_group_complete_givers.len() {
    //             } else if group_complete_givers.len() <= other_group_complete_givers.len() {
    //             } else {
    //             }
    //         }
    //     }
    // }

    matching_groups
}

fn get_complete_givers_per_group(
    groups: &Vec<ParticipantsGroup>,
    last_match_map: &HashMap<(u32, u32), i64>,
) -> HashMap<i32, Vec<Participant>> {
    groups
        .iter()
        .map(|group| {
            (
                group.id,
                get_givers_who_have_matched_everyone_from_group(
                    &group.participants,
                    &last_match_map,
                ),
            )
        })
        .collect()
}

fn get_givers_who_have_matched_everyone_from_group(
    participants: &Vec<Participant>,
    last_match_map: &HashMap<(u32, u32), i64>,
) -> Vec<Participant> {
    participants
        .iter()
        .filter(|giver| has_giver_matched_all_receivers(&last_match_map, giver, participants))
        .cloned()
        .collect()
}

fn has_giver_matched_all_receivers(
    last_match_map: &HashMap<(u32, u32), i64>,
    giver: &Participant,
    participants: &Vec<Participant>,
) -> bool {
    for receiver in participants {
        if giver.id == receiver.id {
            continue;
        }

        if !last_match_map.contains_key(&(giver.id, receiver.id)) {
            return false;
        }
    }

    true
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

fn get_last_match_map(past_matching_rounds: &Vec<MatchingRound>) -> HashMap<(u32, u32), i64> {
    let mut last_match_map: HashMap<(u32, u32), i64> = HashMap::new();

    for matching_round in past_matching_rounds {
        let days_since_matching_round = get_days_since_matching_round(matching_round);

        for past_match in &matching_round.matches {
            last_match_map.insert(
                (past_match.giver.id, past_match.receiver.id),
                days_since_matching_round,
            );
        }
    }

    return last_match_map;
}

fn get_days_since_matching_round(matching_round: &MatchingRound) -> i64 {
    let today = OffsetDateTime::now_utc().date();
    let time_since_last_match = today - matching_round.date;
    return time_since_last_match.whole_days();
}

fn get_matches(
    last_match_map: &HashMap<(u32, u32), i64>,
    mut unmatched_givers: Vec<Participant>,
    mut unmatched_receivers: Vec<Participant>,
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
    unmatched_receivers: &Vec<Participant>,
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

fn create_match(giver: Participant, receiver: Participant, score: i64) -> Match {
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

fn get_days_since_last_match(
    last_match_map: &HashMap<(u32, u32), i64>,
    giver_id: u32,
    receiver_id: u32,
) -> i64 {
    let days_since_last_match_option = last_match_map.get(&(giver_id, receiver_id));

    let days_since_last_match = match &days_since_last_match_option {
        Some(duration) => **duration,
        None => MAX_SCORE,
    };

    days_since_last_match
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
