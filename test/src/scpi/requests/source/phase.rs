use super::super::ScpiRequest;
use super::super::str_extensions::StrExtensions;

pub fn decode(string: &str, source: usize) -> Option<ScpiRequest> {
    let command = string.skip_expected_chars("PHASe");

    if command.starts_with("?") {
        return Some(ScpiRequest::SourcePhaseGet(source));
    }

    None
}
