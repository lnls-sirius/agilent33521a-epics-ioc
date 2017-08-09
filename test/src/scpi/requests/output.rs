use super::ScpiRequest;
use super::str_extensions::StrExtensions;
use super::super::errors::{ErrorKind, Result};

pub struct Builder {
    channel: usize,
}

impl Builder {
    pub fn with_channel(channel: usize) -> Builder {
        Builder { channel }
    }

    pub fn on(self) -> ScpiRequest {
        ScpiRequest::OutputOn(self.channel)
    }

    pub fn off(self) -> ScpiRequest {
        ScpiRequest::OutputOff(self.channel)
    }

    pub fn query(self) -> ScpiRequest {
        ScpiRequest::OutputStatus(self.channel)
    }
}

pub fn decode(string: &str) -> Result<ScpiRequest> {
    try_decode(string)
        .ok_or(ErrorKind::UnknownScpiRequest(String::from(string)).into())
}

pub fn try_decode(string: &str) -> Option<ScpiRequest> {
    let request_data = string.skip_expected_chars("OUTPut");

    if let Some((channel, command)) = request_data.parse_integer() {
        if command == "?" {
            return Some(ScpiRequest::OutputStatus(channel));
        } else if command.chars().next() == Some(' ') {
            match command.trim() {
                "ON" => return Some(ScpiRequest::OutputOn(channel)),
                "OFF" => return Some(ScpiRequest::OutputOff(channel)),
                _ => {}
            }
        }
    }

    None
}
