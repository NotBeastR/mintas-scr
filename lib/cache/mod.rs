use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Instant, Duration};
struct CacheEntry {
    value: Value,
    expires: Option<Instant>,
}
lazy_static::lazy_static! {
    static ref CACHE: Mutex<HashMap<String, CacheEntry>> = Mutex::new(HashMap::new());
}
pub struct CacheModule;
impl CacheModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "get" => Self::get(args),
            "set" => Self::set(args),
            "has" => Self::has(args),
            "delete" | "remove" => Self::delete(args),
            "clear" => Self::clear(),
            "keys" => Self::keys(),
            "size" => Self::size(),
            "ttl" => Self::ttl(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown cache function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn get(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(key)) = args.get(0) {
            let mut cache = CACHE.lock().unwrap();
            cache.retain(|_, entry| entry.expires.map(|e| e > Instant::now()).unwrap_or(true));
            if let Some(entry) = cache.get(key) {
                return Ok(entry.value.clone());
            }
            if let Some(default) = args.get(1) {
                return Ok(default.clone());
            }
        }
        Ok(Value::Empty)
    }
    fn set(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 { return Ok(Value::Boolean(false)); }
        if let Some(Value::String(key)) = args.get(0) {
            let value = args[1].clone();
            let ttl_ms = match args.get(2) {
                Some(Value::Number(n)) => Some(*n as u64),
                _ => None,
            };
            let expires = ttl_ms.map(|ms| Instant::now() + Duration::from_millis(ms));
            CACHE.lock().unwrap().insert(key.clone(), CacheEntry { value, expires });
            return Ok(Value::Boolean(true));
        }
        Ok(Value::Boolean(false))
    }
    fn has(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(key)) = args.get(0) {
            let cache = CACHE.lock().unwrap();
            if let Some(entry) = cache.get(key) {
                let valid = entry.expires.map(|e| e > Instant::now()).unwrap_or(true);
                return Ok(Value::Boolean(valid));
            }
        }
        Ok(Value::Boolean(false))
    }
    fn delete(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(key)) = args.get(0) {
            return Ok(Value::Boolean(CACHE.lock().unwrap().remove(key).is_some()));
        }
        Ok(Value::Boolean(false))
    }
    fn clear() -> MintasResult<Value> {
        CACHE.lock().unwrap().clear();
        Ok(Value::Boolean(true))
    }
    fn keys() -> MintasResult<Value> {
        let cache = CACHE.lock().unwrap();
        let keys: Vec<Value> = cache.keys().map(|k| Value::String(k.clone())).collect();
        Ok(Value::Array(keys))
    }
    fn size() -> MintasResult<Value> {
        Ok(Value::Number(CACHE.lock().unwrap().len() as f64))
    }
    fn ttl(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(key)) = args.get(0) {
            let cache = CACHE.lock().unwrap();
            if let Some(entry) = cache.get(key) {
                if let Some(expires) = entry.expires {
                    let now = Instant::now();
                    if expires > now {
                        return Ok(Value::Number((expires - now).as_millis() as f64));
                    }
                    return Ok(Value::Number(0.0));
                }
                return Ok(Value::Number(-1.0)); 
            }
        }
        Ok(Value::Number(-2.0)) 
    }
}