use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct PersonRow {
    pub person_id: String,
    pub firstname: String,
    pub lastname: String,
}

#[derive(Serialize, Debug,PartialEq, Eq,Hash)]
pub struct Person {
    pub person_id: String,
    pub firstname: String,
    pub lastname: String,
    pub events: Vec<Event>,
    pub sources: Vec<Source>
}

#[derive(Serialize, Debug,PartialEq, Eq,Hash)]
pub struct Event {
    pub event_type: String,
    pub timestamp: String,
    pub location: Option<String>,
    pub sources: Vec<usize>,
}

#[derive(Serialize, Debug,PartialEq, Eq,Hash)]
pub struct Source {
    pub num: usize,
    pub description: String
}