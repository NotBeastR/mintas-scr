use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
pub struct WorkerModule;
static WORKER_COUNTER: AtomicU64 = AtomicU64::new(1);
impl WorkerModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "spawn" => Self::spawn(args),
            "create" => Self::create(args),
            "start" => Self::start(args),
            "stop" => Self::stop(args),
            "status" => Self::status(args),
            "list" => Self::list(args),
            "send" => Self::send(args),
            "receive" => Self::receive(args),
            "pool" => Self::pool(args),
            "terminate" => Self::terminate(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown worker function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn spawn(args: &[Value]) -> MintasResult<Value> {
        let name = if !args.is_empty() {
            match &args[0] {
                Value::String(s) => s.clone(),
                _ => format!("worker_{}", WORKER_COUNTER.fetch_add(1, Ordering::SeqCst)),
            }
        } else {
            format!("worker_{}", WORKER_COUNTER.fetch_add(1, Ordering::SeqCst))
        };
        let mut worker = HashMap::new();
        worker.insert("id".to_string(), Value::String(name.clone()));
        worker.insert("status".to_string(), Value::String("running".to_string()));
        worker.insert("created_at".to_string(), Value::Number(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as f64
        ));
        Ok(Value::Table(worker))
    }
    fn create(args: &[Value]) -> MintasResult<Value> {
        Self::spawn(args)
    }
    fn start(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "worker.start".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Boolean(true))
    }
    fn stop(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "worker.stop".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Boolean(true))
    }
    fn status(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "worker.status".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let mut result = HashMap::new();
        result.insert("status".to_string(), Value::String("running".to_string()));
        result.insert("tasks_completed".to_string(), Value::Number(0.0));
        result.insert("uptime".to_string(), Value::Number(0.0));
        Ok(Value::Table(result))
    }
    fn list(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Array(Vec::new()))
    }
    fn send(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "worker.send".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Boolean(true))
    }
    fn receive(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "worker.receive".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Empty)
    }
    fn pool(args: &[Value]) -> MintasResult<Value> {
        let size = if !args.is_empty() {
            match &args[0] {
                Value::Number(n) => *n as usize,
                _ => 4,
            }
        } else {
            4
        };
        let mut pool = HashMap::new();
        pool.insert("size".to_string(), Value::Number(size as f64));
        pool.insert("active".to_string(), Value::Number(0.0));
        pool.insert("idle".to_string(), Value::Number(size as f64));
        Ok(Value::Table(pool))
    }
    fn terminate(args: &[Value]) -> MintasResult<Value> {
        Self::stop(args)
    }
}