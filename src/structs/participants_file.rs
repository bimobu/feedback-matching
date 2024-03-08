use crate::structs::participant::Participant;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ParticipantsGroup {
    pub id: i32,
    pub participants: Vec<Participant>,
    pub excluded_participants: Vec<Participant>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ParticipantsFile {
    pub groups: Vec<ParticipantsGroup>,
}
