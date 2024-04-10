use crate::structs::participant::Participant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Match {
    pub giver: Participant,
    pub receiver: Participant,
    pub score: i64,
}
