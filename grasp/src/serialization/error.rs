// Boilerplate provided by serde

use std;
use std::fmt::{self, Display};

#[cfg(feature = "serde")]
use serde::{de, ser};

#[cfg(feature = "serde")]
#[derive(Debug)]
pub struct SerializationError {
    pub message: String,
}

#[cfg(feature = "serde")]
impl ser::Error for SerializationError {
    fn custom<T: Display>(msg: T) -> Self {
        SerializationError { message: msg.to_string() }
    }
}

#[cfg(feature = "serde")]
impl de::Error for SerializationError {
    fn custom<T: Display>(msg: T) -> Self {
        SerializationError { message: msg.to_string() }
    }
}

#[cfg(feature = "serde")]
impl Display for SerializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SerializationError {message } => formatter.write_str(message),
        }
    }
}

#[cfg(feature = "serde")]
impl std::error::Error for SerializationError {}

#[derive(Debug)]
pub struct FormattingError {
    pub message: String,
}

impl Display for FormattingError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FormattingError {message } => formatter.write_str(message),
        }
    }
}
