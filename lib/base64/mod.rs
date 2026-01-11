use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
pub struct Base64Module;
impl Base64Module {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "encode" => Self::encode(args),
            "decode" => Self::decode(args),
            "urlsafe_encode" => Self::urlsafe_encode(args),
            "urlsafe_decode" => Self::urlsafe_decode(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown base64 function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn encode(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            Ok(Value::String(Self::base64_encode(s.as_bytes())))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn decode(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            match Self::base64_decode(s) {
                Ok(bytes) => Ok(Value::String(String::from_utf8_lossy(&bytes).to_string())),
                Err(_) => Ok(Value::String(String::new())),
            }
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn urlsafe_encode(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let encoded = Self::base64_encode(s.as_bytes());
            Ok(Value::String(encoded.replace('+', "-").replace('/', "_").trim_end_matches('=').to_string()))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn urlsafe_decode(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let mut s = s.replace('-', "+").replace('_', "/");
            while s.len() % 4 != 0 { s.push('='); }
            match Self::base64_decode(&s) {
                Ok(bytes) => Ok(Value::String(String::from_utf8_lossy(&bytes).to_string())),
                Err(_) => Ok(Value::String(String::new())),
            }
        } else {
            Ok(Value::String(String::new()))
        }
    }
    const ALPHABET: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    fn base64_encode(data: &[u8]) -> String {
        let mut result = String::new();
        for chunk in data.chunks(3) {
            let b0 = chunk[0] as usize;
            let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
            let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
            result.push(Self::ALPHABET[b0 >> 2] as char);
            result.push(Self::ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
            if chunk.len() > 1 {
                result.push(Self::ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
            } else {
                result.push('=');
            }
            if chunk.len() > 2 {
                result.push(Self::ALPHABET[b2 & 0x3f] as char);
            } else {
                result.push('=');
            }
        }
        result
    }
    fn base64_decode(data: &str) -> Result<Vec<u8>, ()> {
        let mut result = Vec::new();
        let chars: Vec<u8> = data.bytes().filter(|&b| b != b'=').collect();
        for chunk in chars.chunks(4) {
            if chunk.len() < 2 { break; }
            let decode_char = |c: u8| -> Result<u8, ()> {
                match c {
                    b'A'..=b'Z' => Ok(c - b'A'),
                    b'a'..=b'z' => Ok(c - b'a' + 26),
                    b'0'..=b'9' => Ok(c - b'0' + 52),
                    b'+' => Ok(62),
                    b'/' => Ok(63),
                    _ => Err(()),
                }
            };
            let b0 = decode_char(chunk[0])?;
            let b1 = decode_char(chunk[1])?;
            result.push((b0 << 2) | (b1 >> 4));
            if chunk.len() > 2 {
                let b2 = decode_char(chunk[2])?;
                result.push((b1 << 4) | (b2 >> 2));
                if chunk.len() > 3 {
                    let b3 = decode_char(chunk[3])?;
                    result.push((b2 << 6) | b3);
                }
            }
        }
        Ok(result)
    }
}