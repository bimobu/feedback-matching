use crate::structs::matching_round::MatchingRound;
use crate::structs::participant::Participant;
use crate::structs::participants_file::ParticipantsFile;
use crate::structs::r#match::Match;

use chrono::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

pub fn match_participants(
    participants_file: &ParticipantsFile,
    rng: &mut impl Rng,
) -> MatchingRound {
    let mut matches: Vec<Match> = Vec::new();

    let mut unmatched_givers = participants_file.participants.clone();

    let mut unmatched_receivers = participants_file.participants.clone();
    unmatched_receivers.shuffle(rng);

    println!("{:#?}", unmatched_givers);
    println!("{:#?}", unmatched_receivers);

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
        date: Utc::now().to_string(),
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
    use super::*;
    use crate::structs::participant::Participant;
    use pretty_assertions::assert_eq;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

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
                },
                Participant {
                    id: 2,
                    first_name: "Jane".to_string(),
                    last_name: "Smith".to_string(),
                },
                Participant {
                    id: 3,
                    first_name: "Bob".to_string(),
                    last_name: "Johnson".to_string(),
                },
            ],
        };
        let mut rng = get_seeded_rng();

        // Act
        let matching_round = match_participants(&participants_data, &mut rng);

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
        let mut rng = get_seeded_rng();

        // Act
        let matching_round = match_participants(&participants_data, &mut rng);

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
            }],
        };
        let mut rng = get_seeded_rng();

        // Act
        let matching_round = match_participants(&participants_data, &mut rng);

        // Assert
        assert!(matching_round.matches.is_empty());
    }
}
