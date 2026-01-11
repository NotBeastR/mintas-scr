use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::process::{Command, Stdio};
pub struct SubprocessModule;
impl SubprocessModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "run" | "exec" => Self::run(args),
            "spawn" => Self::spawn(args),
            "shell" => Self::shell(args),
            "output" => Self::output(args),
            "call" => Self::call(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown subprocess function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn run(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(cmd)) = args.get(0) {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() {
                return Ok(Value::Table(HashMap::new()));
            }
            let program = parts[0];
            let cmd_args = &parts[1..];
            match Command::new(program).args(cmd_args).output() {
                Ok(output) => {
                    let mut result = HashMap::new();
                    result.insert("stdout".to_string(), Value::String(String::from_utf8_lossy(&output.stdout).to_string()));
                    result.insert("stderr".to_string(), Value::String(String::from_utf8_lossy(&output.stderr).to_string()));
                    result.insert("code".to_string(), Value::Number(output.status.code().unwrap_or(-1) as f64));
                    result.insert("success".to_string(), Value::Boolean(output.status.success()));
                    Ok(Value::Table(result))
                }
                Err(e) => {
                    let mut result = HashMap::new();
                    result.insert("error".to_string(), Value::String(e.to_string()));
                    result.insert("success".to_string(), Value::Boolean(false));
                    Ok(Value::Table(result))
                }
            }
        } else if let Some(Value::Array(arr)) = args.get(0) {
            if arr.is_empty() {
                return Ok(Value::Table(HashMap::new()));
            }
            let program = match &arr[0] {
                Value::String(s) => s.clone(),
                _ => return Ok(Value::Table(HashMap::new())),
            };
            let cmd_args: Vec<String> = arr[1..].iter().filter_map(|v| {
                if let Value::String(s) = v { Some(s.clone()) } else { None }
            }).collect();
            match Command::new(&program).args(&cmd_args).output() {
                Ok(output) => {
                    let mut result = HashMap::new();
                    result.insert("stdout".to_string(), Value::String(String::from_utf8_lossy(&output.stdout).to_string()));
                    result.insert("stderr".to_string(), Value::String(String::from_utf8_lossy(&output.stderr).to_string()));
                    result.insert("code".to_string(), Value::Number(output.status.code().unwrap_or(-1) as f64));
                    result.insert("success".to_string(), Value::Boolean(output.status.success()));
                    Ok(Value::Table(result))
                }
                Err(e) => {
                    let mut result = HashMap::new();
                    result.insert("error".to_string(), Value::String(e.to_string()));
                    result.insert("success".to_string(), Value::Boolean(false));
                    Ok(Value::Table(result))
                }
            }
        } else {
            Err(MintasError::TypeError { message: "Command required".to_string(), location: SourceLocation::new(0, 0) })
        }
    }
    fn spawn(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(cmd)) = args.get(0) {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() {
                return Ok(Value::Boolean(false));
            }
            match Command::new(parts[0])
                .args(&parts[1..])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => {
                    let mut result = HashMap::new();
                    result.insert("pid".to_string(), Value::Number(child.id() as f64));
                    result.insert("spawned".to_string(), Value::Boolean(true));
                    Ok(Value::Table(result))
                }
                Err(_) => Ok(Value::Boolean(false)),
            }
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn shell(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(cmd)) = args.get(0) {
            #[cfg(target_os = "windows")]
            let output = Command::new("cmd").args(["/C", cmd]).output();
            #[cfg(not(target_os = "windows"))]
            let output = Command::new("sh").args(["-c", cmd]).output();
            match output {
                Ok(out) => {
                    let mut result = HashMap::new();
                    result.insert("stdout".to_string(), Value::String(String::from_utf8_lossy(&out.stdout).to_string()));
                    result.insert("stderr".to_string(), Value::String(String::from_utf8_lossy(&out.stderr).to_string()));
                    result.insert("code".to_string(), Value::Number(out.status.code().unwrap_or(-1) as f64));
                    result.insert("success".to_string(), Value::Boolean(out.status.success()));
                    Ok(Value::Table(result))
                }
                Err(e) => {
                    let mut result = HashMap::new();
                    result.insert("error".to_string(), Value::String(e.to_string()));
                    result.insert("success".to_string(), Value::Boolean(false));
                    Ok(Value::Table(result))
                }
            }
        } else {
            Ok(Value::Table(HashMap::new()))
        }
    }
    fn output(args: &[Value]) -> MintasResult<Value> {
        match Self::run(args)? {
            Value::Table(map) => {
                if let Some(Value::String(stdout)) = map.get("stdout") {
                    Ok(Value::String(stdout.trim().to_string()))
                } else {
                    Ok(Value::String(String::new()))
                }
            }
            _ => Ok(Value::String(String::new())),
        }
    }
    fn call(args: &[Value]) -> MintasResult<Value> {
        match Self::run(args)? {
            Value::Table(map) => {
                if let Some(Value::Number(code)) = map.get("code") {
                    Ok(Value::Number(*code))
                } else {
                    Ok(Value::Number(-1.0))
                }
            }
            _ => Ok(Value::Number(-1.0)),
        }
    }
}