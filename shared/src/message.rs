use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub from: String,
    pub to: Vec<String>,
    pub msg: Parts,
}
impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut adresses = String::new();
        for addr in &self.to {
            adresses += addr;
            adresses += ", ";
        }
        write!(f, "{}->{}: {}", self.from, adresses, self.msg)
    }
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Part {
    Text(String),
}
#[derive(Serialize, Deserialize)]
pub struct Parts(Vec<Part>);
impl Display for Parts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut msg = String::new();
        for part in &self.0 {
            msg += &format!("{}", part);
        }
        write!(f, "{}", msg)
    }
}
impl Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            Part::Text(s) => s,
        };
        write!(f, "{}", result)
    }
}
