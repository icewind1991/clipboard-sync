use err_derive::Error;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string, Error as SerdeError};
use std::convert::TryFrom;
use ws::{Error as WsError, ErrorKind, Message};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardCommand {
    Listen { session: String },
    Set { session: String, value: String },
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error(display = "Invalid message encoding")]
    Encoding,
    #[error(display = "Invalid formatted message: {}", _1)]
    InvalidMessage(#[error(cause)] SerdeError, String),
    #[error(display = "Unknown error")]
    Unknown,
}

impl From<WsError> for ParseError {
    fn from(from: WsError) -> Self {
        match from.kind {
            ErrorKind::Encoding(_) => ParseError::Encoding,
            _ => ParseError::Unknown,
        }
    }
}

impl TryFrom<Message> for ClipboardCommand {
    type Error = ParseError;

    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        let text = msg.as_text()?;
        from_str::<ClipboardCommand>(text)
            .map_err(|err| ParseError::InvalidMessage(err, text.into()))
    }
}

impl From<ClipboardCommand> for Message {
    fn from(command: ClipboardCommand) -> Self {
        Message::from(&command)
    }
}

impl From<&ClipboardCommand> for Message {
    fn from(command: &ClipboardCommand) -> Self {
        Message::from(to_string(command).unwrap())
    }
}
