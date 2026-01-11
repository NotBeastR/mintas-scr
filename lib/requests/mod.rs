use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct RequestsModule;
impl RequestsModule {
    pub fn call_function(function_name: &str, args: &[Value]) -> MintasResult<Value> {
        match function_name {
            "get" => Self::get(args),
            "post" => Self::post(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown function '{}' in requests module", function_name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn get(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "requests.get".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let url = match &args[0] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "URL must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        match reqwest::blocking::get(url) {
            Ok(response) => {
                let status = response.status().as_u16() as f64;
                let text = response.text().unwrap_or_default();
                let mut result_map = HashMap::new();
                result_map.insert("status".to_string(), Value::Number(status));
                result_map.insert("text".to_string(), Value::String(text));
                Ok(Value::Table(result_map))
            }
            Err(e) => Err(MintasError::RuntimeError {
                message: format!("Request failed: {}", e),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn post(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 1 {
            return Err(MintasError::InvalidArgumentCount {
                function: "requests.post".to_string(),
                expected: 1,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let url = match &args[0] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "URL must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let client = reqwest::blocking::Client::new();
        let mut request = client.post(url);
        if args.len() > 1 {
            if let Value::Table(data) = &args[1] {
                let mut json_map = HashMap::new();
                for (k, v) in data {
                    if let Value::String(s) = v {
                        json_map.insert(k.clone(), s.clone());
                    }
                }
                request = request.json(&json_map);
            }
        }
        match request.send() {
            Ok(response) => {
                let status = response.status().as_u16() as f64;
                let text = response.text().unwrap_or_default();
                let mut result_map = HashMap::new();
                result_map.insert("status".to_string(), Value::Number(status));
                result_map.insert("text".to_string(), Value::String(text));
                Ok(Value::Table(result_map))
            }
            Err(e) => Err(MintasError::RuntimeError {
                message: format!("Request failed: {}", e),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
}