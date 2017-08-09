use std::fmt;
use std::fmt::{Display, Formatter};

use super::errors::{ErrorKind, Result};

mod str_extensions;
mod output;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ScpiRequest {
    OutputOn(usize),
    OutputOff(usize),
    OutputStatus(usize),
}

impl ScpiRequest {
    pub fn from(string: &str) -> Result<ScpiRequest> {
        if string.len() > 4 {
            let mut fourth_char = 4;

            while !string.is_char_boundary(fourth_char) {
                fourth_char += 1;
            }

            let (first_four_chars, _) = string.split_at(fourth_char);

            match first_four_chars {
                "OUTP" => return output::decode(string),
                _ => {}
            };
        }

        Err(ErrorKind::UnknownScpiRequest(String::from(string)).into())
    }

    pub fn output(channel: usize) -> output::Builder {
        output::Builder::with_channel(channel)
    }
}

impl Display for ScpiRequest {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            ScpiRequest::OutputOn(channel) => {
                write!(formatter, "OUTP{} ON", channel)
            }
            ScpiRequest::OutputOff(channel) => {
                write!(formatter, "OUTP{} OFF", channel)
            }
            ScpiRequest::OutputStatus(channel) => {
                write!(formatter, "OUTP{}?", channel)
            }
        }
    }
}
