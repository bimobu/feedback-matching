use crate::structs::matching_round::MatchingRound;
use crate::structs::participant::{map_participants_to_match_participants, MatchParticipant};
use crate::structs::participants_file::ParticipantsFile;
use crate::structs::r#match::Match;
use crate::{MAX_SCORE, NUMBER_OF_TRIES};

use super::last_match_map::{get_days_since_last_match, get_last_match_map};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
struct MatchingGroup {
    participants: Vec<MatchParticipant>,
}

pub fn match_participants(
    participants_file: &ParticipantsFile,
    past_matching_rounds: &Vec<MatchingRound>,
    cross_team_round: bool,
    rng: &mut impl Rng,
) -> MatchingRound {
    let matching_groups: Vec<MatchingGroup> = participants_file
        .groups
        .iter()
        .map(|g| MatchingGroup {
            participants: map_participants_to_match_participants(&g.participants, g.id),
        })
        .collect();

    let last_match_map = get_last_match_map(past_matching_rounds);
    let mut best_score = i64::MIN;
    let mut best_matches_and_score = None;
    let number_of_participants: usize = participants_file
        .groups
        .iter()
        .map(|g| g.participants.len())
        .sum();

    for _ in 0..NUMBER_OF_TRIES {
        let mut overall_matches: Vec<Match> = Vec::new();
        let mut overall_unmatched_givers = Vec::new();
        let mut overall_unmatched_receivers = Vec::new();

        for group in &matching_groups {
            // why mut?
            let (mut matches, mut unmatched_givers, mut unmatched_receivers) = get_good_matches(
                &group.participants,
                &group.participants,
                &last_match_map,
                cross_team_round,
                rng,
            );

            overall_matches.append(&mut matches);
            overall_unmatched_givers.append(&mut unmatched_givers);
            overall_unmatched_receivers.append(&mut unmatched_receivers);
        }

        if cross_team_round {
            let (mut matches, unmatched_givers, unmatched_receivers) = get_good_matches(
                &overall_unmatched_givers,
                &overall_unmatched_receivers,
                &last_match_map,
                false,
                rng,
            );

            overall_matches.append(&mut matches);
            overall_unmatched_givers = unmatched_givers;
            overall_unmatched_receivers = unmatched_receivers;
        }

        let score = score_matches(&overall_matches, number_of_participants);
        let unmatched_givers = overall_unmatched_givers
            .iter()
            .map(|g| g.full_name())
            .collect::<Vec<String>>();
        let unmatched_receivers = overall_unmatched_receivers
            .iter()
            .map(|r| r.full_name())
            .collect::<Vec<String>>();

        if best_score < score {
            best_score = score;
            best_matches_and_score = Some((
                overall_matches,
                score,
                unmatched_givers,
                unmatched_receivers,
            ));
        }
    }

    let (matches, _score, unmatched_givers, unmatched_receivers) =
        best_matches_and_score.expect("No matches created");

    if !unmatched_givers.is_empty() {
        println!("Unmatched givers: {:#?}", unmatched_givers);
    }

    if !unmatched_receivers.is_empty() {
        println!("Unmatched receivers: {:#?}", unmatched_receivers);
    }

    let next_matching_round_id = get_next_matching_round_id(past_matching_rounds);

    let matching_round = MatchingRound {
        id: next_matching_round_id,
        date: OffsetDateTime::now_utc().date(),
        matches,
    };

    matching_round
}

fn get_good_matches(
    givers: &Vec<MatchParticipant>,
    receivers: &Vec<MatchParticipant>,
    last_match_map: &HashMap<(u32, u32), i64>,
    skip_matches_below_max_score: bool,
    rng: &mut impl Rng,
) -> (Vec<Match>, Vec<MatchParticipant>, Vec<MatchParticipant>) {
    let mut unmatched_givers = get_shuffled_vector(givers, rng);
    let mut unmatched_receivers = get_shuffled_vector(receivers, rng);

    let mut matches: Vec<Match> = Vec::new();
    let mut skipped_giver_count = 0;

    while unmatched_givers.len() > skipped_giver_count {
        let unmatched_giver = &unmatched_givers[0];
        let giver_id = unmatched_giver.id;

        let best_receiver_index_and_score_option =
            get_optimal_receiver_index_and_score(last_match_map, giver_id, &unmatched_receivers);

        match best_receiver_index_and_score_option {
            Some((index, score)) => {
                if skip_matches_below_max_score && score < MAX_SCORE {
                    skipped_giver_count += 1;
                    continue;
                }
                let giver = unmatched_givers.swap_remove(0);
                let receiver = unmatched_receivers.swap_remove(index);
                matches.push(create_match(giver, receiver, score));
            }
            None => {
                skipped_giver_count += 1;
            }
        }
    }

    (matches, unmatched_givers, unmatched_receivers)
}

fn get_shuffled_vector<T: Clone>(vec: &Vec<T>, rng: &mut impl Rng) -> Vec<T> {
    let mut cloned_vec = vec.clone();
    cloned_vec.shuffle(rng);
    cloned_vec
}

fn get_optimal_receiver_index_and_score(
    last_match_map: &HashMap<(u32, u32), i64>,
    giver_id: u32,
    unmatched_receivers: &Vec<MatchParticipant>,
) -> Option<(usize, i64)> {
    let mut best_receiver_score = i64::MIN;
    let mut best_receiver_index_and_score = None;

    for (i, receiver) in unmatched_receivers.iter().enumerate() {
        if giver_id == receiver.id {
            continue;
        }

        let score = get_days_since_last_match(last_match_map, giver_id, receiver.id);

        if best_receiver_score < score {
            best_receiver_score = score;
            best_receiver_index_and_score = Some((i, score));
        }
    }

    best_receiver_index_and_score
}

fn create_match(giver: MatchParticipant, receiver: MatchParticipant, score: i64) -> Match {
    return Match {
        giver,
        receiver,
        score,
    };
}

fn score_matches(matches: &Vec<Match>, number_of_participants: usize) -> i64 {
    let days_since_last_matches: Vec<i64> = matches.iter().map(|m| m.score).collect();
    let sum: i64 = days_since_last_matches.iter().sum();

    if number_of_participants == 0 {
        0
    } else {
        sum / number_of_participants as i64
    }
}

fn get_next_matching_round_id(past_matching_rounds: &Vec<MatchingRound>) -> i32 {
    let last_matching_round = past_matching_rounds.last();

    return match last_matching_round {
        Some(matching_round) => matching_round.id + 1,
        None => 1,
    };
}

// TODO add tests
