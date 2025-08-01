use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Contest {
    pub id: String,
    pub name: String,
    pub contest_type: String,
}

pub trait ContestType {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
}
