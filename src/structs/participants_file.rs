use crate::structs::participant::Participant;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ParticipantsFile {
    pub participants: Vec<Participant>,
}
