use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::env;
pub struct EnvModule;
impl EnvModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "get" => Self::get(args),
            "set" => Self::set(args),
            "remove" | "unset" => Self::remove(args),
            "has" => Self::has(args),
            "all" => Self::all(),
            "keys" => Self::keys(),
            "load" => Self::load_dotenv(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown env function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn get(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(key)) = args.get(0) {
            let default = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            Ok(Value::String(env::var(key).unwrap_or(default)))
        } else {
            Err(MintasError::TypeError {
                message: "env.get requires a string key".to_string(),
                location: SourceLocation::new(0, 0),
            })
        }
    }
    fn set(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "env.set".to_string(), expected: 2, got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        if let (Some(Value::String(key)), Some(Value::String(val))) = (args.get(0), args.get(1)) {
            env::set_var(key, val);
            Ok(Value::Boolean(true))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn remove(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(key)) = args.get(0) {
            env::remove_var(key);
            Ok(Value::Boolean(true))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn has(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(key)) = args.get(0) {
            Ok(Value::Boolean(env::var(key).is_ok()))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn all() -> MintasResult<Value> {
        let mut result = HashMap::new();
        for (key, val) in env::vars() {
            result.insert(key, Value::String(val));
        }
        Ok(Value::Table(result))
    }
    fn keys() -> MintasResult<Value> {
        let keys: Vec<Value> = env::vars().map(|(k, _)| Value::String(k)).collect();
        Ok(Value::Array(keys))
    }
    fn load_dotenv(args: &[Value]) -> MintasResult<Value> {
        let path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => ".env".to_string(),
        };
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let mut count = 0;
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') { continue; }
                    if let Some(eq_pos) = line.find('=') {
                        let key = line[..eq_pos].trim();
                        let val = line[eq_pos + 1..].trim().trim_matches('"');
                        env::set_var(key, val);
                        count += 1;
                    }
                }
                Ok(Value::Number(count as f64))
            }
            Err(_) => Ok(Value::Number(0.0)),
        }
    }
}