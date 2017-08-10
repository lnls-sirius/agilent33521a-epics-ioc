mod leading;
mod trailing;

use super::super::super::ScpiRequest;
use super::super::super::super::str_extensions::StrExtensions;

pub fn decode(string: &str, source: usize) -> Option<ScpiRequest> {
    let command = string.skip_expected_chars("TRANsition");

    if command.starts_with(":") {
        let command = command.skip_chars(1);

        match command.view_first_chars(3) {
            "TRA" => return trailing::decode(command, source),
            _ => {}
        }

        match command.view_first_chars(4) {
            "LEAD" => return leading::decode(command, source),
            _ => {}
        }
    }

    None
}
