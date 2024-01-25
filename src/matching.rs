use crate::structs::matching_round::MatchingRound;
use crate::structs::participant::Participant;
use crate::structs::participants_file::ParticipantsFile;
use crate::structs::r#match::Match;

use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;
use time::OffsetDateTime;

struct ParticipantWithPriority {
    participant: Participant,
    priority: i64,
}

pub fn match_participants(
    participants_file: &ParticipantsFile,
    past_matching_rounds: &Vec<MatchingRound>,
    rng: &mut impl Rng,
) -> (MatchingRound, i64) {
    let unmatched_givers = participants_file.participants.clone();
    let unmatched_receivers = participants_file.participants.clone();

    let (matches, score) = get_good_matches(
        &unmatched_givers,
        &unmatched_receivers,
        &past_matching_rounds,
        rng,
    );

    let next_matching_round_id = get_next_matching_round_id(past_matching_rounds);
    let matching_round = MatchingRound {
        id: next_matching_round_id,
        date: OffsetDateTime::now_utc().date(),
        matches,
    };

    (matching_round, score)
}

fn get_good_matches(
    unmatched_givers: &Vec<Participant>,
    unmatched_receivers: &Vec<Participant>,
    past_matching_rounds: &Vec<MatchingRound>,
    rng: &mut impl Rng,
) -> (Vec<Match>, i64) {
    let last_match_map = get_last_match_map(past_matching_rounds);
    let priority_map =
        get_priority_map(&last_match_map, unmatched_givers, unmatched_receivers, rng);

    let mut matches_with_scores = Vec::new();

    for _ in 0..5 {
        let matches = get_matches(&unmatched_givers, &priority_map, rng);

        match matches {
            Some(matches) => {
                let score = score_matches(&last_match_map, &matches);

                if score == i64::MAX {
                    return (matches, score);
                }

                matches_with_scores.push((matches, score));
            }
            None => continue,
        }
    }

    let good_matches_and_score = matches_with_scores
        .iter()
        .max_by(|ms1, ms2| ms1.1.cmp(&ms2.1))
        .unwrap();

    good_matches_and_score.clone()
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

fn get_priority_map(
    last_match_map: &HashMap<(u32, u32), i64>,
    unmatched_givers: &Vec<Participant>,
    unmatched_receivers: &Vec<Participant>,
    rng: &mut impl Rng,
) -> HashMap<u32, Vec<Participant>> {
    let mut priority_map: HashMap<u32, Vec<Participant>> = HashMap::new();
    let mut cloned_unmatched_receivers = unmatched_receivers.clone();
    cloned_unmatched_receivers.shuffle(rng);

    for giver in unmatched_givers {
        let mut participants_with_priorities: Vec<ParticipantWithPriority> = Vec::new();

        for receiver in &cloned_unmatched_receivers {
            if giver.id == receiver.id {
                continue;
            }

            participants_with_priorities.push(ParticipantWithPriority {
                participant: receiver.clone(),
                priority: get_days_since_last_match(&last_match_map, giver.id, receiver.id),
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

fn get_matches(
    unmatched_givers: &Vec<Participant>,
    priority_map: &HashMap<u32, Vec<Participant>>,
    rng: &mut impl Rng,
) -> Option<Vec<Match>> {
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

        match receiver_option {
            Some(receiver) => {
                matches.push(create_match(
                    cloned_unmatched_givers.remove(0),
                    receiver.clone(),
                ));

                matched_receiver_ids.insert(receiver.id);
            }
            None => {
                return None;
            }
        }
    }

    return Some(matches);
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

fn create_match(giver: Participant, receiver: Participant) -> Match {
    return Match {
        giver: giver.clone(),
        receiver: receiver.clone(),
    };
}

fn score_matches(last_match_map: &HashMap<(u32, u32), i64>, matches: &Vec<Match>) -> i64 {
    let days_since_last_matches: Vec<i64> = matches
        .iter()
        .map(|m| get_days_since_last_match(last_match_map, m.giver.id, m.receiver.id))
        .collect();

    let min_days_since_last_match = days_since_last_matches.iter().min();

    match min_days_since_last_match {
        Some(min_days_since_last_match) => *min_days_since_last_match,
        None => 0,
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
        None => i64::MAX,
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
        let (matching_round, _) =
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
        let (matching_round, _) =
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
        let (matching_round, _) =
            match_participants(&participants_data, &past_matching_rounds, &mut rng);

        // Assert
        assert!(matching_round.matches.is_empty());
    }
}
