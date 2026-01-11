use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use tungstenite::connect;
use url::Url;
pub struct SocketsModule;
impl SocketsModule {
    pub fn call_function(function_name: &str, args: &[Value]) -> MintasResult<Value> {
        match function_name {
            "connect" => Self::connect_socket(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown function '{}' in sockets module", function_name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn connect_socket(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "sockets.connect".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let url_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "URL must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let url = Url::parse(url_str).map_err(|e| MintasError::RuntimeError {
            message: format!("Invalid URL: {}", e),
            location: SourceLocation::new(0, 0),
        })?;
        match connect(url) {
            Ok((_socket, _response)) => {
                let mut result_map = HashMap::new();
                result_map.insert("connected".to_string(), Value::Boolean(true));
                result_map.insert("status".to_string(), Value::String("Connected successfully".to_string()));
                Ok(Value::Table(result_map))
            }
            Err(e) => Err(MintasError::RuntimeError {
                message: format!("WebSocket connection failed: {}", e),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
}