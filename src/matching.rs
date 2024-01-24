use crate::structs::matching_round::MatchingRound;
use crate::structs::participant::Participant;
use crate::structs::participants_file::ParticipantsFile;
use crate::structs::r#match::Match;

use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;
use time::Duration;
use time::OffsetDateTime;

struct ParticipantWithPriority {
    participant: Participant,
    priority: i64,
}

pub fn match_participants(
    participants_file: &ParticipantsFile,
    past_matching_rounds: &Vec<MatchingRound>,
    rng: &mut impl Rng,
) -> MatchingRound {
    let unmatched_givers = participants_file.participants.clone();
    let unmatched_receivers = participants_file.participants.clone();

    // TODO use a sorted list instead of the second hashmap
    let priority_map = get_priority_map(
        past_matching_rounds,
        &unmatched_givers,
        &unmatched_receivers,
    );

    let matches = get_matches(&unmatched_givers, priority_map, rng);

    // Create and serialize the MatchingRound struct
    let next_matching_round_id = get_next_matching_round_id(past_matching_rounds);
    let matching_round = MatchingRound {
        id: next_matching_round_id,
        date: OffsetDateTime::now_utc().date(),
        matches,
    };

    matching_round
}

fn get_next_matching_round_id(past_matching_rounds: &Vec<MatchingRound>) -> i32 {
    let last_matching_round = past_matching_rounds.last();

    return match last_matching_round {
        Some(matching_round) => matching_round.id + 1,
        None => 1,
    };
}

fn get_matches(
    unmatched_givers: &Vec<Participant>,
    priority_map: HashMap<u32, Vec<Participant>>,
    rng: &mut impl Rng,
) -> Vec<Match> {
    let mut matches: Vec<Match> = Vec::new();

    let mut cloned_unmatched_givers = unmatched_givers.clone();
    let mut matched_receiver_ids = HashSet::<u32>::new();

    cloned_unmatched_givers.shuffle(rng);

    while !cloned_unmatched_givers.is_empty() {
        let giver = &cloned_unmatched_givers[0];
        let giver_id = giver.id;

        let giver_priority_map = priority_map.get(&giver_id);

        if giver_priority_map.is_none() {
            panic!("No priority map for giver {giver_id}");
        }

        let recievers_sorted_by_priority = giver_priority_map.unwrap();

        let receiver_option =
            get_optimal_receiver(&matched_receiver_ids, recievers_sorted_by_priority);

        if receiver_option.is_none() {
            panic!("No receiver found for giver {giver_id}");
        }

        let receiver = receiver_option.unwrap();

        matches.push(create_match(
            cloned_unmatched_givers.remove(0),
            receiver.clone(),
        ));

        matched_receiver_ids.insert(receiver.id);
    }

    return matches;
}

fn get_optimal_receiver<'a>(
    matched_receiver_ids: &'a HashSet<u32>,
    recievers_sorted_by_priority: &'a Vec<Participant>,
) -> Option<&'a Participant> {
    for receiver in recievers_sorted_by_priority {
        if !matched_receiver_ids.contains(&receiver.id) {
            return Some(receiver);
        }
    }

    None
}

fn get_priority_map(
    past_matching_rounds: &Vec<MatchingRound>,
    unmatched_givers: &Vec<Participant>,
    unmatched_receivers: &Vec<Participant>,
) -> HashMap<u32, Vec<Participant>> {
    let last_match_map = get_last_match_map(past_matching_rounds);

    let mut priority_map: HashMap<u32, Vec<Participant>> = HashMap::new();

    for giver in unmatched_givers {
        let mut participants_with_priorities: Vec<ParticipantWithPriority> = Vec::new();

        for receiver in unmatched_receivers {
            if giver.id == receiver.id {
                continue;
            }

            let duration_since_last_match = last_match_map.get(&(giver.id, receiver.id));
            let days_since_last_match = match &duration_since_last_match {
                Some(duration) => duration.whole_days(),
                None => i64::MAX,
            };

            participants_with_priorities.push(ParticipantWithPriority {
                participant: receiver.clone(),
                priority: days_since_last_match,
            });
        }

        participants_with_priorities.sort_by(|a, b| b.priority.cmp(&a.priority));

        let recievers_sorted_by_priority: Vec<Participant> = participants_with_priorities
            .iter()
            .map(|p| p.participant.clone())
            .collect();

        priority_map.insert(giver.id, recievers_sorted_by_priority);
    }

    return priority_map;
}

