mod str_extensions;

mod output;

use std::fmt;
use std::fmt::{Display, Formatter};

use self::str_extensions::StrExtensions;
use super::errors::{ErrorKind, Result};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ScpiRequest {
    OutputOn(usize),
    OutputOff(usize),
    OutputStatus(usize),
}

impl ScpiRequest {
    pub fn from(string: &str) -> Result<ScpiRequest> {
        let first_four_chars = string.view_first_chars(4);

        match first_four_chars {
            "OUTP" => return output::decode(string),
            _ => {}
        };

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
