use std::fmt::Display;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct User {
    pub uuid: String,
    pub name: String,
}
impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