fn get_last_match_map(past_matching_rounds: &Vec<MatchingRound>) -> HashMap<(u32, u32), Duration> {
    let mut last_match_map: HashMap<(u32, u32), Duration> = HashMap::new();
    let today = OffsetDateTime::now_utc().date();

    for matching_round in past_matching_rounds {
        for past_match in &matching_round.matches {
            let time_since_last_match = today - matching_round.date;
            last_match_map.insert(
                (past_match.giver.id, past_match.receiver.id),
                time_since_last_match,
            );
        }
    }

    return last_match_map;
}

fn create_match(giver: Participant, receiver: Participant) -> Match {
    return Match {
        giver: giver.clone(),
        receiver: receiver.clone(),
    };
}

#[cfg(test)]
mod tests {
    use crate::structs::participant::Gender;

    use super::*;
    use pretty_assertions::assert_eq;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use time::Date;

    fn get_seeded_rng() -> ChaCha8Rng {
        let rng = ChaCha8Rng::seed_from_u64(14);
        return rng;
    }

    #[test]
    fn test_match_participants() {
        // Arrange
        let participants_data = ParticipantsFile {
            participants: vec![
                Participant {
                    id: 1,
                    first_name: "John".to_string(),
                    last_name: "Doe".to_string(),
                    gender: Gender::Male,
                },
                Participant {
                    id: 2,
                    first_name: "Jane".to_string(),
                    last_name: "Smith".to_string(),
                    gender: Gender::Female,
                },
                Participant {
                    id: 3,
                    first_name: "Bob".to_string(),
                    last_name: "Johnson".to_string(),
                    gender: Gender::Male,
                },
            ],
        };
        let past_matching_rounds: Vec<MatchingRound> = vec![MatchingRound {
            id: 1,
            date: Date::from_calendar_date(2024, time::Month::January, 11).expect(""),
            matches: vec![
                Match {
                    giver: Participant {
                        id: 1,
                        first_name: "John".to_string(),
                        last_name: "Doe".to_string(),
                        gender: Gender::Male,
                    },
                    receiver: Participant {
                        id: 3,
                        first_name: "Bob".to_string(),
                        last_name: "Johnson".to_string(),
                        gender: Gender::Male,
                    },
                },
                Match {
                    giver: Participant {
                        id: 3,
                        first_name: "Bob".to_string(),
                        last_name: "Johnson".to_string(),
                        gender: Gender::Male,
                    },
                    receiver: Participant {
                        id: 2,
                        first_name: "Jane".to_string(),
                        last_name: "Smith".to_string(),
                        gender: Gender::Female,
                    },
                },
                Match {
                    giver: Participant {
                        id: 2,
                        first_name: "Jane".to_string(),
                        last_name: "Smith".to_string(),
                        gender: Gender::Female,
                    },
                    receiver: Participant {
                        id: 1,
                        first_name: "John".to_string(),
                        last_name: "Doe".to_string(),
                        gender: Gender::Male,
                    },
                },
            ],
        }];
        let mut rng = get_seeded_rng();

        // Act
        let matching_round =
            match_participants(&participants_data, &past_matching_rounds, &mut rng);

        // Assert
        assert_eq!(
            matching_round.matches.len(),
            participants_data.participants.len()
        );

        for matched_pair in matching_round.matches.iter() {
            assert_ne!(matched_pair.giver.id, matched_pair.receiver.id);
        }
    }

    #[test]
    fn test_match_participants_empty() {
        // Arrange
        let participants_data = ParticipantsFile {
            participants: vec![],
        };
        let past_matching_rounds = vec![];
        let mut rng = get_seeded_rng();

        // Act
        let matching_round =
            match_participants(&participants_data, &past_matching_rounds, &mut rng);

        // Assert
        assert!(matching_round.matches.is_empty());
    }

