use crate::structs::r#match::Match;
use serde::{Deserialize, Serialize};
use time::Date;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchingRound {
    pub date: Date,
    pub matches: Vec<Match>,
}
