use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
pub struct SysfilesModule;
impl SysfilesModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "read" => Self::read_file(args),
            "write" => Self::write_file(args),
            "append" => Self::append_file(args),
            "copy" => Self::copy(args),
            "move" | "rename" => Self::move_file(args),
            "remove" | "delete" => Self::remove(args),
            "mkdir" | "mkdirp" => Self::mkdir(args),
            "rmdir" => Self::rmdir(args),
            "exists" => Self::exists(args),
            "stat" | "info" => Self::stat(args),
            "list" | "readdir" => Self::list_dir(args),
            "glob" => Self::glob(args),
            "size" => Self::size(args),
            "touch" => Self::touch(args),
            "empty" => Self::empty_dir(args),
            "ensure" => Self::ensure_dir(args),
            "copydir" => Self::copy_dir(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown sysfiles function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn read_file(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(path)) = args.get(0) {
            match fs::read_to_string(path) {
                Ok(content) => Ok(Value::String(content)),
                Err(e) => Err(MintasError::RuntimeError {
                    message: format!("Failed to read file: {}", e),
                    location: SourceLocation::new(0, 0),
                }),
            }
        } else {
            Err(MintasError::TypeError { message: "Path required".to_string(), location: SourceLocation::new(0, 0) })
        }
    }
    fn write_file(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount { function: "sysfiles.write".to_string(), expected: 2, got: args.len(), location: SourceLocation::new(0, 0) });
        }
        if let (Some(Value::String(path)), Some(Value::String(content))) = (args.get(0), args.get(1)) {
            if let Some(parent) = Path::new(path).parent() {
                let _ = fs::create_dir_all(parent);
            }
            match fs::write(path, content) {
                Ok(_) => Ok(Value::Boolean(true)),
                Err(_) => Ok(Value::Boolean(false)),
            }
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn append_file(args: &[Value]) -> MintasResult<Value> {
        use std::io::Write;
        if args.len() < 2 { return Ok(Value::Boolean(false)); }
        if let (Some(Value::String(path)), Some(Value::String(content))) = (args.get(0), args.get(1)) {
            match fs::OpenOptions::new().create(true).append(true).open(path) {
                Ok(mut file) => {
                    match file.write_all(content.as_bytes()) {
                        Ok(_) => Ok(Value::Boolean(true)),
                        Err(_) => Ok(Value::Boolean(false)),
                    }
                }
                Err(_) => Ok(Value::Boolean(false)),
            }
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn copy(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 { return Ok(Value::Boolean(false)); }
        if let (Some(Value::String(src)), Some(Value::String(dst))) = (args.get(0), args.get(1)) {
            if let Some(parent) = Path::new(dst).parent() {
                let _ = fs::create_dir_all(parent);
            }
            match fs::copy(src, dst) {
                Ok(_) => Ok(Value::Boolean(true)),
                Err(_) => Ok(Value::Boolean(false)),
            }
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn move_file(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 { return Ok(Value::Boolean(false)); }
        if let (Some(Value::String(src)), Some(Value::String(dst))) = (args.get(0), args.get(1)) {
            if let Some(parent) = Path::new(dst).parent() {
                let _ = fs::create_dir_all(parent);
            }
            match fs::rename(src, dst) {
                Ok(_) => Ok(Value::Boolean(true)),
                Err(_) => Ok(Value::Boolean(false)),
            }
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn remove(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(path)) = args.get(0) {
            let p = Path::new(path);
            if p.is_dir() {
                match fs::remove_dir_all(path) {
                    Ok(_) => Ok(Value::Boolean(true)),
                    Err(_) => Ok(Value::Boolean(false)),
                }
            } else {
                match fs::remove_file(path) {
                    Ok(_) => Ok(Value::Boolean(true)),
                    Err(_) => Ok(Value::Boolean(false)),
                }
            }
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn mkdir(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(path)) = args.get(0) {
            match fs::create_dir_all(path) {
                Ok(_) => Ok(Value::Boolean(true)),
                Err(_) => Ok(Value::Boolean(false)),
            }
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn rmdir(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(path)) = args.get(0) {
            match fs::remove_dir_all(path) {
                Ok(_) => Ok(Value::Boolean(true)),
                Err(_) => Ok(Value::Boolean(false)),
            }
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn exists(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(path)) = args.get(0) {
            Ok(Value::Boolean(Path::new(path).exists()))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn stat(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(path)) = args.get(0) {
            match fs::metadata(path) {
                Ok(meta) => {
                    let mut result = HashMap::new();
                    result.insert("size".to_string(), Value::Number(meta.len() as f64));
                    result.insert("is_file".to_string(), Value::Boolean(meta.is_file()));
                    result.insert("is_dir".to_string(), Value::Boolean(meta.is_dir()));
                    result.insert("readonly".to_string(), Value::Boolean(meta.permissions().readonly()));
                    Ok(Value::Table(result))
                }
                Err(_) => Ok(Value::Table(HashMap::new())),
            }
        } else {
            Ok(Value::Table(HashMap::new()))
        }
    }
    fn list_dir(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(path)) = args.get(0) {
            match fs::read_dir(path) {
                Ok(entries) => {
                    let files: Vec<Value> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| Value::String(e.file_name().to_string_lossy().to_string()))
                        .collect();
                    Ok(Value::Array(files))
                }
                Err(_) => Ok(Value::Array(vec![])),
            }
        } else {
            Ok(Value::Array(vec![]))
        }
    }
    fn glob(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(pattern)) = args.get(0) {
            let dir = Path::new(pattern).parent().unwrap_or(Path::new("."));
            let ext = Path::new(pattern).extension().map(|e| e.to_string_lossy().to_string());
            match fs::read_dir(dir) {
                Ok(entries) => {
                    let files: Vec<Value> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            if let Some(ref ext_filter) = ext {
                                e.path().extension().map(|e| e.to_string_lossy().to_string()) == Some(ext_filter.clone())
                            } else {
                                true
                            }
                        })
                        .map(|e| Value::String(e.path().to_string_lossy().to_string()))
                        .collect();
                    Ok(Value::Array(files))
                }
                Err(_) => Ok(Value::Array(vec![])),
            }
        } else {
            Ok(Value::Array(vec![]))
        }
    }
    fn size(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(path)) = args.get(0) {
            match fs::metadata(path) {
                Ok(meta) => Ok(Value::Number(meta.len() as f64)),
                Err(_) => Ok(Value::Number(0.0)),
            }
        } else {
            Ok(Value::Number(0.0))
        }
    }
    fn touch(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(path)) = args.get(0) {
            if let Some(parent) = Path::new(path).parent() {
                let _ = fs::create_dir_all(parent);
            }
            match fs::OpenOptions::new().create(true).write(true).open(path) {
                Ok(_) => Ok(Value::Boolean(true)),
                Err(_) => Ok(Value::Boolean(false)),
            }
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn empty_dir(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(path)) = args.get(0) {
            let _ = fs::remove_dir_all(path);
            match fs::create_dir_all(path) {
                Ok(_) => Ok(Value::Boolean(true)),
                Err(_) => Ok(Value::Boolean(false)),
            }
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn ensure_dir(args: &[Value]) -> MintasResult<Value> {
        Self::mkdir(args)
    }
    fn copy_dir(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 { return Ok(Value::Boolean(false)); }
        if let (Some(Value::String(src)), Some(Value::String(dst))) = (args.get(0), args.get(1)) {
            Self::copy_dir_recursive(Path::new(src), Path::new(dst))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn copy_dir_recursive(src: &Path, dst: &Path) -> MintasResult<Value> {
        let _ = fs::create_dir_all(dst);
        if let Ok(entries) = fs::read_dir(src) {
            for entry in entries.filter_map(|e| e.ok()) {
                let src_path = entry.path();
                let dst_path = dst.join(entry.file_name());
                if src_path.is_dir() {
                    Self::copy_dir_recursive(&src_path, &dst_path)?;
                } else {
                    let _ = fs::copy(&src_path, &dst_path);
                }
            }
        }
        Ok(Value::Boolean(true))
    }
}