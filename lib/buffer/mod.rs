use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct BufferModule;
impl BufferModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "create" | "alloc" => Self::create(args),
            "from" => Self::from(args),
            "from_hex" => Self::from_hex(args),
            "from_base64" => Self::from_base64(args),
            "to_string" => Self::to_string(args),
            "to_hex" => Self::to_hex(args),
            "to_base64" => Self::to_base64(args),
            "concat" => Self::concat(args),
            "slice" => Self::slice(args),
            "length" | "len" => Self::length(args),
            "get" => Self::get(args),
            "set" => Self::set(args),
            "fill" => Self::fill(args),
            "copy" => Self::copy(args),
            "equals" => Self::equals(args),
            "compare" => Self::compare(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown buffer function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn create(args: &[Value]) -> MintasResult<Value> {
        let size = match args.get(0) { Some(Value::Number(n)) => *n as usize, _ => 0 };
        let fill = match args.get(1) { Some(Value::Number(n)) => *n as u8, _ => 0 };
        let buffer: Vec<Value> = vec![Value::Number(fill as f64); size];
        Ok(Value::Array(buffer))
    }
    fn from(args: &[Value]) -> MintasResult<Value> {
        match args.get(0) {
            Some(Value::String(s)) => {
                let buffer: Vec<Value> = s.bytes().map(|b| Value::Number(b as f64)).collect();
                Ok(Value::Array(buffer))
            }
            Some(Value::Array(arr)) => Ok(Value::Array(arr.clone())),
            _ => Ok(Value::Array(vec![])),
        }
    }
    fn from_hex(args: &[Value]) -> MintasResult<Value> {
        let hex = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Array(vec![])) };
        let buffer: Vec<Value> = (0..hex.len()).step_by(2)
            .filter_map(|i| u8::from_str_radix(&hex[i..i.min(hex.len()-1)+2], 16).ok())
            .map(|b| Value::Number(b as f64))
            .collect();
        Ok(Value::Array(buffer))
    }
    fn from_base64(args: &[Value]) -> MintasResult<Value> {
        let b64 = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Array(vec![])) };
        let decoded = Self::base64_decode(&b64);
        let buffer: Vec<Value> = decoded.iter().map(|b| Value::Number(*b as f64)).collect();
        Ok(Value::Array(buffer))
    }
    fn to_string(args: &[Value]) -> MintasResult<Value> {
        let buffer = match args.get(0) { Some(Value::Array(arr)) => arr.clone(), _ => return Ok(Value::String(String::new())) };
        let bytes: Vec<u8> = buffer.iter().filter_map(|v| match v { Value::Number(n) => Some(*n as u8), _ => None }).collect();
        Ok(Value::String(String::from_utf8_lossy(&bytes).to_string()))
    }
    fn to_hex(args: &[Value]) -> MintasResult<Value> {
        let buffer = match args.get(0) { Some(Value::Array(arr)) => arr.clone(), _ => return Ok(Value::String(String::new())) };
        let hex: String = buffer.iter().filter_map(|v| match v { Value::Number(n) => Some(format!("{:02x}", *n as u8)), _ => None }).collect();
        Ok(Value::String(hex))
    }
    fn to_base64(args: &[Value]) -> MintasResult<Value> {
        let buffer = match args.get(0) { Some(Value::Array(arr)) => arr.clone(), _ => return Ok(Value::String(String::new())) };
        let bytes: Vec<u8> = buffer.iter().filter_map(|v| match v { Value::Number(n) => Some(*n as u8), _ => None }).collect();
        Ok(Value::String(Self::base64_encode(&bytes)))
    }
    fn concat(args: &[Value]) -> MintasResult<Value> {
        let mut result = Vec::new();
        for arg in args {
            if let Value::Array(arr) = arg { result.extend(arr.clone()); }
        }
        Ok(Value::Array(result))
    }
    fn slice(args: &[Value]) -> MintasResult<Value> {
        let buffer = match args.get(0) { Some(Value::Array(arr)) => arr.clone(), _ => return Ok(Value::Array(vec![])) };
        let start = match args.get(1) { Some(Value::Number(n)) => *n as usize, _ => 0 };
        let end = match args.get(2) { Some(Value::Number(n)) => *n as usize, _ => buffer.len() };
        Ok(Value::Array(buffer[start.min(buffer.len())..end.min(buffer.len())].to_vec()))
    }
    fn length(args: &[Value]) -> MintasResult<Value> {
        let buffer = match args.get(0) { Some(Value::Array(arr)) => arr.len(), _ => 0 };
        Ok(Value::Number(buffer as f64))
    }
    fn get(args: &[Value]) -> MintasResult<Value> {
        let buffer = match args.get(0) { Some(Value::Array(arr)) => arr.clone(), _ => return Ok(Value::Empty) };
        let index = match args.get(1) { Some(Value::Number(n)) => *n as usize, _ => return Ok(Value::Empty) };
        buffer.get(index).cloned().ok_or_else(|| MintasError::RuntimeError {
            message: "Index out of bounds".to_string(), location: SourceLocation::new(0, 0),
        })
    }
    fn set(args: &[Value]) -> MintasResult<Value> {
        let mut buffer = match args.get(0) { Some(Value::Array(arr)) => arr.clone(), _ => return Ok(Value::Array(vec![])) };
        let index = match args.get(1) { Some(Value::Number(n)) => *n as usize, _ => return Ok(Value::Array(buffer)) };
        let value = args.get(2).cloned().unwrap_or(Value::Number(0.0));
        if index < buffer.len() { buffer[index] = value; }
        Ok(Value::Array(buffer))
    }
    fn fill(args: &[Value]) -> MintasResult<Value> {
        let mut buffer = match args.get(0) { Some(Value::Array(arr)) => arr.clone(), _ => return Ok(Value::Array(vec![])) };
        let value = args.get(1).cloned().unwrap_or(Value::Number(0.0));
        for item in buffer.iter_mut() { *item = value.clone(); }
        Ok(Value::Array(buffer))
    }
    fn copy(args: &[Value]) -> MintasResult<Value> {
        let buffer = match args.get(0) { Some(Value::Array(arr)) => arr.clone(), _ => return Ok(Value::Array(vec![])) };
        Ok(Value::Array(buffer))
    }
    fn equals(args: &[Value]) -> MintasResult<Value> {
        let a = match args.get(0) { Some(Value::Array(arr)) => arr.clone(), _ => return Ok(Value::Boolean(false)) };
        let b = match args.get(1) { Some(Value::Array(arr)) => arr.clone(), _ => return Ok(Value::Boolean(false)) };
        Ok(Value::Boolean(a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| {
            match (x, y) { (Value::Number(a), Value::Number(b)) => (*a as u8) == (*b as u8), _ => false }
        })))
    }
    fn compare(args: &[Value]) -> MintasResult<Value> {
        let a = match args.get(0) { Some(Value::Array(arr)) => arr.clone(), _ => return Ok(Value::Number(0.0)) };
        let b = match args.get(1) { Some(Value::Array(arr)) => arr.clone(), _ => return Ok(Value::Number(0.0)) };
        for (x, y) in a.iter().zip(b.iter()) {
            if let (Value::Number(av), Value::Number(bv)) = (x, y) {
                if (*av as u8) < (*bv as u8) { return Ok(Value::Number(-1.0)); }
                if (*av as u8) > (*bv as u8) { return Ok(Value::Number(1.0)); }
            }
        }
        Ok(Value::Number((a.len() as i32 - b.len() as i32).signum() as f64))
    }
    fn base64_encode(data: &[u8]) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::new();
        for chunk in data.chunks(3) {
            let b = [chunk.get(0).copied().unwrap_or(0), chunk.get(1).copied().unwrap_or(0), chunk.get(2).copied().unwrap_or(0)];
            result.push(CHARS[(b[0] >> 2) as usize] as char);
            result.push(CHARS[((b[0] & 0x03) << 4 | b[1] >> 4) as usize] as char);
            result.push(if chunk.len() > 1 { CHARS[((b[1] & 0x0f) << 2 | b[2] >> 6) as usize] as char } else { '=' });
            result.push(if chunk.len() > 2 { CHARS[(b[2] & 0x3f) as usize] as char } else { '=' });
        }
        result
    }
    fn base64_decode(data: &str) -> Vec<u8> {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = Vec::new();
        let bytes: Vec<u8> = data.bytes().filter(|&b| b != b'=').collect();
        for chunk in bytes.chunks(4) {
            if chunk.len() < 2 { break; }
            let idx: Vec<u8> = chunk.iter().map(|&b| CHARS.iter().position(|&c| c == b).unwrap_or(0) as u8).collect();
            result.push((idx[0] << 2) | (idx.get(1).unwrap_or(&0) >> 4));
            if chunk.len() > 2 { result.push((idx[1] << 4) | (idx.get(2).unwrap_or(&0) >> 2)); }
            if chunk.len() > 3 { result.push((idx[2] << 6) | idx.get(3).unwrap_or(&0)); }
        }
        result
    }
}