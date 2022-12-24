use std::fmt::Display;

/// Errors used in hopper_macros.
#[derive(Debug)]
pub enum Error {
    /// Used when an invalid struct (enum or with unnamed fields) gets parsed.
    InvalidStructErr,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid Struct Error (enum or unnamed fields)")
    }
}

impl std::error::Error for Error {}
