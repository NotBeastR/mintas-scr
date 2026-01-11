use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
lazy_static::lazy_static! {
    static ref QUEUES: Mutex<HashMap<String, VecDeque<Value>>> = Mutex::new(HashMap::new());
}
pub struct QueueModule;
impl QueueModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "create" => Self::create(args),
            "push" | "enqueue" => Self::push(args),
            "pop" | "dequeue" => Self::pop(args),
            "peek" | "front" => Self::peek(args),
            "size" | "len" => Self::size(args),
            "empty" | "is_empty" => Self::is_empty(args),
            "clear" => Self::clear(args),
            "list" => Self::list(args),
            "delete" => Self::delete(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown queue function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn create(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => "default".to_string() };
        QUEUES.lock().unwrap().insert(name.clone(), VecDeque::new());
        Ok(Value::String(name))
    }
    fn push(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => "default".to_string() };
        let value = args.get(1).cloned().unwrap_or(Value::Empty);
        let mut queues = QUEUES.lock().unwrap();
        queues.entry(name).or_insert_with(VecDeque::new).push_back(value);
        Ok(Value::Boolean(true))
    }
    fn pop(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => "default".to_string() };
        let mut queues = QUEUES.lock().unwrap();
        if let Some(queue) = queues.get_mut(&name) {
            if let Some(value) = queue.pop_front() {
                return Ok(value);
            }
        }
        Ok(Value::Empty)
    }
    fn peek(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => "default".to_string() };
        let queues = QUEUES.lock().unwrap();
        if let Some(queue) = queues.get(&name) {
            if let Some(value) = queue.front() {
                return Ok(value.clone());
            }
        }
        Ok(Value::Empty)
    }
    fn size(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => "default".to_string() };
        let queues = QUEUES.lock().unwrap();
        let size = queues.get(&name).map(|q| q.len()).unwrap_or(0);
        Ok(Value::Number(size as f64))
    }
    fn is_empty(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => "default".to_string() };
        let queues = QUEUES.lock().unwrap();
        let empty = queues.get(&name).map(|q| q.is_empty()).unwrap_or(true);
        Ok(Value::Boolean(empty))
    }
    fn clear(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => "default".to_string() };
        let mut queues = QUEUES.lock().unwrap();
        if let Some(queue) = queues.get_mut(&name) { queue.clear(); }
        Ok(Value::Boolean(true))
    }
    fn list(_args: &[Value]) -> MintasResult<Value> {
        let queues = QUEUES.lock().unwrap();
        let names: Vec<Value> = queues.keys().map(|k| Value::String(k.clone())).collect();
        Ok(Value::Array(names))
    }
    fn delete(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Boolean(false)) };
        Ok(Value::Boolean(QUEUES.lock().unwrap().remove(&name).is_some()))
    }
}