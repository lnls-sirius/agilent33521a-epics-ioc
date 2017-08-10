mod arbitrary;
mod noise;
mod prbs;
mod pulse;
mod ramp;

use super::super::ScpiRequest;
use super::super::str_extensions::StrExtensions;

pub fn decode(string: &str, source: usize) -> Option<ScpiRequest> {
    let command = string.skip_expected_chars("FUNCtion");

    if command.starts_with(":") {
        let command = command.skip_chars(1);

        match command.view_first_chars(3) {
            "ARB" => return arbitrary::decode(command, source),
            _ => {}
        }

        match command.view_first_chars(4) {
            "NOIS" => return noise::decode(command, source),
            "PRBS" => return prbs::decode(command, source),
            "PULS" => return pulse::decode(command, source),
            "RAMP" => return ramp::decode(command, source),
            _ => {}
        }
    }

    None
}
