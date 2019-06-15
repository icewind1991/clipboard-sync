use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClipboardCommand {
    #[serde(rename = "listen")]
    Listen { session: String },
    #[serde(rename = "set")]
    Set { session: String, value: String }
}