    #[test]
    fn test_match_participants_single_participant() {
        // Arrange
        let participants_data = ParticipantsFile {
            participants: vec![Participant {
                id: 1,
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                gender: crate::structs::participant::Gender::Male,
            }],
        };
        let past_matching_rounds = vec![];
        let mut rng = get_seeded_rng();

        // Act
        let matching_round =
            match_participants(&participants_data, &past_matching_rounds, &mut rng);

        // Assert
        assert!(matching_round.matches.is_empty());
    }
}

// pub fn match_participants(
//     participants_file: &ParticipantsFile,
//     past_matching_rounds: &Vec<MatchingRound>,
//     rng: &mut impl Rng,
// ) -> MatchingRound {
//     let mut matches: Vec<Match> = Vec::new();

//     let mut unmatched_givers = participants_file.participants.clone();
//     let mut unmatched_receivers = participants_file.participants.clone();

//     let priority_map = get_priority_map(
//         past_matching_rounds,
//         &unmatched_givers,
//         &unmatched_receivers,
//     );

//     // TODO turn these into maps (how are they iterated over, is it then always the same order?)
//     unmatched_givers.shuffle(rng);
//     unmatched_receivers.shuffle(rng);

//     while !unmatched_givers.is_empty() {
//         let giver = &unmatched_givers[0];
//         let giver_id = giver.id;

//         let giver_priority_map = priority_map.get(&giver_id);

//         if giver_priority_map.is_none() {
//             panic!("No priority map for giver {giver_id}");
//         }

//         let giver_priority_map = giver_priority_map.unwrap();

//         if unmatched_givers.len() == 2 {
//             let unmatched_giver_ids: Vec<u32> = unmatched_givers.iter().map(|p| p.id).collect();
//             let unmatched_receiver_ids: Vec<u32> =
//                 unmatched_receivers.iter().map(|p| p.id).collect();

//             let unmatched_givers_set: HashSet<u32> = unmatched_giver_ids.into_iter().collect();
//             let unmatched_receivers_set: HashSet<u32> =
//                 unmatched_receiver_ids.into_iter().collect();

//             let intersection: Vec<&u32> = unmatched_givers_set
//                 .intersection(&unmatched_receivers_set)
//                 .collect();

//             if intersection.len() == 2 {
//                 if unmatched_givers[0].id == unmatched_receivers[0].id
//                     || unmatched_givers[1].id == unmatched_receivers[1].id
//                 {
//                     matches.push(create_match(
//                         unmatched_givers.remove(1),
//                         unmatched_receivers.remove(0),
//                     ));
//                 } else {
//                     matches.push(create_match(
//                         unmatched_givers.remove(0),
//                         unmatched_receivers.remove(0),
//                     ));
//                 }
//             } else if intersection.len() == 1 {
//                 if unmatched_givers[0].id == unmatched_receivers[0].id {
//                     matches.push(create_match(
//                         unmatched_givers.remove(1),
//                         unmatched_receivers.remove(0),
//                     ));
//                 } else {
//                     matches.push(create_match(
//                         unmatched_givers.remove(0),
//                         unmatched_receivers.remove(0),
//                     ));
//                 }
//             } else {
//                 let receiver_index =
//                     get_optimal_receiver_index(&unmatched_receivers, giver_priority_map);

//                 matches.push(create_match(
//                     unmatched_givers.remove(0),
//                     unmatched_receivers.remove(receiver_index),
//                 ))
//             }
//         } else {
//             let receiver_index =
//                 get_optimal_receiver_index(&unmatched_receivers, giver_priority_map);

//             matches.push(create_match(
//                 unmatched_givers.remove(0),
//                 unmatched_receivers.remove(receiver_index),
//             ))
//         }
//     }

//     // Create and serialize the MatchingRound struct
//     let matching_round = MatchingRound {
//         date: OffsetDateTime::now_utc().date(),
//         matches,
//     };

//     matching_round
// }
