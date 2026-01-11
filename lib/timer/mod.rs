use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
lazy_static::lazy_static! {
    static ref TIMERS: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
}
pub struct TimerModule;
impl TimerModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "now" => Self::now(),
            "start" => Self::start(args),
            "stop" | "end" => Self::stop(args),
            "elapsed" => Self::elapsed(args),
            "sleep" | "wait" => Self::sleep(args),
            "timestamp" => Self::timestamp(),
            "measure" => Self::measure(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown timer function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn now() -> MintasResult<Value> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Ok(Value::Number(now.as_millis() as f64))
    }
    fn start(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "default".to_string(),
        };
        TIMERS.lock().unwrap().insert(name.clone(), Instant::now());
        Ok(Value::String(name))
    }
    fn stop(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "default".to_string(),
        };
        if let Some(start) = TIMERS.lock().unwrap().remove(&name) {
            let elapsed = start.elapsed();
            let mut result = HashMap::new();
            result.insert("name".to_string(), Value::String(name));
            result.insert("ms".to_string(), Value::Number(elapsed.as_millis() as f64));
            result.insert("us".to_string(), Value::Number(elapsed.as_micros() as f64));
            result.insert("ns".to_string(), Value::Number(elapsed.as_nanos() as f64));
            result.insert("secs".to_string(), Value::Number(elapsed.as_secs_f64()));
            return Ok(Value::Table(result));
        }
        Ok(Value::Number(0.0))
    }
    fn elapsed(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "default".to_string(),
        };
        if let Some(start) = TIMERS.lock().unwrap().get(&name) {
            return Ok(Value::Number(start.elapsed().as_millis() as f64));
        }
        Ok(Value::Number(0.0))
    }
    fn sleep(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::Number(ms)) = args.get(0) {
            std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
            Ok(Value::Boolean(true))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn timestamp() -> MintasResult<Value> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Ok(Value::Number(now.as_secs() as f64))
    }
    fn measure(_args: &[Value]) -> MintasResult<Value> {
        let now = Instant::now();
        Ok(Value::Number(now.elapsed().as_nanos() as f64))
    }
}