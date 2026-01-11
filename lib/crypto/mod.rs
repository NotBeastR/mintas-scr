use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct CryptoModule;
impl CryptoModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "encrypt" => Self::encrypt(args),
            "decrypt" => Self::decrypt(args),
            "hash" => Self::hash(args),
            "hmac" => Self::hmac(args),
            "random_bytes" => Self::random_bytes(args),
            "random_hex" => Self::random_hex(args),
            "random_string" => Self::random_string(args),
            "uuid" => Self::uuid(),
            "constant_time_compare" => Self::constant_time_compare(args),
            "xor" => Self::xor(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown crypto function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn encrypt(args: &[Value]) -> MintasResult<Value> {
        let data = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let key = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let key_bytes: Vec<u8> = key.bytes().collect();
        let encrypted: Vec<u8> = data.bytes().enumerate()
            .map(|(i, b)| b ^ key_bytes[i % key_bytes.len()])
            .collect();
        Ok(Value::String(Self::bytes_to_hex(&encrypted)))
    }
    fn decrypt(args: &[Value]) -> MintasResult<Value> {
        let hex_data = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let key = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let data = Self::hex_to_bytes(&hex_data);
        let key_bytes: Vec<u8> = key.bytes().collect();
        let decrypted: Vec<u8> = data.iter().enumerate()
            .map(|(i, b)| b ^ key_bytes[i % key_bytes.len()])
            .collect();
        Ok(Value::String(String::from_utf8_lossy(&decrypted).to_string()))
    }
    fn hash(args: &[Value]) -> MintasResult<Value> {
        let data = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let algo = match args.get(1) { Some(Value::String(s)) => s.to_lowercase(), _ => "sha256".to_string() };
        let hash = match algo.as_str() {
            "md5" => Self::simple_hash(&data, 16),
            "sha1" => Self::simple_hash(&data, 20),
            "sha256" | _ => Self::simple_hash(&data, 32),
        };
        Ok(Value::String(hash))
    }
    fn simple_hash(data: &str, len: usize) -> String {
        let mut hash = vec![0u8; len];
        for (i, b) in data.bytes().enumerate() {
            hash[i % len] ^= b;
            hash[(i + 1) % len] = hash[(i + 1) % len].wrapping_add(b);
            hash[(i + 2) % len] = hash[(i + 2) % len].wrapping_mul(b.wrapping_add(1));
        }
        Self::bytes_to_hex(&hash)
    }
    fn hmac(args: &[Value]) -> MintasResult<Value> {
        let data = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let key = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let combined = format!("{}{}", key, data);
        Ok(Value::String(Self::simple_hash(&combined, 32)))
    }
    fn random_bytes(args: &[Value]) -> MintasResult<Value> {
        let len = match args.get(0) { Some(Value::Number(n)) => *n as usize, _ => 16 };
        let bytes: Vec<u8> = (0..len).map(|_| Self::random_byte()).collect();
        Ok(Value::Array(bytes.iter().map(|b| Value::Number(*b as f64)).collect()))
    }
    fn random_hex(args: &[Value]) -> MintasResult<Value> {
        let len = match args.get(0) { Some(Value::Number(n)) => *n as usize, _ => 32 };
        let bytes: Vec<u8> = (0..len/2).map(|_| Self::random_byte()).collect();
        Ok(Value::String(Self::bytes_to_hex(&bytes)))
    }
    fn random_string(args: &[Value]) -> MintasResult<Value> {
        let len = match args.get(0) { Some(Value::Number(n)) => *n as usize, _ => 16 };
        let charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let chars: Vec<char> = charset.chars().collect();
        let result: String = (0..len).map(|_| chars[Self::random_byte() as usize % chars.len()]).collect();
        Ok(Value::String(result))
    }
    fn uuid() -> MintasResult<Value> {
        let bytes: Vec<u8> = (0..16).map(|_| Self::random_byte()).collect();
        let uuid = format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
            (bytes[6] & 0x0f) | 0x40, bytes[7], (bytes[8] & 0x3f) | 0x80, bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        );
        Ok(Value::String(uuid))
    }
    fn constant_time_compare(args: &[Value]) -> MintasResult<Value> {
        let a = match args.get(0) { Some(Value::String(s)) => s.as_bytes(), _ => return Ok(Value::Boolean(false)) };
        let b = match args.get(1) { Some(Value::String(s)) => s.as_bytes(), _ => return Ok(Value::Boolean(false)) };
        if a.len() != b.len() { return Ok(Value::Boolean(false)); }
        let result = a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y));
        Ok(Value::Boolean(result == 0))
    }
    fn xor(args: &[Value]) -> MintasResult<Value> {
        let a = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let b = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let result: String = a.bytes().zip(b.bytes().cycle()).map(|(x, y)| (x ^ y) as char).collect();
        Ok(Value::String(result))
    }
    fn random_byte() -> u8 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        ((seed * 1103515245 + 12345) % 256) as u8
    }
    fn bytes_to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
    fn hex_to_bytes(hex: &str) -> Vec<u8> {
        (0..hex.len()).step_by(2)
            .filter_map(|i| u8::from_str_radix(&hex[i..i+2], 16).ok())
            .collect()
    }
}