use std::fmt::Display;

pub struct Message {
    pub from: String,
    pub to: Vec<String>,
    pub msg: String,
}
impl From<String> for Message {
    fn from(str: String) -> Self {
        let vec = str.split(0x03 as char).collect::<Vec<&str>>();
        Message {
            from: vec[0].to_string(),
            to: vec[1].split(0x04 as char).map(|s| s.to_string()).collect(),
            msg: vec[2].to_string(),
        }
    }
}
impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for addr in &self.to {
            result += &addr;
            result += ", ";
        }
        write!(f, "{}->{}: {}", self.from, result, self.msg)
    }
}
