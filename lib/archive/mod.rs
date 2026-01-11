use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct ArchiveModule;
impl ArchiveModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "create" => Self::create(args),
            "add" => Self::add(args),
            "extract" => Self::extract(args),
            "list" => Self::list(args),
            "save" => Self::save(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown archive function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn create(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => "archive.tar".to_string() };
        let mut archive = HashMap::new();
        archive.insert("name".to_string(), Value::String(name));
        archive.insert("files".to_string(), Value::Array(vec![]));
        archive.insert("__type__".to_string(), Value::String("Archive".to_string()));
        Ok(Value::Table(archive))
    }
    fn add(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::Table(mut archive)) = args.get(0).cloned() {
            let path = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Table(archive)) };
            let content = match args.get(2) { Some(Value::String(s)) => s.clone(), _ => String::new() };
            let mut file = HashMap::new();
            file.insert("path".to_string(), Value::String(path));
            file.insert("content".to_string(), Value::String(content.clone()));
            file.insert("size".to_string(), Value::Number(content.len() as f64));
            if let Some(Value::Array(ref mut files)) = archive.get_mut("files") {
                files.push(Value::Table(file));
            }
            return Ok(Value::Table(archive));
        }
        Ok(Value::Empty)
    }
    fn extract(args: &[Value]) -> MintasResult<Value> {
        let path = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let mut result = HashMap::new();
        result.insert("path".to_string(), Value::String(path));
        result.insert("files".to_string(), Value::Array(vec![]));
        Ok(Value::Table(result))
    }
    fn list(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::Table(archive)) = args.get(0) {
            if let Some(Value::Array(files)) = archive.get("files") {
                let names: Vec<Value> = files.iter().filter_map(|f| {
                    if let Value::Table(file) = f {
                        file.get("path").cloned()
                    } else { None }
                }).collect();
                return Ok(Value::Array(names));
            }
        }
        Ok(Value::Array(vec![]))
    }
    fn save(args: &[Value]) -> MintasResult<Value> {
        let archive = match args.get(0) { Some(Value::Table(a)) => a.clone(), _ => return Ok(Value::Boolean(false)) };
        let _path = match args.get(1) { 
            Some(Value::String(s)) => s.clone(), 
            _ => archive.get("name").and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None }).unwrap_or("archive.tar".to_string())
        };
        Ok(Value::Boolean(true))
    }
}