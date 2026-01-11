use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct DnsModule;
impl DnsModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "lookup" => Self::lookup(args),
            "resolve" => Self::resolve(args),
            "reverse" => Self::reverse_lookup(args),
            "mx" => Self::mx_lookup(args),
            "txt" => Self::txt_lookup(args),
            "ns" => Self::ns_lookup(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown dns function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    #[cfg(feature = "dns")]
    fn lookup(args: &[Value]) -> MintasResult<Value> {
        use std::net::ToSocketAddrs;
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "dns.lookup".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let hostname = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "dns.lookup requires a string hostname".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let addr = format!("{}:80", hostname);
        match addr.to_socket_addrs() {
            Ok(addrs) => {
                let ips: Vec<Value> = addrs
                    .map(|a| Value::String(a.ip().to_string()))
                    .collect();
                let mut result = HashMap::new();
                result.insert("hostname".to_string(), Value::String(hostname));
                result.insert("addresses".to_string(), Value::Array(ips.clone()));
                if let Some(Value::String(ip)) = ips.first() {
                    result.insert("ip".to_string(), Value::String(ip.clone()));
                }
                Ok(Value::Table(result))
            }
            Err(e) => {
                let mut result = HashMap::new();
                result.insert("hostname".to_string(), Value::String(hostname));
                result.insert("error".to_string(), Value::String(format!("{}", e)));
                result.insert("addresses".to_string(), Value::Array(vec![]));
                Ok(Value::Table(result))
            }
        }
    }
    #[cfg(not(feature = "dns"))]
    fn lookup(args: &[Value]) -> MintasResult<Value> {
        use std::net::ToSocketAddrs;
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "dns.lookup".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let hostname = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "dns.lookup requires a string hostname".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let addr = format!("{}:80", hostname);
        match addr.to_socket_addrs() {
            Ok(addrs) => {
                let ips: Vec<Value> = addrs
                    .map(|a| Value::String(a.ip().to_string()))
                    .collect();
                let mut result = HashMap::new();
                result.insert("hostname".to_string(), Value::String(hostname));
                result.insert("addresses".to_string(), Value::Array(ips.clone()));
                if let Some(Value::String(ip)) = ips.first() {
                    result.insert("ip".to_string(), Value::String(ip.clone()));
                }
                Ok(Value::Table(result))
            }
            Err(e) => {
                let mut result = HashMap::new();
                result.insert("hostname".to_string(), Value::String(hostname));
                result.insert("error".to_string(), Value::String(format!("{}", e)));
                Ok(Value::Table(result))
            }
        }
    }
    fn resolve(args: &[Value]) -> MintasResult<Value> {
        Self::lookup(args)
    }
    fn reverse_lookup(args: &[Value]) -> MintasResult<Value> {
        use std::net::IpAddr;
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "dns.reverse".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let ip_str = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "dns.reverse requires an IP address string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        match ip_str.parse::<IpAddr>() {
            Ok(_ip) => {
                let mut result = HashMap::new();
                result.insert("ip".to_string(), Value::String(ip_str));
                result.insert("hostname".to_string(), Value::String("reverse lookup requires dns feature".to_string()));
                Ok(Value::Table(result))
            }
            Err(e) => {
                let mut result = HashMap::new();
                result.insert("error".to_string(), Value::String(format!("Invalid IP: {}", e)));
                Ok(Value::Table(result))
            }
        }
    }
    fn mx_lookup(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "dns.mx".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let domain = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "dns.mx requires a domain string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let mut result = HashMap::new();
        result.insert("domain".to_string(), Value::String(domain));
        result.insert("records".to_string(), Value::Array(vec![]));
        result.insert("note".to_string(), Value::String("MX lookup requires dns feature".to_string()));
        Ok(Value::Table(result))
    }
    fn txt_lookup(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "dns.txt".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let domain = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "dns.txt requires a domain string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let mut result = HashMap::new();
        result.insert("domain".to_string(), Value::String(domain));
        result.insert("records".to_string(), Value::Array(vec![]));
        Ok(Value::Table(result))
    }
    fn ns_lookup(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "dns.ns".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let domain = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "dns.ns requires a domain string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let mut result = HashMap::new();
        result.insert("domain".to_string(), Value::String(domain));
        result.insert("nameservers".to_string(), Value::Array(vec![]));
        Ok(Value::Table(result))
    }
}