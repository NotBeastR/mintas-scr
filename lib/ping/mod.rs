use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;
pub struct PingModule;
impl PingModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "host" | "ping" => Self::ping_host(args),
            "check" => Self::check_host(args),
            "port" => Self::check_port(args),
            "trace" => Self::traceroute(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown ping function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn ping_host(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "ping.host".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let host = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "ping.host requires a hostname string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let count = match args.get(1) {
            Some(Value::Number(n)) => *n as u32,
            _ => 4,
        };
        let start = Instant::now();
        #[cfg(target_os = "windows")]
        let output = Command::new("ping")
            .args(["-n", &count.to_string(), &host])
            .output();
        #[cfg(not(target_os = "windows"))]
        let output = Command::new("ping")
            .args(["-c", &count.to_string(), &host])
            .output();
        let elapsed = start.elapsed().as_millis() as f64;
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let success = out.status.success();
                let mut avg_time = 0.0;
                let mut packet_loss = 0.0;
                for line in stdout.lines() {
                    let line_lower = line.to_lowercase();
                    if line_lower.contains("average") || line_lower.contains("avg") {
                        if let Some(ms_pos) = line.find("ms") {
                            let before_ms = &line[..ms_pos];
                            let parts: Vec<&str> = before_ms.split(|c: char| !c.is_numeric() && c != '.').collect();
                            for part in parts.iter().rev() {
                                if let Ok(n) = part.parse::<f64>() {
                                    avg_time = n;
                                    break;
                                }
                            }
                        }
                    }
                    if line_lower.contains("loss") || line_lower.contains("lost") {
                        if let Some(pct_pos) = line.find('%') {
                            let before_pct = &line[..pct_pos];
                            let parts: Vec<&str> = before_pct.split(|c: char| !c.is_numeric()).collect();
                            for part in parts.iter().rev() {
                                if let Ok(n) = part.parse::<f64>() {
                                    packet_loss = n;
                                    break;
                                }
                            }
                        }
                    }
                }
                let mut result = HashMap::new();
                result.insert("host".to_string(), Value::String(host));
                result.insert("alive".to_string(), Value::Boolean(success && packet_loss < 100.0));
                result.insert("time_ms".to_string(), Value::Number(avg_time));
                result.insert("total_ms".to_string(), Value::Number(elapsed));
                result.insert("packet_loss".to_string(), Value::Number(packet_loss));
                result.insert("count".to_string(), Value::Number(count as f64));
                result.insert("output".to_string(), Value::String(stdout.to_string()));
                Ok(Value::Table(result))
            }
            Err(e) => {
                let mut result = HashMap::new();
                result.insert("host".to_string(), Value::String(host));
                result.insert("alive".to_string(), Value::Boolean(false));
                result.insert("error".to_string(), Value::String(format!("{}", e)));
                Ok(Value::Table(result))
            }
        }
    }
    fn check_host(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "ping.check".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let host = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "ping.check requires a hostname string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        #[cfg(target_os = "windows")]
        let output = Command::new("ping")
            .args(["-n", "1", "-w", "1000", &host])
            .output();
        #[cfg(not(target_os = "windows"))]
        let output = Command::new("ping")
            .args(["-c", "1", "-W", "1", &host])
            .output();
        match output {
            Ok(out) => Ok(Value::Boolean(out.status.success())),
            Err(_) => Ok(Value::Boolean(false)),
        }
    }
    fn check_port(args: &[Value]) -> MintasResult<Value> {
        use std::net::{TcpStream, ToSocketAddrs};
        use std::time::Duration;
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "ping.port".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let host = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "ping.port requires a hostname string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let port = match &args[1] {
            Value::Number(n) => *n as u16,
            _ => return Err(MintasError::TypeError {
                message: "ping.port requires a port number".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let timeout = match args.get(2) {
            Some(Value::Number(n)) => Duration::from_millis(*n as u64),
            _ => Duration::from_secs(5),
        };
        let addr_str = format!("{}:{}", host, port);
        let start = Instant::now();
        match addr_str.to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    match TcpStream::connect_timeout(&addr, timeout) {
                        Ok(_) => {
                            let elapsed = start.elapsed().as_millis() as f64;
                            let mut result = HashMap::new();
                            result.insert("host".to_string(), Value::String(host));
                            result.insert("port".to_string(), Value::Number(port as f64));
                            result.insert("open".to_string(), Value::Boolean(true));
                            result.insert("time_ms".to_string(), Value::Number(elapsed));
                            Ok(Value::Table(result))
                        }
                        Err(e) => {
                            let mut result = HashMap::new();
                            result.insert("host".to_string(), Value::String(host));
                            result.insert("port".to_string(), Value::Number(port as f64));
                            result.insert("open".to_string(), Value::Boolean(false));
                            result.insert("error".to_string(), Value::String(format!("{}", e)));
                            Ok(Value::Table(result))
                        }
                    }
                } else {
                    let mut result = HashMap::new();
                    result.insert("host".to_string(), Value::String(host));
                    result.insert("port".to_string(), Value::Number(port as f64));
                    result.insert("open".to_string(), Value::Boolean(false));
                    result.insert("error".to_string(), Value::String("No addresses found".to_string()));
                    Ok(Value::Table(result))
                }
            }
            Err(e) => {
                let mut result = HashMap::new();
                result.insert("host".to_string(), Value::String(host));
                result.insert("port".to_string(), Value::Number(port as f64));
                result.insert("open".to_string(), Value::Boolean(false));
                result.insert("error".to_string(), Value::String(format!("DNS error: {}", e)));
                Ok(Value::Table(result))
            }
        }
    }
    fn traceroute(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "ping.trace".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let host = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "ping.trace requires a hostname string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        #[cfg(target_os = "windows")]
        let output = Command::new("tracert")
            .args(["-d", "-h", "15", &host])
            .output();
        #[cfg(not(target_os = "windows"))]
        let output = Command::new("traceroute")
            .args(["-n", "-m", "15", &host])
            .output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut hops = Vec::new();
                for line in stdout.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() { continue; }
                    let first_char = trimmed.chars().next().unwrap_or(' ');
                    if first_char.is_numeric() {
                        hops.push(Value::String(trimmed.to_string()));
                    }
                }
                let mut result = HashMap::new();
                result.insert("host".to_string(), Value::String(host));
                result.insert("hops".to_string(), Value::Array(hops));
                result.insert("output".to_string(), Value::String(stdout.to_string()));
                Ok(Value::Table(result))
            }
            Err(e) => {
                let mut result = HashMap::new();
                result.insert("host".to_string(), Value::String(host));
                result.insert("error".to_string(), Value::String(format!("{}", e)));
                Ok(Value::Table(result))
            }
        }
    }
}