use crate::errors::{MintasError, MintasResult};
use crate::evaluator::Value;
use serde_json::{Value as JsonValue, Map as JsonMap};
pub struct JsonModule;
impl JsonModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "encode" => Self::encode(args),
            "decode" => Self::decode(args),
            "pretty" => Self::pretty(args),
            "stringify" => Self::stringify(args),
            "parse" => Self::parse(args),
            "get" => Self::get(args),
            "set" => Self::set(args),
            "keys" => Self::keys(args),
            "values" => Self::values(args),
            "has_key" => Self::has_key(args),
            "is_valid" => Self::is_valid(args),
            "merge" => Self::merge(args),
            "to_table" => Self::to_table(args),
            "from_table" => Self::from_table(args),
            _ => Err(MintasError::UnknownFunction {
                name: format!("json.{}", name),
                location: crate::errors::SourceLocation::new(0, 0),
            }),
        }
    }
    fn expect_string_arg(args: &[Value], index: usize, func_name: &str) -> MintasResult<String> {
        if index >= args.len() {
            return Err(MintasError::InvalidArgumentCount {
                function: format!("json.{}", func_name),
                expected: index + 1,
                got: args.len(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        match &args[index] {
            Value::String(s) => Ok(s.clone()),
            _ => Err(MintasError::TypeError {
                message: format!("json.{} expects a string for argument {}", func_name, index + 1),
                location: crate::errors::SourceLocation::new(0, 0),
            }),
        }
    }
    fn expect_table_arg(args: &[Value], index: usize, func_name: &str) -> MintasResult<std::collections::HashMap<String, Value>> {
        if index >= args.len() {
            return Err(MintasError::InvalidArgumentCount {
                function: format!("json.{}", func_name),
                expected: index + 1,
                got: args.len(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        match &args[index] {
            Value::Table(map) => Ok(map.clone()),
            _ => Err(MintasError::TypeError {
                message: format!("json.{} expects a table for argument {}", func_name, index + 1),
                location: crate::errors::SourceLocation::new(0, 0),
            }),
        }
    }
    fn encode(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "json.encode".to_string(),
                expected: 1,
                got: 0,
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        let json_value = Self::mintas_to_json(&args[0])?;
        let json_string = serde_json::to_string(&json_value)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON encoding error: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        Ok(Value::String(json_string))
    }
    fn decode(args: &[Value]) -> MintasResult<Value> {
        let json_string = Self::expect_string_arg(args, 0, "decode")?;
        let json_value: JsonValue = serde_json::from_str(&json_string)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON parsing error: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        Self::json_to_mintas(&json_value)
    }
    fn pretty(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "json.pretty".to_string(),
                expected: 1,
                got: 0,
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        let json_value = Self::mintas_to_json(&args[0])?;
        let json_string = serde_json::to_string_pretty(&json_value)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON pretty encoding error: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        Ok(Value::String(json_string))
    }
    fn stringify(args: &[Value]) -> MintasResult<Value> {
        Self::encode(args)
    }
    fn parse(args: &[Value]) -> MintasResult<Value> {
        Self::decode(args)
    }
    fn get(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "json.get".to_string(),
                expected: 2,
                got: args.len(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        let json_string = Self::expect_string_arg(args, 0, "get")?;
        let key_path = Self::expect_string_arg(args, 1, "get")?;
        let json_value: JsonValue = serde_json::from_str(&json_string)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON parsing error: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let mut current = &json_value;
        for key in key_path.split('.') {
            match current {
                JsonValue::Object(map) => {
                    current = map.get(key).ok_or_else(|| MintasError::RuntimeError {
                        message: format!("Key '{}' not found in JSON", key),
                        location: crate::errors::SourceLocation::new(0, 0),
                    })?;
                }
                JsonValue::Array(arr) => {
                    let index: usize = key.parse().map_err(|_| MintasError::RuntimeError {
                        message: format!("Invalid array index: {}", key),
                        location: crate::errors::SourceLocation::new(0, 0),
                    })?;
                    current = arr.get(index).ok_or_else(|| MintasError::RuntimeError {
                        message: format!("Array index {} out of bounds", index),
                        location: crate::errors::SourceLocation::new(0, 0),
                    })?;
                }
                _ => return Err(MintasError::RuntimeError {
                    message: format!("Cannot access '{}' on non-object/array", key),
                    location: crate::errors::SourceLocation::new(0, 0),
                }),
            }
        }
        Self::json_to_mintas(current)
    }
    fn set(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 3 {
            return Err(MintasError::InvalidArgumentCount {
                function: "json.set".to_string(),
                expected: 3,
                got: args.len(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        let json_string = Self::expect_string_arg(args, 0, "set")?;
        let key_path = Self::expect_string_arg(args, 1, "set")?;
        let new_value = &args[2];
        let mut json_value: JsonValue = serde_json::from_str(&json_string)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON parsing error: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        if key_path.contains('.') {
            return Err(MintasError::RuntimeError {
                message: "Nested key paths not yet supported in json.set".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        let new_json_value = Self::mintas_to_json(new_value)?;
        if let JsonValue::Object(ref mut map) = json_value {
            map.insert(key_path, new_json_value);
        } else {
            return Err(MintasError::RuntimeError {
                message: "json.set requires a JSON object".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        let result_string = serde_json::to_string(&json_value)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON encoding error: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        Ok(Value::String(result_string))
    }
    fn keys(args: &[Value]) -> MintasResult<Value> {
        let json_string = Self::expect_string_arg(args, 0, "keys")?;
        let json_value: JsonValue = serde_json::from_str(&json_string)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON parsing error: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        if let JsonValue::Object(map) = json_value {
            let keys: Vec<Value> = map.keys().map(|k| Value::String(k.clone())).collect();
            Ok(Value::Array(keys))
        } else {
            Err(MintasError::RuntimeError {
                message: "json.keys requires a JSON object".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })
        }
    }
    fn values(args: &[Value]) -> MintasResult<Value> {
        let json_string = Self::expect_string_arg(args, 0, "values")?;
        let json_value: JsonValue = serde_json::from_str(&json_string)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON parsing error: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        if let JsonValue::Object(map) = json_value {
            let values: MintasResult<Vec<Value>> = map.values()
                .map(|v| Self::json_to_mintas(v))
                .collect();
            Ok(Value::Array(values?))
        } else {
            Err(MintasError::RuntimeError {
                message: "json.values requires a JSON object".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })
        }
    }
    fn has_key(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "json.has_key".to_string(),
                expected: 2,
                got: args.len(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        let json_string = Self::expect_string_arg(args, 0, "has_key")?;
        let key = Self::expect_string_arg(args, 1, "has_key")?;
        let json_value: JsonValue = serde_json::from_str(&json_string)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON parsing error: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let has_key = matches!(json_value, JsonValue::Object(ref map) if map.contains_key(&key));
        Ok(Value::Boolean(has_key))
    }
    fn is_valid(args: &[Value]) -> MintasResult<Value> {
        let json_string = Self::expect_string_arg(args, 0, "is_valid")?;
        let is_valid = serde_json::from_str::<JsonValue>(&json_string).is_ok();
        Ok(Value::Boolean(is_valid))
    }
    fn merge(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "json.merge".to_string(),
                expected: 2,
                got: args.len(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        let json1_string = Self::expect_string_arg(args, 0, "merge")?;
        let json2_string = Self::expect_string_arg(args, 1, "merge")?;
        let json1: JsonValue = serde_json::from_str(&json1_string)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON parsing error for first argument: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let json2: JsonValue = serde_json::from_str(&json2_string)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON parsing error for second argument: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        if let (JsonValue::Object(mut map1), JsonValue::Object(map2)) = (json1, json2) {
            for (k, v) in map2 {
                map1.insert(k, v);
            }
            let merged = JsonValue::Object(map1);
            let result_string = serde_json::to_string(&merged)
                .map_err(|e| MintasError::RuntimeError {
                    message: format!("JSON encoding error: {}", e),
                    location: crate::errors::SourceLocation::new(0, 0),
                })?;
            Ok(Value::String(result_string))
        } else {
            Err(MintasError::RuntimeError {
                message: "json.merge requires two JSON objects".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })
        }
    }
    fn to_table(args: &[Value]) -> MintasResult<Value> {
        let json_string = Self::expect_string_arg(args, 0, "to_table")?;
        let json_value: JsonValue = serde_json::from_str(&json_string)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON parsing error: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        if let JsonValue::Object(map) = json_value {
            let mut table = std::collections::HashMap::new();
            for (k, v) in map {
                table.insert(k, Self::json_to_mintas(&v)?);
            }
            Ok(Value::Table(table))
        } else {
            Err(MintasError::RuntimeError {
                message: "json.to_table requires a JSON object".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })
        }
    }
    fn from_table(args: &[Value]) -> MintasResult<Value> {
        let table = Self::expect_table_arg(args, 0, "from_table")?;
        let json_value = Self::mintas_to_json(&Value::Table(table))?;
        let json_string = serde_json::to_string(&json_value)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("JSON encoding error: {}", e),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        Ok(Value::String(json_string))
    }
    fn mintas_to_json(value: &Value) -> MintasResult<JsonValue> {
        match value {
            Value::Number(n) => Ok(JsonValue::Number(serde_json::Number::from_f64(*n)
                .ok_or_else(|| MintasError::RuntimeError {
                    message: "Invalid number for JSON".to_string(),
                    location: crate::errors::SourceLocation::new(0, 0),
                })?)),
            Value::String(s) => Ok(JsonValue::String(s.clone())),
            Value::Boolean(b) => Ok(JsonValue::Bool(*b)),
            Value::Array(arr) => {
                let json_arr: MintasResult<Vec<JsonValue>> = arr.iter()
                    .map(|v| Self::mintas_to_json(v))
                    .collect();
                Ok(JsonValue::Array(json_arr?))
            }
            Value::Table(map) => {
                let mut json_map = JsonMap::new();
                for (k, v) in map {
                    json_map.insert(k.clone(), Self::mintas_to_json(v)?);
                }
                Ok(JsonValue::Object(json_map))
            }
            Value::Empty => Ok(JsonValue::Null),
            _ => Ok(JsonValue::String(format!("{:?}", value))),
        }
    }
    fn json_to_mintas(json_value: &JsonValue) -> MintasResult<Value> {
        match json_value {
            JsonValue::Null => Ok(Value::Empty),
            JsonValue::Bool(b) => Ok(Value::Boolean(*b)),
            JsonValue::Number(n) => Ok(Value::Number(n.as_f64().unwrap_or(0.0))),
            JsonValue::String(s) => Ok(Value::String(s.clone())),
            JsonValue::Array(arr) => {
                let mintas_arr: MintasResult<Vec<Value>> = arr.iter()
                    .map(|v| Self::json_to_mintas(v))
                    .collect();
                Ok(Value::Array(mintas_arr?))
            }
            JsonValue::Object(map) => {
                let mut mintas_map = std::collections::HashMap::new();
                for (k, v) in map {
                    mintas_map.insert(k.clone(), Self::json_to_mintas(v)?);
                }
                Ok(Value::Table(mintas_map))
            }
        }
    }
}