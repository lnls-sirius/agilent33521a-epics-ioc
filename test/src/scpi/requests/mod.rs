use std::fmt;
use std::fmt::{Display, Formatter};

use super::errors::{ErrorKind, Result};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ScpiRequest {}

impl ScpiRequest {
    pub fn from(string: &str) -> Result<ScpiRequest> {
        Err(ErrorKind::UnknownScpiRequest(String::from(string)).into())
    }
}

impl Display for ScpiRequest {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Ok(())
    }
}
