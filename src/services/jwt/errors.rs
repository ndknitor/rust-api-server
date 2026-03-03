use std::fmt;

#[derive(Debug)]
pub enum JWTServiceError {
    SignFailed,
}

impl fmt::Display for JWTServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SignFailed")
    }
}

impl std::error::Error for JWTServiceError {}
