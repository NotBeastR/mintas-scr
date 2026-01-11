use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct MyyamlModule;
impl MyyamlModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "parse" | "load" => Self::parse(args),
            "stringify" | "dump" => Self::stringify(args),
            "get" => Self::get(args),
            "set" => Self::set(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown myyaml function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn parse(args: &[Value]) -> MintasResult<Value> {
        let yaml = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        Ok(Self::parse_yaml(&yaml))
    }
    fn parse_yaml(yaml: &str) -> Value {
        let mut result = HashMap::new();
        let _current_indent = 0;
        let _stack: Vec<(usize, String, HashMap<String, Value>)> = vec![];
        for line in yaml.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
            let _indent = line.len() - line.trim_start().len();
            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim().to_string();
                let value_str = trimmed[colon_pos + 1..].trim();
                let value = if value_str.is_empty() {
                    Value::Table(HashMap::new())
                } else if value_str == "true" {
                    Value::Boolean(true)
                } else if value_str == "false" {
                    Value::Boolean(false)
                } else if let Ok(n) = value_str.parse::<f64>() {
                    Value::Number(n)
                } else {
                    Value::String(value_str.trim_matches('"').trim_matches('\'').to_string())
                };
                result.insert(key, value);
            } else if trimmed.starts_with("- ") {
                let item = trimmed[2..].trim();
                let _value = if let Ok(n) = item.parse::<f64>() {
                    Value::Number(n)
                } else {
                    Value::String(item.to_string())
                };
            }
        }
        Value::Table(result)
    }
    fn stringify(args: &[Value]) -> MintasResult<Value> {
        let value = args.get(0).cloned().unwrap_or(Value::Empty);
        let indent = match args.get(1) { Some(Value::Number(n)) => *n as usize, _ => 2 };
        Ok(Value::String(Self::value_to_yaml(&value, 0, indent)))
    }
    fn value_to_yaml(value: &Value, depth: usize, indent: usize) -> String {
        let prefix = " ".repeat(depth * indent);
        match value {
            Value::Table(map) => {
                let mut result = String::new();
                for (k, v) in map {
                    match v {
                        Value::Table(_) | Value::Array(_) => {
                            result.push_str(&format!("{}{}:\n{}", prefix, k, Self::value_to_yaml(v, depth + 1, indent)));
                        }
                        _ => {
                            result.push_str(&format!("{}{}: {}\n", prefix, k, Self::value_to_yaml(v, 0, indent)));
                        }
                    }
                }
                result
            }
            Value::Array(arr) => {
                let mut result = String::new();
                for item in arr {
                    result.push_str(&format!("{}- {}\n", prefix, Self::value_to_yaml(item, 0, indent).trim()));
                }
                result
            }
            Value::String(s) => {
                if s.contains('\n') || s.contains(':') || s.contains('#') {
                    format!("\"{}\"", s.replace('"', "\\\""))
                } else {
                    s.clone()
                }
            }
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            _ => "null".to_string(),
        }
    }
    fn get(args: &[Value]) -> MintasResult<Value> {
        let data = match args.get(0) { Some(Value::Table(t)) => t.clone(), _ => return Ok(Value::Empty) };
        let key = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        Ok(data.get(&key).cloned().unwrap_or(Value::Empty))
    }
    fn set(args: &[Value]) -> MintasResult<Value> {
        let mut data = match args.get(0) { Some(Value::Table(t)) => t.clone(), _ => HashMap::new() };
        let key = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Table(data)) };
        let value = args.get(2).cloned().unwrap_or(Value::Empty);
        data.insert(key, value);
        Ok(Value::Table(data))
    }
}