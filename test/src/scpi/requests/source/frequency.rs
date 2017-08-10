use super::super::ScpiRequest;
use super::super::str_extensions::StrExtensions;

pub fn decode(string: &str, source: usize) -> Option<ScpiRequest> {
    let command = string.skip_expected_chars("FREQuency");

    if command.starts_with("?") {
        return Some(ScpiRequest::SourceFrequencyGet(source));
    }

    None
}
