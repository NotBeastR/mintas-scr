use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct SmtpModule;
impl SmtpModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "send" => Self::send_email(args),
            "connect" => Self::connect(args),
            "test" => Self::test_connection(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown smtp function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    #[cfg(feature = "smtp")]
    fn send_email(args: &[Value]) -> MintasResult<Value> {
        use lettre::{Message, SmtpTransport, Transport};
        use lettre::transport::smtp::authentication::Credentials;
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "smtp.send".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let config = match &args[0] {
            Value::Table(map) => map,
            _ => return Err(MintasError::TypeError {
                message: "smtp.send requires a table argument".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let host = Self::get_string(config, "host", "smtp.gmail.com");
        let port = Self::get_number(config, "port", 587.0) as u16;
        let user = Self::get_string(config, "user", "");
        let pass = Self::get_string(config, "pass", "");
        let from = Self::get_string(config, "from", &user);
        let to = Self::get_string(config, "to", "");
        let subject = Self::get_string(config, "subject", "No Subject");
        let body = Self::get_string(config, "body", "");
        if to.is_empty() {
            return Err(MintasError::RuntimeError {
                message: "smtp.send: 'to' address is required".to_string(),
                location: SourceLocation::new(0, 0),
            });
        }
        let email = Message::builder()
            .from(from.parse().map_err(|e| MintasError::RuntimeError {
                message: format!("Invalid 'from' address: {}", e),
                location: SourceLocation::new(0, 0),
            })?)
            .to(to.parse().map_err(|e| MintasError::RuntimeError {
                message: format!("Invalid 'to' address: {}", e),
                location: SourceLocation::new(0, 0),
            })?)
            .subject(subject)
            .body(body)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("Failed to build email: {}", e),
                location: SourceLocation::new(0, 0),
            })?;
        let creds = Credentials::new(user, pass);
        let mailer = SmtpTransport::starttls_relay(&host)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("Failed to connect to SMTP server: {}", e),
                location: SourceLocation::new(0, 0),
            })?
            .port(port)
            .credentials(creds)
            .build();
        match mailer.send(&email) {
            Ok(_) => {
                let mut result = HashMap::new();
                result.insert("success".to_string(), Value::Boolean(true));
                result.insert("message".to_string(), Value::String("Email sent successfully".to_string()));
                Ok(Value::Table(result))
            }
            Err(e) => {
                let mut result = HashMap::new();
                result.insert("success".to_string(), Value::Boolean(false));
                result.insert("error".to_string(), Value::String(format!("{}", e)));
                Ok(Value::Table(result))
            }
        }
    }
    #[cfg(not(feature = "smtp"))]
    fn send_email(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "SMTP not available. Build with: cargo build --features smtp".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    fn connect(args: &[Value]) -> MintasResult<Value> {
        let mut config = HashMap::new();
        if let Some(Value::String(host)) = args.get(0) {
            config.insert("host".to_string(), Value::String(host.clone()));
        }
        if let Some(Value::Number(port)) = args.get(1) {
            config.insert("port".to_string(), Value::Number(*port));
        }
        config.insert("__type__".to_string(), Value::String("SmtpConfig".to_string()));
        Ok(Value::Table(config))
    }
    #[cfg(feature = "smtp")]
    fn test_connection(args: &[Value]) -> MintasResult<Value> {
        use lettre::SmtpTransport;
        use lettre::transport::smtp::authentication::Credentials;
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "smtp.test".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let config = match &args[0] {
            Value::Table(map) => map,
            _ => return Err(MintasError::TypeError {
                message: "smtp.test requires a table argument".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let host = Self::get_string(config, "host", "smtp.gmail.com");
        let user = Self::get_string(config, "user", "");
        let pass = Self::get_string(config, "pass", "");
        let creds = Credentials::new(user, pass);
        match SmtpTransport::starttls_relay(&host) {
            Ok(transport) => {
                let mailer = transport.credentials(creds).build();
                match mailer.test_connection() {
                    Ok(true) => {
                        let mut result = HashMap::new();
                        result.insert("connected".to_string(), Value::Boolean(true));
                        result.insert("host".to_string(), Value::String(host));
                        Ok(Value::Table(result))
                    }
                    _ => {
                        let mut result = HashMap::new();
                        result.insert("connected".to_string(), Value::Boolean(false));
                        result.insert("error".to_string(), Value::String("Connection test failed".to_string()));
                        Ok(Value::Table(result))
                    }
                }
            }
            Err(e) => {
                let mut result = HashMap::new();
                result.insert("connected".to_string(), Value::Boolean(false));
                result.insert("error".to_string(), Value::String(format!("{}", e)));
                Ok(Value::Table(result))
            }
        }
    }
    #[cfg(not(feature = "smtp"))]
    fn test_connection(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "SMTP not available. Build with: cargo build --features smtp".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    fn get_string(map: &HashMap<String, Value>, key: &str, default: &str) -> String {
        match map.get(key) {
            Some(Value::String(s)) => s.clone(),
            _ => default.to_string(),
        }
    }
    fn get_number(map: &HashMap<String, Value>, key: &str, default: f64) -> f64 {
        match map.get(key) {
            Some(Value::Number(n)) => *n,
            _ => default,
        }
    }
}