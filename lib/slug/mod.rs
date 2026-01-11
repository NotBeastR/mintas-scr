use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
pub struct SlugModule;
impl SlugModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "make" | "create" | "slugify" => Self::slugify(args),
            "valid" | "is_valid" => Self::is_valid(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown slug function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn slugify(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let separator = match args.get(1) {
                Some(Value::String(sep)) => sep.chars().next().unwrap_or('-'),
                _ => '-',
            };
            let slug: String = s
                .to_lowercase()
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() { c }
                    else if c.is_whitespace() || c == '_' || c == '-' { separator }
                    else { ' ' }
                })
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(&separator.to_string());
            let mut result = String::new();
            let mut last_was_sep = true;
            for c in slug.chars() {
                if c == separator {
                    if !last_was_sep {
                        result.push(c);
                        last_was_sep = true;
                    }
                } else {
                    result.push(c);
                    last_was_sep = false;
                }
            }
            Ok(Value::String(result.trim_matches(separator).to_string()))
        } else {
            Ok(Value::String(String::new()))
        }
    }
    fn is_valid(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::String(s)) = args.get(0) {
            let valid = !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            Ok(Value::Boolean(valid))
        } else {
            Ok(Value::Boolean(false))
        }
    }
}