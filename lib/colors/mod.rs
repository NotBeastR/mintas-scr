use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
pub struct ColorsModule;
impl ColorsModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "red" => Self::color(args, "31"),
            "green" => Self::color(args, "32"),
            "yellow" => Self::color(args, "33"),
            "blue" => Self::color(args, "34"),
            "magenta" => Self::color(args, "35"),
            "cyan" => Self::color(args, "36"),
            "white" => Self::color(args, "37"),
            "black" => Self::color(args, "30"),
            "bold" => Self::style(args, "1"),
            "dim" => Self::style(args, "2"),
            "italic" => Self::style(args, "3"),
            "underline" => Self::style(args, "4"),
            "blink" => Self::style(args, "5"),
            "inverse" => Self::style(args, "7"),
            "hidden" => Self::style(args, "8"),
            "strike" => Self::style(args, "9"),
            "reset" => Ok(Value::String("\x1b[0m".to_string())),
            "bg_red" => Self::color(args, "41"),
            "bg_green" => Self::color(args, "42"),
            "bg_yellow" => Self::color(args, "43"),
            "bg_blue" => Self::color(args, "44"),
            "bg_magenta" => Self::color(args, "45"),
            "bg_cyan" => Self::color(args, "46"),
            "bg_white" => Self::color(args, "47"),
            "rgb" => Self::rgb(args),
            "hex" => Self::hex(args),
            "strip" => Self::strip(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown colors function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn color(args: &[Value], code: &str) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            Ok(Value::String(format!("\x1b[{}m{}\x1b[0m", code, s)))
        } else {
            Ok(Value::String(format!("\x1b[{}m", code)))
        }
    }
    fn style(args: &[Value], code: &str) -> MintasResult<Value> {
        Self::color(args, code)
    }
    fn rgb(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 4 {
            return Ok(Value::String(String::new()));
        }
        let text = match &args[0] { Value::String(s) => s.clone(), _ => return Ok(Value::String(String::new())) };
        let r = match &args[1] { Value::Number(n) => *n as u8, _ => 0 };
        let g = match &args[2] { Value::Number(n) => *n as u8, _ => 0 };
        let b = match &args[3] { Value::Number(n) => *n as u8, _ => 0 };
        Ok(Value::String(format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)))
    }
    fn hex(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Ok(Value::String(String::new()));
        }
        let text = match &args[0] { Value::String(s) => s.clone(), _ => return Ok(Value::String(String::new())) };
        let hex = match &args[1] { Value::String(s) => s.trim_start_matches('#').to_string(), _ => return Ok(Value::String(String::new())) };
        if hex.len() >= 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            Ok(Value::String(format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)))
        } else {
            Ok(Value::String(text))
        }
    }
    fn strip(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let mut result = String::new();
            let mut in_escape = false;
            for c in s.chars() {
                if c == '\x1b' {
                    in_escape = true;
                } else if in_escape {
                    if c == 'm' { in_escape = false; }
                } else {
                    result.push(c);
                }
            }
            Ok(Value::String(result))
        } else {
            Ok(Value::String(String::new()))
        }
    }
}