use std::{fmt, error};

#[derive(Debug, Clone)]
pub struct UrlError;

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Url Error")
    }
}

impl error::Error for UrlError {}
