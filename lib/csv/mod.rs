use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct CsvModule;
impl CsvModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "parse" => Self::parse(args),
            "stringify" | "generate" => Self::stringify(args),
            "read" => Self::read_file(args),
            "write" => Self::write_file(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown csv function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn parse(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(content)) = args.get(0) {
            let has_header = match args.get(1) {
                Some(Value::Boolean(b)) => *b,
                _ => true,
            };
            let delimiter = match args.get(2) {
                Some(Value::String(s)) => s.chars().next().unwrap_or(','),
                _ => ',',
            };
            let lines: Vec<&str> = content.lines().collect();
            if lines.is_empty() {
                return Ok(Value::Array(vec![]));
            }
            if has_header && lines.len() > 1 {
                let headers: Vec<String> = Self::parse_line(lines[0], delimiter);
                let rows: Vec<Value> = lines[1..].iter().map(|line| {
                    let values = Self::parse_line(line, delimiter);
                    let mut row = HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        row.insert(header.clone(), Value::String(values.get(i).cloned().unwrap_or_default()));
                    }
                    Value::Table(row)
                }).collect();
                Ok(Value::Array(rows))
            } else {
                let rows: Vec<Value> = lines.iter().map(|line| {
                    let values: Vec<Value> = Self::parse_line(line, delimiter)
                        .into_iter().map(Value::String).collect();
                    Value::Array(values)
                }).collect();
                Ok(Value::Array(rows))
            }
        } else {
            Ok(Value::Array(vec![]))
        }
    }
    fn stringify(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::Array(rows)) = args.get(0) {
            let delimiter = match args.get(1) {
                Some(Value::String(s)) => s.chars().next().unwrap_or(','),
                _ => ',',
            };
            let mut lines = Vec::new();
            let mut headers: Option<Vec<String>> = None;
            for row in rows {
                match row {
                    Value::Table(map) => {
                        if headers.is_none() {
                            let h: Vec<String> = map.keys().cloned().collect();
                            lines.push(h.join(&delimiter.to_string()));
                            headers = Some(h);
                        }
                        if let Some(ref h) = headers {
                            let values: Vec<String> = h.iter().map(|key| {
                                match map.get(key) {
                                    Some(Value::String(s)) => Self::escape_csv(s, delimiter),
                                    Some(Value::Number(n)) => n.to_string(),
                                    Some(Value::Boolean(b)) => b.to_string(),
                                    _ => String::new(),
                                }
                            }).collect();
                            lines.push(values.join(&delimiter.to_string()));
                        }
                    }
                    Value::Array(arr) => {
                        let values: Vec<String> = arr.iter().map(|v| {
                            match v {
                                Value::String(s) => Self::escape_csv(s, delimiter),
                                Value::Number(n) => n.to_string(),
                                Value::Boolean(b) => b.to_string(),
                                _ => String::new(),
                            }
                        }).collect();
                        lines.push(values.join(&delimiter.to_string()));
                    }
                    _ => {}
                }
            }
            Ok(Value::String(lines.join("\n")))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn read_file(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(path)) = args.get(0) {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let mut new_args = vec![Value::String(content)];
                    new_args.extend(args[1..].to_vec());
                    Self::parse(&new_args)
                }
                Err(e) => Err(MintasError::RuntimeError {
                    message: format!("Failed to read CSV file: {}", e),
                    location: SourceLocation::new(0, 0),
                }),
            }
        } else {
            Ok(Value::Array(vec![]))
        }
    }
    fn write_file(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Ok(Value::Boolean(false));
        }
        if let (Some(Value::String(path)), Some(data)) = (args.get(0), args.get(1)) {
            let csv_content = match Self::stringify(&[data.clone()])? {
                Value::String(s) => s,
                _ => return Ok(Value::Boolean(false)),
            };
            match std::fs::write(path, csv_content) {
                Ok(_) => Ok(Value::Boolean(true)),
                Err(_) => Ok(Value::Boolean(false)),
            }
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn parse_line(line: &str, delimiter: char) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '"' {
                if in_quotes && chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            } else if c == delimiter && !in_quotes {
                result.push(current.trim().to_string());
                current = String::new();
            } else {
                current.push(c);
            }
        }
        result.push(current.trim().to_string());
        result
    }
    fn escape_csv(s: &str, delimiter: char) -> String {
        if s.contains(delimiter) || s.contains('"') || s.contains('\n') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    }
}