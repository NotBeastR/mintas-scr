use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct GraphqlModule;
impl GraphqlModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "query" => Self::query(args),
            "mutation" => Self::mutation(args),
            "subscribe" => Self::subscribe(args),
            "client" => Self::client(args),
            "build_query" => Self::build_query(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown graphql function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn client(args: &[Value]) -> MintasResult<Value> {
        let url = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let mut client = HashMap::new();
        client.insert("url".to_string(), Value::String(url));
        client.insert("headers".to_string(), Value::Table(HashMap::new()));
        client.insert("__type__".to_string(), Value::String("GraphQLClient".to_string()));
        Ok(Value::Table(client))
    }
    fn query(args: &[Value]) -> MintasResult<Value> {
        let _url = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let query = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let variables = match args.get(2) { Some(Value::Table(t)) => t.clone(), _ => HashMap::new() };
        let mut body = HashMap::new();
        body.insert("query".to_string(), Value::String(query));
        if !variables.is_empty() {
            body.insert("variables".to_string(), Value::Table(variables));
        }
        #[cfg(feature = "networking")]
        {
            let body_json = Self::to_json(&Value::Table(body));
            match reqwest::blocking::Client::new()
                .post(&url)
                .header("Content-Type", "application/json")
                .body(body_json)
                .send() {
                Ok(resp) => {
                    let text = resp.text().unwrap_or_default();
                    return Ok(Self::parse_json(&text));
                }
                Err(e) => return Err(MintasError::RuntimeError {
                    message: format!("GraphQL request failed: {}", e),
                    location: SourceLocation::new(0, 0),
                }),
            }
        }
        #[cfg(not(feature = "networking"))]
        Ok(Value::Table(body))
    }
    fn mutation(args: &[Value]) -> MintasResult<Value> {
        Self::query(args)
    }
    fn subscribe(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Boolean(true))
    }
    fn build_query(args: &[Value]) -> MintasResult<Value> {
        let operation = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => "query".to_string() };
        let name = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => "".to_string() };
        let fields = match args.get(2) { Some(Value::Array(a)) => a.clone(), _ => vec![] };
        let fields_str: String = fields.iter().filter_map(|f| {
            if let Value::String(s) = f { Some(s.clone()) } else { None }
        }).collect::<Vec<_>>().join("\n  ");
        let query = if name.is_empty() {
            format!("{} {{\n  {}\n}}", operation, fields_str)
        } else {
            format!("{} {} {{\n  {}\n}}", operation, name, fields_str)
        };
        Ok(Value::String(query))
    }
    #[allow(dead_code)]
    fn to_json(value: &Value) -> String {
        match value {
            Value::Table(map) => {
                let pairs: Vec<String> = map.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, Self::to_json(v)))
                    .collect();
                format!("{{{}}}", pairs.join(","))
            }
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| Self::to_json(v)).collect();
                format!("[{}]", items.join(","))
            }
            Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            _ => "null".to_string(),
        }
    }
    #[allow(dead_code)]
    fn parse_json(json: &str) -> Value {
        let trimmed = json.trim();
        if trimmed.starts_with('{') {
            Value::Table(HashMap::new()) 
        } else if trimmed.starts_with('[') {
            Value::Array(vec![])
        } else if trimmed == "true" {
            Value::Boolean(true)
        } else if trimmed == "false" {
            Value::Boolean(false)
        } else if let Ok(n) = trimmed.parse::<f64>() {
            Value::Number(n)
        } else {
            Value::String(trimmed.trim_matches('"').to_string())
        }
    }
}