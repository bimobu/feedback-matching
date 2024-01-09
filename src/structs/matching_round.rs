use crate::structs::r#match::Match;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchingRound {
    pub date: String,
    pub matches: Vec<Match>,
}
