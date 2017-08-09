mod voltage;

use super::ScpiRequest;
use super::str_extensions::StrExtensions;
use super::super::errors::{ErrorKind, Result};

pub struct Builder {
    source: usize,
}

impl Builder {
    pub fn with_source(source: usize) -> Builder {
        Builder { source }
    }

    pub fn voltage(self) -> voltage::Builder {
        voltage::Builder::with_source(self.source)
    }
}

pub fn decode(string: &str) -> Result<ScpiRequest> {
    try_decode(string)
        .ok_or(ErrorKind::UnknownScpiRequest(String::from(string)).into())
}

pub fn try_decode(string: &str) -> Option<ScpiRequest> {
    let request_data = string.skip_expected_chars("SOURce");

    if let Some((source, command)) = request_data.parse_integer() {
        if command.starts_with(":") {
            let command = command.skip_chars(1);
            let first_four_chars = command.view_first_chars(4);

            match first_four_chars {
                "VOLT" => return voltage::try_decode(command, source),
                _ => {}
            }
        }
    }

    None
}
