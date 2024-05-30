use serde::{Deserialize, Serialize};

use super::participant::MatchParticipant;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Match {
    pub giver: MatchParticipant,
    pub receiver: MatchParticipant,
    pub score: i64,
}
