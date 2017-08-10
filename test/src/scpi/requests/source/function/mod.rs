mod arbitrary;

use super::super::ScpiRequest;
use super::super::str_extensions::StrExtensions;

pub fn decode(string: &str, source: usize) -> Option<ScpiRequest> {
    let command = string.skip_expected_chars("FUNCtion");

    if command.starts_with(":") {
        let command = command.skip_chars(1);
        let first_three_chars = command.view_first_chars(3);

        match first_three_chars {
            "ARB" => return arbitrary::decode(command, source),
            _ => {}
        }
    }

    None
}
