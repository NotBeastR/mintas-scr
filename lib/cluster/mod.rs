use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct ClusterModule;
impl ClusterModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "fork" => Self::fork(args),
            "is_master" => Self::is_master(args),
            "is_worker" => Self::is_worker(args),
            "workers" => Self::workers(args),
            "worker_count" => Self::worker_count(args),
            "broadcast" => Self::broadcast(args),
            "send" => Self::send(args),
            "on_message" => Self::on_message(args),
            "shutdown" => Self::shutdown(args),
            "restart" => Self::restart(args),
            "cpu_count" => Self::cpu_count(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown cluster function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn fork(args: &[Value]) -> MintasResult<Value> {
        let count = if !args.is_empty() {
            match &args[0] {
                Value::Number(n) => *n as usize,
                _ => std::thread::available_parallelism().map(|p| p.get()).unwrap_or(4),
            }
        } else {
            std::thread::available_parallelism().map(|p| p.get()).unwrap_or(4)
        };
        let mut workers = Vec::new();
        for i in 0..count {
            let mut worker = HashMap::new();
            worker.insert("id".to_string(), Value::Number(i as f64));
            worker.insert("pid".to_string(), Value::Number((std::process::id() + i as u32) as f64));
            worker.insert("status".to_string(), Value::String("online".to_string()));
            workers.push(Value::Table(worker));
        }
        Ok(Value::Array(workers))
    }
    fn is_master(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Boolean(true))
    }
    fn is_worker(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Boolean(false))
    }
    fn workers(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Array(Vec::new()))
    }
    fn worker_count(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Number(0.0))
    }
    fn broadcast(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "cluster.broadcast".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Boolean(true))
    }
    fn send(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "cluster.send".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Boolean(true))
    }
    fn on_message(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "cluster.on_message".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Boolean(true))
    }
    fn shutdown(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Boolean(true))
    }
    fn restart(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "cluster.restart".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Boolean(true))
    }
    fn cpu_count(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Number(std::thread::available_parallelism().map(|p| p.get()).unwrap_or(4) as f64))
    }
}