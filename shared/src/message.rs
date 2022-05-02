use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{part::Parts, user::User};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub from: User,
    pub to: Vec<String>,
    pub msg: Parts,
}
impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} :  {}", self.from, self.msg)
    }
}
