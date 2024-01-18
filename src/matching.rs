use crate::structs::matching_round::MatchingRound;
use crate::structs::participant::Participant;
use crate::structs::participants_file::ParticipantsFile;
use crate::structs::r#match::Match;

use rand::seq::SliceRandom;
use rand::Rng;
use time::OffsetDateTime;

pub fn match_participants(
    participants_file: &ParticipantsFile,
    _past_matching_rounds: &Vec<MatchingRound>,
    rng: &mut impl Rng,
) -> MatchingRound {
    let mut matches: Vec<Match> = Vec::new();

    let mut unmatched_givers = participants_file.participants.clone();

    let mut unmatched_receivers = participants_file.participants.clone();
    unmatched_receivers.shuffle(rng);

    while !unmatched_givers.is_empty() {
        if unmatched_givers.len() == 2 {
            if unmatched_givers[0].id == unmatched_receivers[0].id
                || unmatched_givers[1].id == unmatched_receivers[1].id
            {
                matches.push(create_match(
                    unmatched_givers.remove(1),
                    unmatched_receivers.remove(0),
                ));
            } else {
                matches.push(create_match(
                    unmatched_givers.remove(0),
                    unmatched_receivers.remove(0),
                ));
            }
        } else {
            if unmatched_givers[0].id == unmatched_receivers[0].id {
                if unmatched_givers.len() == 1 {
                    break;
                }

                matches.push(create_match(
                    unmatched_givers.remove(1),
                    unmatched_receivers.remove(0),
                ));
            } else {
                matches.push(create_match(
                    unmatched_givers.remove(0),
                    unmatched_receivers.remove(0),
                ));
            }
        }
    }

    // Create and serialize the MatchingRound struct
    let matching_round = MatchingRound {
        date: OffsetDateTime::now_utc().date(),
        matches,
    };

    matching_round
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
