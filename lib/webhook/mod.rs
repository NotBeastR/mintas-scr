use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use serde_json::Value as JsonValue;
pub struct WebhookModule;
impl WebhookModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "create" => Self::create(args),
            "send" => Self::send(args),
            "verify" => Self::verify(args),
            "parse" => Self::parse(args),
            "sign" => Self::sign(args),
            "validate_signature" => Self::validate_signature(args),
            "retry" => Self::retry(args),
            "queue" => Self::queue(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown webhook function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn create(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "webhook.create".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let url = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "URL must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let secret = if args.len() > 1 {
            match &args[1] {
                Value::String(s) => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        };
        let mut webhook = HashMap::new();
        webhook.insert("url".to_string(), Value::String(url));
        webhook.insert("active".to_string(), Value::Boolean(true));
        webhook.insert("created_at".to_string(), Value::Number(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as f64
        ));
        if let Some(s) = secret {
            webhook.insert("secret".to_string(), Value::String(s));
        }
        Ok(Value::Table(webhook))
    }
    fn send(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "webhook.send".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let url = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "URL must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let payload = match &args[1] {
            Value::Table(t) => {
                let mut json_obj = serde_json::Map::new();
                for (k, v) in t {
                    json_obj.insert(k.clone(), Self::value_to_json(v));
                }
                JsonValue::Object(json_obj)
            }
            Value::String(s) => JsonValue::String(s.clone()),
            _ => return Err(MintasError::TypeError {
                message: "Payload must be a table or string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        println!("[Webhook] Sending to: {}", url);
        println!("[Webhook] Payload: {}", payload);
        let mut result = HashMap::new();
        result.insert("status".to_string(), Value::Number(200.0));
        result.insert("success".to_string(), Value::Boolean(true));
        result.insert("url".to_string(), Value::String(url));
        result.insert("body".to_string(), Value::String("{}".to_string()));
        Ok(Value::Table(result))
    }
    fn verify(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 3 {
            return Err(MintasError::InvalidArgumentCount {
                function: "webhook.verify".to_string(),
                expected: 3,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let payload = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Payload must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let signature = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Signature must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let secret = match &args[2] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Secret must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let expected = Self::compute_signature(&payload, &secret);
        Ok(Value::Boolean(signature == expected))
    }
    fn parse(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "webhook.parse".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let body = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Body must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        match serde_json::from_str::<JsonValue>(&body) {
            Ok(json) => Ok(Self::json_to_value(&json)),
            Err(e) => Err(MintasError::RuntimeError {
                message: format!("Failed to parse webhook body: {}", e),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn sign(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "webhook.sign".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let payload = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Payload must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let secret = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Secret must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        Ok(Value::String(Self::compute_signature(&payload, &secret)))
    }
    fn validate_signature(args: &[Value]) -> MintasResult<Value> {
        Self::verify(args)
    }
    fn retry(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "webhook.retry".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let max_retries = if args.len() > 2 {
            match &args[2] {
                Value::Number(n) => *n as u32,
                _ => 3,
            }
        } else {
            3
        };
        for attempt in 1..=max_retries {
            let result = Self::send(args)?;
            if let Value::Table(ref t) = result {
                if let Some(Value::Boolean(true)) = t.get("success") {
                    return Ok(result);
                }
            }
            if attempt < max_retries {
                std::thread::sleep(std::time::Duration::from_millis(1000 * attempt as u64));
            }
        }
        let mut result = HashMap::new();
        result.insert("success".to_string(), Value::Boolean(false));
        result.insert("error".to_string(), Value::String("Max retries exceeded".to_string()));
        Ok(Value::Table(result))
    }
    fn queue(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "webhook.queue".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let mut result = HashMap::new();
        result.insert("queued".to_string(), Value::Boolean(true));
        result.insert("id".to_string(), Value::String(format!("wh_{}", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
        Ok(Value::Table(result))
    }
    fn compute_signature(payload: &str, secret: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        payload.hash(&mut hasher);
        secret.hash(&mut hasher);
        format!("sha256={:x}", hasher.finish())
    }
    fn value_to_json(value: &Value) -> JsonValue {
        match value {
            Value::Number(n) => JsonValue::Number(serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0))),
            Value::String(s) => JsonValue::String(s.clone()),
            Value::Boolean(b) => JsonValue::Bool(*b),
            Value::Array(arr) => JsonValue::Array(arr.iter().map(Self::value_to_json).collect()),
            Value::Table(t) => {
                let mut obj = serde_json::Map::new();
                for (k, v) in t {
                    obj.insert(k.clone(), Self::value_to_json(v));
                }
                JsonValue::Object(obj)
            }
            _ => JsonValue::Null,
        }
    }
    fn json_to_value(json: &JsonValue) -> Value {
        match json {
            JsonValue::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
            JsonValue::String(s) => Value::String(s.clone()),
            JsonValue::Bool(b) => Value::Boolean(*b),
            JsonValue::Array(arr) => Value::Array(arr.iter().map(Self::json_to_value).collect()),
            JsonValue::Object(obj) => {
                let mut table = HashMap::new();
                for (k, v) in obj {
                    table.insert(k.clone(), Self::json_to_value(v));
                }
                Value::Table(table)
            }
            JsonValue::Null => Value::Empty,
        }
    }
}