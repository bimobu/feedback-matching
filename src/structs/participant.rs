use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Participant {
    pub id: u32,
    pub first_name: String,
    pub last_name: String,
    pub gender: Gender,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MatchParticipant {
    pub id: u32,
    pub group_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub gender: Gender,
}

// TODO do this with traits, share the function for both structs
impl Participant {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

impl MatchParticipant {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

pub fn map_participants_to_match_participants(
    participants: &Vec<Participant>,
    group_id: i32,
) -> Vec<MatchParticipant> {
    participants
        .iter()
        .map(|p| map_participant_to_match_participant(p, group_id))
        .collect()
}

pub fn map_participant_to_match_participant(
    participant: &Participant,
    group_id: i32,
) -> MatchParticipant {
    MatchParticipant {
        id: participant.id,
        group_id,
        first_name: participant.first_name.clone(),
        last_name: participant.last_name.clone(),
        gender: participant.gender.clone(),
    }
}
