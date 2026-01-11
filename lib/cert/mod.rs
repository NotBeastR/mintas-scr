use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct CertModule;
impl CertModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "generate" => Self::generate(args),
            "load" => Self::load(args),
            "verify" => Self::verify(args),
            "info" => Self::info(args),
            "sign" => Self::sign(args),
            "self_signed" => Self::self_signed(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown cert function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn generate(args: &[Value]) -> MintasResult<Value> {
        let cn = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => "localhost".to_string() };
        let days = match args.get(1) { Some(Value::Number(n)) => *n as i64, _ => 365 };
        let mut cert = HashMap::new();
        cert.insert("common_name".to_string(), Value::String(cn));
        cert.insert("valid_days".to_string(), Value::Number(days as f64));
        cert.insert("type".to_string(), Value::String("X509".to_string()));
        cert.insert("__type__".to_string(), Value::String("Certificate".to_string()));
        cert.insert("public_key".to_string(), Value::String(Self::generate_key("public")));
        cert.insert("private_key".to_string(), Value::String(Self::generate_key("private")));
        Ok(Value::Table(cert))
    }
    fn generate_key(key_type: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let key: String = (0..64).map(|i| {
            let v = ((seed.wrapping_mul(1103515245).wrapping_add(i as u128)) % 16) as u8;
            format!("{:x}", v)
        }).collect();
        format!("-----BEGIN {} KEY-----\n{}\n-----END {} KEY-----", 
            key_type.to_uppercase(), key, key_type.to_uppercase())
    }
    fn load(args: &[Value]) -> MintasResult<Value> {
        let path = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let mut cert = HashMap::new();
        cert.insert("path".to_string(), Value::String(path));
        cert.insert("content".to_string(), Value::String(content));
        cert.insert("__type__".to_string(), Value::String("Certificate".to_string()));
        Ok(Value::Table(cert))
    }
    fn verify(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::Table(cert)) = args.get(0) {
            let has_key = cert.contains_key("public_key") || cert.contains_key("content");
            return Ok(Value::Boolean(has_key));
        }
        Ok(Value::Boolean(false))
    }
    fn info(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::Table(cert)) = args.get(0) {
            let mut info = HashMap::new();
            if let Some(cn) = cert.get("common_name") { info.insert("cn".to_string(), cn.clone()); }
            if let Some(days) = cert.get("valid_days") { info.insert("valid_days".to_string(), days.clone()); }
            info.insert("type".to_string(), Value::String("X509".to_string()));
            return Ok(Value::Table(info));
        }
        Ok(Value::Empty)
    }
    fn sign(args: &[Value]) -> MintasResult<Value> {
        let data = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let _cert = match args.get(1) { Some(Value::Table(c)) => c.clone(), _ => return Ok(Value::Empty) };
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let sig: String = data.bytes().enumerate().map(|(i, b)| {
            format!("{:02x}", b.wrapping_add((seed.wrapping_mul(i as u128) % 256) as u8))
        }).collect();
        Ok(Value::String(sig))
    }
    fn self_signed(args: &[Value]) -> MintasResult<Value> {
        Self::generate(args)
    }
}