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

impl Participant {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}
