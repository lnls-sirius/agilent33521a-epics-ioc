mod frequency;
mod function;
mod phase;
mod voltage;

use super::ScpiRequest;
use super::str_extensions::StrExtensions;

pub fn decode(string: &str) -> Option<ScpiRequest> {
    let request_data = string.skip_expected_chars("SOURce");

    if let Some((source, command)) = request_data.parse_integer() {
        if command.starts_with(":") {
            let command = command.skip_chars(1);

            match command.view_first_chars(4) {
                "FREQ" => return frequency::decode(command, source),
                "FUNC" => return function::decode(command, source),
                "PHAS" => return phase::decode(command, source),
                "VOLT" => return voltage::decode(command, source),
                _ => {}
            }
        }
    }

    None
}
