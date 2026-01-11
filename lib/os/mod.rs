use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::env;
pub struct OsModule;
impl OsModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "platform" => Self::platform(),
            "arch" => Self::arch(),
            "hostname" => Self::hostname(),
            "homedir" => Self::homedir(),
            "tmpdir" => Self::tmpdir(),
            "cwd" => Self::cwd(),
            "chdir" => Self::chdir(args),
            "cpus" => Self::cpus(),
            "memory" => Self::memory(),
            "uptime" => Self::uptime(),
            "user" => Self::user(),
            "shell" => Self::shell(),
            "exit" => Self::exit(args),
            "info" => Self::info(),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown os function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn platform() -> MintasResult<Value> {
        Ok(Value::String(env::consts::OS.to_string()))
    }
    fn arch() -> MintasResult<Value> {
        Ok(Value::String(env::consts::ARCH.to_string()))
    }
    fn hostname() -> MintasResult<Value> {
        #[cfg(target_os = "windows")]
        {
            Ok(Value::String(env::var("COMPUTERNAME").unwrap_or_default()))
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(Value::String(env::var("HOSTNAME").or_else(|_| {
                std::fs::read_to_string("/etc/hostname").map(|s| s.trim().to_string())
            }).unwrap_or_default()))
        }
    }
    fn homedir() -> MintasResult<Value> {
        Ok(Value::String(env::var("HOME").or_else(|_| env::var("USERPROFILE")).unwrap_or_default()))
    }
    fn tmpdir() -> MintasResult<Value> {
        Ok(Value::String(env::temp_dir().to_string_lossy().to_string()))
    }
    fn cwd() -> MintasResult<Value> {
        Ok(Value::String(env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()))
    }
    fn chdir(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(path)) = args.get(0) {
            match env::set_current_dir(path) {
                Ok(_) => Ok(Value::Boolean(true)),
                Err(_) => Ok(Value::Boolean(false)),
            }
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn cpus() -> MintasResult<Value> {
        Ok(Value::Number(std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1) as f64))
    }
    fn memory() -> MintasResult<Value> {
        let mut result = HashMap::new();
        result.insert("available".to_string(), Value::String("use sys-info for detailed memory".to_string()));
        Ok(Value::Table(result))
    }
    fn uptime() -> MintasResult<Value> {
        Ok(Value::Number(0.0))
    }
    fn user() -> MintasResult<Value> {
        Ok(Value::String(env::var("USER").or_else(|_| env::var("USERNAME")).unwrap_or_default()))
    }
    fn shell() -> MintasResult<Value> {
        Ok(Value::String(env::var("SHELL").or_else(|_| env::var("COMSPEC")).unwrap_or_default()))
    }
    fn exit(args: &[Value]) -> MintasResult<Value> {
        let code = match args.get(0) {
            Some(Value::Number(n)) => *n as i32,
            _ => 0,
        };
        std::process::exit(code);
    }
    fn info() -> MintasResult<Value> {
        let mut result = HashMap::new();
        result.insert("platform".to_string(), Value::String(env::consts::OS.to_string()));
        result.insert("arch".to_string(), Value::String(env::consts::ARCH.to_string()));
        result.insert("family".to_string(), Value::String(env::consts::FAMILY.to_string()));
        result.insert("cpus".to_string(), Value::Number(std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1) as f64));
        Ok(Value::Table(result))
    }
}