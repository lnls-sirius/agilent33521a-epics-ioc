use std::cmp;

use super::ScpiRequest;
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
    let request_data = skip_expected_chars(string, "OUTPut");

    if let Some((channel, command)) = parse_integer(request_data) {
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

fn parse_integer(string: &str) -> Option<(usize, &str)> {
    let mut digit_chars_indices = string.char_indices().filter_map(
        |(index, c)| if c.is_digit(10) { Some(index) } else { None },
    );

    let bytes_to_skip =
        digit_chars_indices.next().unwrap_or_else(|| string.len());

    if bytes_to_skip > 0 {
        let (number_string, remaining_bytes) = string.split_at(bytes_to_skip);

        if let Ok(number) = number_string.parse() {
            return Some((number, remaining_bytes));
        }
    }

    None
}

fn skip_expected_chars<'a, 'b>(string: &'a str, expected: &'b str) -> &'a str {
    let paired_chars = string.char_indices().zip(expected.chars());

    let mut indices_of_different_chars = paired_chars
        .filter_map(|((index, a), b)| if a == b { Some(index) } else { None });

    let bytes_to_skip = indices_of_different_chars
        .next()
        .unwrap_or_else(|| cmp::min(string.len(), expected.len()));

    skip_bytes(string, bytes_to_skip)
}

fn skip_bytes(string: &str, bytes_to_skip: usize) -> &str {
    let (_skipped_bytes, remaining_bytes) = string.split_at(bytes_to_skip);

    remaining_bytes
}
