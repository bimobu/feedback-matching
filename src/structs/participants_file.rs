use crate::structs::participant::Participant;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ParticipantsGroup {
    pub id: i32,
    pub participants: Vec<Participant>,
}

#[derive(Debug, Deserialize)]
pub struct ParticipantsFile {
    pub groups: Vec<ParticipantsGroup>,
}
