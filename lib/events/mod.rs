use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::sync::Mutex;
lazy_static::lazy_static! {
    static ref EVENTS: Mutex<HashMap<String, Vec<Value>>> = Mutex::new(HashMap::new());
    static ref EVENT_DATA: Mutex<HashMap<String, Vec<Value>>> = Mutex::new(HashMap::new());
}
pub struct EventsModule;
impl EventsModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "on" | "listen" => Self::on(args),
            "emit" | "trigger" => Self::emit(args),
            "off" | "remove" => Self::off(args),
            "once" => Self::once(args),
            "clear" => Self::clear(args),
            "listeners" => Self::listeners(args),
            "events" => Self::events(),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown events function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn on(args: &[Value]) -> MintasResult<Value> {
        let event = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Boolean(false)) };
        let handler = args.get(1).cloned().unwrap_or(Value::Empty);
        let mut events = EVENTS.lock().unwrap();
        events.entry(event).or_insert_with(Vec::new).push(handler);
        Ok(Value::Boolean(true))
    }
    fn emit(args: &[Value]) -> MintasResult<Value> {
        let event = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Boolean(false)) };
        let data = args.get(1).cloned().unwrap_or(Value::Empty);
        let mut event_data = EVENT_DATA.lock().unwrap();
        event_data.entry(event.clone()).or_insert_with(Vec::new).push(data);
        let events = EVENTS.lock().unwrap();
        let count = events.get(&event).map(|h| h.len()).unwrap_or(0);
        Ok(Value::Number(count as f64))
    }
    fn off(args: &[Value]) -> MintasResult<Value> {
        let event = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Boolean(false)) };
        let mut events = EVENTS.lock().unwrap();
        Ok(Value::Boolean(events.remove(&event).is_some()))
    }
    fn once(args: &[Value]) -> MintasResult<Value> {
        Self::on(args)
    }
    fn clear(_args: &[Value]) -> MintasResult<Value> {
        EVENTS.lock().unwrap().clear();
        EVENT_DATA.lock().unwrap().clear();
        Ok(Value::Boolean(true))
    }
    fn listeners(args: &[Value]) -> MintasResult<Value> {
        let event = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Number(0.0)) };
        let events = EVENTS.lock().unwrap();
        let count = events.get(&event).map(|h| h.len()).unwrap_or(0);
        Ok(Value::Number(count as f64))
    }
    fn events() -> MintasResult<Value> {
        let events = EVENTS.lock().unwrap();
        let names: Vec<Value> = events.keys().map(|k| Value::String(k.clone())).collect();
        Ok(Value::Array(names))
    }
}