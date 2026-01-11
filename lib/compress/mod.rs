use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct CompressModule;
impl CompressModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "deflate" => Self::deflate(args),
            "inflate" => Self::inflate(args),
            "gzip" => Self::gzip(args),
            "gunzip" => Self::gunzip(args),
            "zip" => Self::zip(args),
            "unzip" => Self::unzip(args),
            "compress" => Self::compress(args),
            "decompress" => Self::decompress(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown compress function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn compress(args: &[Value]) -> MintasResult<Value> {
        let data = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let mut result = String::new();
        let chars: Vec<char> = data.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let ch = chars[i];
            let mut count = 1;
            while i + count < chars.len() && chars[i + count] == ch && count < 255 {
                count += 1;
            }
            if count > 3 {
                result.push_str(&format!("~{}{}", count, ch));
            } else {
                for _ in 0..count { result.push(ch); }
            }
            i += count;
        }
        Ok(Value::String(result))
    }
    fn decompress(args: &[Value]) -> MintasResult<Value> {
        let data = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let mut result = String::new();
        let chars: Vec<char> = data.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '~' && i + 2 < chars.len() {
                let count_str: String = chars[i+1..].iter().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(count) = count_str.parse::<usize>() {
                    let ch = chars[i + 1 + count_str.len()];
                    for _ in 0..count { result.push(ch); }
                    i += 2 + count_str.len();
                    continue;
                }
            }
            result.push(chars[i]);
            i += 1;
        }
        Ok(Value::String(result))
    }
    fn deflate(args: &[Value]) -> MintasResult<Value> { Self::compress(args) }
    fn inflate(args: &[Value]) -> MintasResult<Value> { Self::decompress(args) }
    fn gzip(args: &[Value]) -> MintasResult<Value> { Self::compress(args) }
    fn gunzip(args: &[Value]) -> MintasResult<Value> { Self::decompress(args) }
    fn zip(args: &[Value]) -> MintasResult<Value> { Self::compress(args) }
    fn unzip(args: &[Value]) -> MintasResult<Value> { Self::decompress(args) }
}