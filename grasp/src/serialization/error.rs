// Boilerplate provided by serde

use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, SerializationError>;

#[derive(Debug)]
pub enum SerializationError {
    Message(String),
}

impl ser::Error for SerializationError {
    fn custom<T: Display>(msg: T) -> Self {
        SerializationError::Message(msg.to_string())
    }
}

impl de::Error for SerializationError {
    fn custom<T: Display>(msg: T) -> Self {
        SerializationError::Message(msg.to_string())
    }
}

impl Display for SerializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SerializationError::Message(msg) => formatter.write_str(msg),
        }
    }
}

impl std::error::Error for SerializationError {}
