use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
pub struct PathModule;
impl PathModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "join" => Self::join(args),
            "dirname" | "dir" => Self::dirname(args),
            "basename" | "base" => Self::basename(args),
            "extname" | "ext" => Self::extname(args),
            "resolve" => Self::resolve(args),
            "normalize" => Self::normalize(args),
            "isabs" | "is_absolute" => Self::is_absolute(args),
            "exists" => Self::exists(args),
            "isfile" | "is_file" => Self::is_file(args),
            "isdir" | "is_dir" => Self::is_dir(args),
            "parse" => Self::parse(args),
            "sep" => Self::separator(),
            "relative" => Self::relative(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown path function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn join(args: &[Value]) -> MintasResult<Value> {
        let mut path = PathBuf::new();
        for arg in args {
            if let Value::String(s) = arg {
                path.push(s);
            }
        }
        Ok(Value::String(path.to_string_lossy().to_string()))
    }
    fn dirname(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(p)) = args.get(0) {
            let path = Path::new(p);
            Ok(Value::String(path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn basename(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(p)) = args.get(0) {
            let path = Path::new(p);
            Ok(Value::String(path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn extname(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(p)) = args.get(0) {
            let path = Path::new(p);
            Ok(Value::String(path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default()))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn resolve(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(p)) = args.get(0) {
            let path = Path::new(p);
            match path.canonicalize() {
                Ok(abs) => Ok(Value::String(abs.to_string_lossy().to_string())),
                Err(_) => Ok(Value::String(p.clone())),
            }
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn normalize(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(p)) = args.get(0) {
            let mut components = Vec::new();
            for comp in Path::new(p).components() {
                use std::path::Component;
                match comp {
                    Component::ParentDir => { components.pop(); }
                    Component::CurDir => {}
                    _ => components.push(comp),
                }
            }
            let result: PathBuf = components.iter().collect();
            Ok(Value::String(result.to_string_lossy().to_string()))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn is_absolute(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(p)) = args.get(0) {
            Ok(Value::Boolean(Path::new(p).is_absolute()))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn exists(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(p)) = args.get(0) {
            Ok(Value::Boolean(Path::new(p).exists()))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_file(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(p)) = args.get(0) {
            Ok(Value::Boolean(Path::new(p).is_file()))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_dir(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(p)) = args.get(0) {
            Ok(Value::Boolean(Path::new(p).is_dir()))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn parse(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(p)) = args.get(0) {
            let path = Path::new(p);
            let mut result = HashMap::new();
            result.insert("dir".to_string(), Value::String(path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()));
            result.insert("base".to_string(), Value::String(path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
            result.insert("ext".to_string(), Value::String(path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()));
            result.insert("name".to_string(), Value::String(path.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
            Ok(Value::Table(result))
        } else {
            Ok(Value::Table(HashMap::new()))
        }
    }
    fn separator() -> MintasResult<Value> {
        Ok(Value::String(std::path::MAIN_SEPARATOR.to_string()))
    }
    fn relative(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Ok(Value::String(String::new()));
        }
        if let (Some(Value::String(from)), Some(Value::String(to))) = (args.get(0), args.get(1)) {
            let from_path = Path::new(from);
            let to_path = Path::new(to);
            match to_path.strip_prefix(from_path) {
                Ok(rel) => Ok(Value::String(rel.to_string_lossy().to_string())),
                Err(_) => Ok(Value::String(to.clone())),
            }
        } else {
            Ok(Value::String(String::new()))
        }
    }
}