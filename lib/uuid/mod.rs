use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::time::{SystemTime, UNIX_EPOCH};
pub struct UuidModule;
impl UuidModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "v4" | "random" | "new" => Self::v4(),
            "v1" | "time" => Self::v1(),
            "nil" | "empty" => Self::nil(),
            "parse" => Self::parse(args),
            "valid" | "is_valid" => Self::is_valid(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown uuid function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn v4() -> MintasResult<Value> {
        let mut bytes = [0u8; 16];
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let seed = now.as_nanos() as u64;
        let mut state = seed;
        for byte in &mut bytes {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            *byte = (state >> 33) as u8;
        }
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Ok(Value::String(Self::format_uuid(&bytes)))
    }
    fn v1() -> MintasResult<Value> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let timestamp = now.as_nanos() as u64;
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&timestamp.to_be_bytes());
        let seed = timestamp.wrapping_mul(6364136223846793005);
        bytes[10] = (seed >> 8) as u8;
        bytes[11] = (seed >> 16) as u8;
        bytes[12] = (seed >> 24) as u8;
        bytes[13] = (seed >> 32) as u8;
        bytes[14] = (seed >> 40) as u8;
        bytes[15] = (seed >> 48) as u8;
        bytes[6] = (bytes[6] & 0x0f) | 0x10;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Ok(Value::String(Self::format_uuid(&bytes)))
    }
    fn nil() -> MintasResult<Value> {
        Ok(Value::String("00000000-0000-0000-0000-000000000000".to_string()))
    }
    fn parse(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let clean: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            if clean.len() == 32 {
                let formatted = format!(
                    "{}-{}-{}-{}-{}",
                    &clean[0..8], &clean[8..12], &clean[12..16], &clean[16..20], &clean[20..32]
                );
                Ok(Value::String(formatted.to_lowercase()))
            } else {
                Ok(Value::String(String::new()))
            }
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn is_valid(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let clean: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            Ok(Value::Boolean(clean.len() == 32))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn format_uuid(bytes: &[u8; 16]) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        )
    }
}