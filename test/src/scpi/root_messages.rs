use super::errors::{ErrorKind, Result};
use super::message::ScpiMessage;
use super::messages::empty::EmptyMessage;

pub struct Messages;

impl Messages {
    pub fn empty() -> EmptyMessage {
        EmptyMessage {}
    }

    pub fn decode(string: &str) -> Result<Box<ScpiMessage>> {
        if string.len() == 0 {
            Ok(Box::new(EmptyMessage {}))
        } else {
            Err(ErrorKind::UnknownScpiMessage(String::from(string)).into())
        }
    }
}
