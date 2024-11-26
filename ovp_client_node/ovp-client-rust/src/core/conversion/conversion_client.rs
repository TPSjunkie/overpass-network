use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientConversionError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientConversionResult {
    pub result: Option<Value>,
    pub error: Option<ClientConversionError>,
}

impl ClientConversionResult {
    pub fn new(result: Option<Value>, error: Option<ClientConversionError>) -> Self {
        Self { result, error }
    }

    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }

    pub fn is_err(&self) -> bool {
        self.error.is_some()
    }

    pub fn unwrap(self) -> Value {
        self.result.expect("Called unwrap on a None result")
    }

    pub fn unwrap_err(self) -> ClientConversionError {
        self.error.expect("Called unwrap_err on a None error")
    }
}

pub trait ClientConversion {
    fn convert_to_client(self) -> ClientConversionResult;
}

impl ClientConversion for Value {
    fn convert_to_client(self) -> ClientConversionResult {
        match self {
            Value::Null => ClientConversionResult::new(Some(Value::Null), None),
            Value::Bool(value) => ClientConversionResult::new(Some(Value::Bool(value)), None),
            Value::Number(value) => ClientConversionResult::new(Some(Value::Number(value)), None),
            Value::String(value) => ClientConversionResult::new(Some(Value::String(value)), None),
            Value::Array(value) => ClientConversionResult::new(Some(Value::Array(value)), None),
            Value::Object(value) => ClientConversionResult::new(Some(Value::Object(value)), None),
        }
    }
}
