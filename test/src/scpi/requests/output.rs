use super::ScpiRequest;
use super::str_extensions::StrExtensions;

pub fn decode(string: &str) -> Option<ScpiRequest> {
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
