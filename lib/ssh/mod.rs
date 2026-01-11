use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct SshModule;
impl SshModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "connect" => Self::connect(args),
            "exec" | "run" => Self::execute(args),
            "upload" => Self::upload_file(args),
            "download" => Self::download_file(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown ssh function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    #[cfg(feature = "ssh")]
    fn connect(args: &[Value]) -> MintasResult<Value> {
        use ssh2::Session;
        use std::net::TcpStream;
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "ssh.connect".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let config = match &args[0] {
            Value::Table(map) => map,
            _ => return Err(MintasError::TypeError {
                message: "ssh.connect requires a table argument".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let host = Self::get_string(config, "host", "localhost");
        let port = Self::get_number(config, "port", 22.0) as u16;
        let user = Self::get_string(config, "user", "root");
        let pass = Self::get_string(config, "pass", "");
        let key_path = Self::get_string(config, "key", "");
        let addr = format!("{}:{}", host, port);
        match TcpStream::connect(&addr) {
            Ok(tcp) => {
                match Session::new() {
                    Ok(mut sess) => {
                        sess.set_tcp_stream(tcp);
                        if sess.handshake().is_err() {
                            return Err(MintasError::RuntimeError {
                                message: "SSH handshake failed".to_string(),
                                location: SourceLocation::new(0, 0),
                            });
                        }
                        let auth_result = if !key_path.is_empty() {
                            sess.userauth_pubkey_file(&user, None, std::path::Path::new(&key_path), None)
                        } else if !pass.is_empty() {
                            sess.userauth_password(&user, &pass)
                        } else {
                            sess.userauth_agent(&user)
                        };
                        match auth_result {
                            Ok(_) => {
                                let mut result = HashMap::new();
                                result.insert("connected".to_string(), Value::Boolean(true));
                                result.insert("host".to_string(), Value::String(host.clone()));
                                result.insert("user".to_string(), Value::String(user.clone()));
                                result.insert("__type__".to_string(), Value::String("SshConnection".to_string()));
                                result.insert("__host__".to_string(), Value::String(addr));
                                result.insert("__user__".to_string(), Value::String(user));
                                result.insert("__pass__".to_string(), Value::String(pass));
                                result.insert("__key__".to_string(), Value::String(key_path));
                                Ok(Value::Table(result))
                            }
                            Err(e) => {
                                let mut result = HashMap::new();
                                result.insert("connected".to_string(), Value::Boolean(false));
                                result.insert("error".to_string(), Value::String(format!("Auth failed: {}", e)));
                                Ok(Value::Table(result))
                            }
                        }
                    }
                    Err(e) => {
                        let mut result = HashMap::new();
                        result.insert("connected".to_string(), Value::Boolean(false));
                        result.insert("error".to_string(), Value::String(format!("Session error: {}", e)));
                        Ok(Value::Table(result))
                    }
                }
            }
            Err(e) => {
                let mut result = HashMap::new();
                result.insert("connected".to_string(), Value::Boolean(false));
                result.insert("error".to_string(), Value::String(format!("Connection failed: {}", e)));
                Ok(Value::Table(result))
            }
        }
    }
    #[cfg(not(feature = "ssh"))]
    fn connect(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "SSH not available. Build with: cargo build --features ssh".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    #[cfg(feature = "ssh")]
    fn execute(args: &[Value]) -> MintasResult<Value> {
        use ssh2::Session;
        use std::net::TcpStream;
        use std::io::Read;
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "ssh.exec".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let conn = match &args[0] {
            Value::Table(map) => map,
            _ => return Err(MintasError::TypeError {
                message: "ssh.exec requires a connection table".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let command = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Command must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let addr = Self::get_string(conn, "__host__", "");
        let user = Self::get_string(conn, "__user__", "root");
        let pass = Self::get_string(conn, "__pass__", "");
        let key_path = Self::get_string(conn, "__key__", "");
        match TcpStream::connect(&addr) {
            Ok(tcp) => {
                match Session::new() {
                    Ok(mut sess) => {
                        sess.set_tcp_stream(tcp);
                        if sess.handshake().is_err() {
                            return Err(MintasError::RuntimeError {
                                message: "SSH handshake failed".to_string(),
                                location: SourceLocation::new(0, 0),
                            });
                        }
                        let auth_result = if !key_path.is_empty() {
                            sess.userauth_pubkey_file(&user, None, std::path::Path::new(&key_path), None)
                        } else if !pass.is_empty() {
                            sess.userauth_password(&user, &pass)
                        } else {
                            sess.userauth_agent(&user)
                        };
                        if auth_result.is_err() {
                            return Err(MintasError::RuntimeError {
                                message: "SSH authentication failed".to_string(),
                                location: SourceLocation::new(0, 0),
                            });
                        }
                        match sess.channel_session() {
                            Ok(mut channel) => {
                                if channel.exec(&command).is_err() {
                                    return Err(MintasError::RuntimeError {
                                        message: "Failed to execute command".to_string(),
                                        location: SourceLocation::new(0, 0),
                                    });
                                }
                                let mut stdout = String::new();
                                let mut stderr = String::new();
                                let _ = channel.read_to_string(&mut stdout);
                                let _ = channel.stderr().read_to_string(&mut stderr);
                                let _ = channel.wait_close();
                                let exit_code = channel.exit_status().unwrap_or(-1);
                                let mut result = HashMap::new();
                                result.insert("stdout".to_string(), Value::String(stdout));
                                result.insert("stderr".to_string(), Value::String(stderr));
                                result.insert("exit_code".to_string(), Value::Number(exit_code as f64));
                                result.insert("success".to_string(), Value::Boolean(exit_code == 0));
                                Ok(Value::Table(result))
                            }
                            Err(e) => Err(MintasError::RuntimeError {
                                message: format!("Channel error: {}", e),
                                location: SourceLocation::new(0, 0),
                            }),
                        }
                    }
                    Err(e) => Err(MintasError::RuntimeError {
                        message: format!("Session error: {}", e),
                        location: SourceLocation::new(0, 0),
                    }),
                }
            }
            Err(e) => Err(MintasError::RuntimeError {
                message: format!("Connection failed: {}", e),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    #[cfg(not(feature = "ssh"))]
    fn execute(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "SSH not available. Build with: cargo build --features ssh".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    fn upload_file(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "ssh.upload not yet implemented - use ftp module for file transfers".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    fn download_file(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "ssh.download not yet implemented - use ftp module for file transfers".to_string(),
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