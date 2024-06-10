use crate::{structs::matching_round::MatchingRound, MAX_SCORE};

use std::collections::HashMap;
use time::OffsetDateTime;

pub fn get_last_match_map(past_matching_rounds: &Vec<MatchingRound>) -> HashMap<(u32, u32), i64> {
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

pub fn get_days_since_last_match(
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
