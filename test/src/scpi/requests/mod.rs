use super::errors::{ErrorKind, Result};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ScpiRequest {}

impl ScpiRequest {
    pub fn from(string: &str) -> Result<ScpiRequest> {
        Err(ErrorKind::UnknownScpiRequest(String::from(string)).into())
    }
}
