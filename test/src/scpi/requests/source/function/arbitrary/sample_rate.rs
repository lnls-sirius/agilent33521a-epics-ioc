use super::super::super::super::ScpiRequest;
use super::super::super::super::str_extensions::StrExtensions;

pub struct Builder {
    source: usize,
}

impl Builder {
    pub fn with_source(source: usize) -> Builder {
        Builder { source }
    }

    pub fn get(self) -> ScpiRequest {
        ScpiRequest::SourceVoltageGet(self.source)
    }
}

pub fn try_decode(string: &str, source: usize) -> Option<ScpiRequest> {
    let command = string.skip_expected_chars("SRATe");

    if command.starts_with("?") {
        return Some(ScpiRequest::SourceArbitraryFunctionSampleRateGet(source));
    }

    None
}
