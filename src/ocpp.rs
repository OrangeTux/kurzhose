use serde_json::{Map, Value};
use std::fmt;

#[derive(Debug)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseError")
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(_: serde_json::Error) -> Self {
        ParseError
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Call {
        unique_id: String,
        action: String,
        data: Map<String, Value>,
    },
    CallResult {
        unique_id: String,
        data: Map<String, Value>,
    },
    CallError {
        unique_id: String,
        error_code: String,
        error_description: String,
    },
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Message::Call {
                unique_id,
                action,
                data,
            } => write!(f, "[2, {}, {}, {:?}]", unique_id, action, data),
            Message::CallResult { unique_id, data } => write!(f, "[3, {}, {:?}]", unique_id, data),
            Message::CallError {
                unique_id,
                error_code,
                error_description,
            } => write!(
                f,
                "[3, {}, {}, {}]",
                unique_id, error_code, error_description
            ),
        }
    }
}
