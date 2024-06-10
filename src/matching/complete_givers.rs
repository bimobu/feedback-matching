use crate::structs::matching_round::MatchingRound;
use crate::structs::participant::Participant;
use crate::structs::participants_file::{ParticipantsFile, ParticipantsGroup};

use std::collections::HashMap;

use super::last_match_map::get_last_match_map;

pub fn get_complete_givers(
    participants_file: &ParticipantsFile,
    past_matching_rounds: &Vec<MatchingRound>,
) -> HashMap<i32, Vec<Participant>> {
    let last_match_map = get_last_match_map(past_matching_rounds);
    let groups = &participants_file.groups;

    let complete_givers_per_group = get_complete_givers_per_group(&groups, &last_match_map);

    complete_givers_per_group
}

pub fn get_complete_givers_per_group(
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
