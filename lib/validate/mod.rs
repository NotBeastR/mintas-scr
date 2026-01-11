use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
pub struct ValidateModule;
impl ValidateModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "email" => Self::is_email(args),
            "url" => Self::is_url(args),
            "ip" | "ipv4" => Self::is_ipv4(args),
            "ipv6" => Self::is_ipv6(args),
            "number" | "numeric" => Self::is_numeric(args),
            "alpha" => Self::is_alpha(args),
            "alphanumeric" => Self::is_alphanumeric(args),
            "empty" => Self::is_empty(args),
            "uuid" => Self::is_uuid(args),
            "phone" => Self::is_phone(args),
            "creditcard" | "cc" => Self::is_creditcard(args),
            "hex" => Self::is_hex(args),
            "json" => Self::is_json(args),
            "length" => Self::check_length(args),
            "range" => Self::check_range(args),
            "match" | "regex" => Self::matches_pattern(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown validate function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn is_email(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let valid = s.contains('@') && s.contains('.') && 
                        s.find('@').unwrap() < s.rfind('.').unwrap() &&
                        !s.starts_with('@') && !s.ends_with('.');
            Ok(Value::Boolean(valid))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_url(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let valid = (s.starts_with("http://") || s.starts_with("https://")) && s.len() > 10;
            Ok(Value::Boolean(valid))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_ipv4(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let parts: Vec<&str> = s.split('.').collect();
            let valid = parts.len() == 4 && parts.iter().all(|p| {
                p.parse::<u8>().is_ok()
            });
            Ok(Value::Boolean(valid))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_ipv6(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let parts: Vec<&str> = s.split(':').collect();
            let valid = parts.len() == 8 && parts.iter().all(|p| {
                p.is_empty() || u16::from_str_radix(p, 16).is_ok()
            });
            Ok(Value::Boolean(valid))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_numeric(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            Ok(Value::Boolean(s.parse::<f64>().is_ok()))
        } else if let Some(Value::Number(_)) = args.get(0) {
            Ok(Value::Boolean(true))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_alpha(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            Ok(Value::Boolean(!s.is_empty() && s.chars().all(|c| c.is_alphabetic())))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_alphanumeric(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            Ok(Value::Boolean(!s.is_empty() && s.chars().all(|c| c.is_alphanumeric())))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_empty(args: &[Value]) -> MintasResult<Value> {
        match args.get(0) {
            Some(Value::String(s)) => Ok(Value::Boolean(s.is_empty())),
            Some(Value::Array(a)) => Ok(Value::Boolean(a.is_empty())),
            Some(Value::Table(t)) => Ok(Value::Boolean(t.is_empty())),
            Some(Value::Empty) => Ok(Value::Boolean(true)),
            None => Ok(Value::Boolean(true)),
            _ => Ok(Value::Boolean(false)),
        }
    }
    fn is_uuid(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let clean: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            Ok(Value::Boolean(clean.len() == 32))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_phone(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
            Ok(Value::Boolean(digits.len() >= 10 && digits.len() <= 15))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_creditcard(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let digits: Vec<u32> = s.chars().filter(|c| c.is_ascii_digit())
                .filter_map(|c| c.to_digit(10)).collect();
            if digits.len() < 13 || digits.len() > 19 {
                return Ok(Value::Boolean(false));
            }
            let mut sum = 0;
            let mut double = false;
            for &digit in digits.iter().rev() {
                let mut d = digit;
                if double {
                    d *= 2;
                    if d > 9 { d -= 9; }
                }
                sum += d;
                double = !double;
            }
            Ok(Value::Boolean(sum % 10 == 0))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_hex(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let s = s.trim_start_matches("0x").trim_start_matches("0X").trim_start_matches('#');
            Ok(Value::Boolean(!s.is_empty() && s.chars().all(|c| c.is_ascii_hexdigit())))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn is_json(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let trimmed = s.trim();
            let valid = (trimmed.starts_with('{') && trimmed.ends_with('}')) ||
                       (trimmed.starts_with('[') && trimmed.ends_with(']'));
            Ok(Value::Boolean(valid))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn check_length(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 { return Ok(Value::Boolean(false)); }
        let len = match &args[0] {
            Value::String(s) => s.len(),
            Value::Array(a) => a.len(),
            _ => return Ok(Value::Boolean(false)),
        };
        let min = match &args[1] {
            Value::Number(n) => *n as usize,
            _ => 0,
        };
        let max = match args.get(2) {
            Some(Value::Number(n)) => *n as usize,
            _ => usize::MAX,
        };
        Ok(Value::Boolean(len >= min && len <= max))
    }
    fn check_range(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 3 { return Ok(Value::Boolean(false)); }
        let val = match &args[0] {
            Value::Number(n) => *n,
            _ => return Ok(Value::Boolean(false)),
        };
        let min = match &args[1] {
            Value::Number(n) => *n,
            _ => return Ok(Value::Boolean(false)),
        };
        let max = match &args[2] {
            Value::Number(n) => *n,
            _ => return Ok(Value::Boolean(false)),
        };
        Ok(Value::Boolean(val >= min && val <= max))
    }
    fn matches_pattern(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 { return Ok(Value::Boolean(false)); }
        if let (Some(Value::String(s)), Some(Value::String(pattern))) = (args.get(0), args.get(1)) {
            if pattern == "*" {
                return Ok(Value::Boolean(true));
            }
            if pattern.starts_with('*') && pattern.ends_with('*') {
                let inner = &pattern[1..pattern.len()-1];
                return Ok(Value::Boolean(s.contains(inner)));
            }
            if pattern.starts_with('*') {
                return Ok(Value::Boolean(s.ends_with(&pattern[1..])));
            }
            if pattern.ends_with('*') {
                return Ok(Value::Boolean(s.starts_with(&pattern[..pattern.len()-1])));
            }
            Ok(Value::Boolean(s == pattern))
        } else {
            Ok(Value::Boolean(false))
        }
    }
}