use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct FtpModule;
impl FtpModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "connect" => Self::connect(args),
            "list" => Self::list_files(args),
            "download" => Self::download(args),
            "upload" => Self::upload(args),
            "delete" => Self::delete_file(args),
            "mkdir" => Self::make_dir(args),
            "pwd" => Self::print_working_dir(args),
            "cd" => Self::change_dir(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown ftp function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    #[cfg(feature = "ftp")]
    fn connect(args: &[Value]) -> MintasResult<Value> {
        use suppaftp::FtpStream;
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "ftp.connect".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let config = match &args[0] {
            Value::Table(map) => map,
            _ => return Err(MintasError::TypeError {
                message: "ftp.connect requires a table argument".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let host = Self::get_string(config, "host", "localhost");
        let port = Self::get_number(config, "port", 21.0) as u16;
        let user = Self::get_string(config, "user", "anonymous");
        let pass = Self::get_string(config, "pass", "");
        let addr = format!("{}:{}", host, port);
        match FtpStream::connect(&addr) {
            Ok(mut ftp) => {
                match ftp.login(&user, &pass) {
                    Ok(_) => {
                        let mut result = HashMap::new();
                        result.insert("connected".to_string(), Value::Boolean(true));
                        result.insert("host".to_string(), Value::String(host));
                        result.insert("user".to_string(), Value::String(user.clone()));
                        result.insert("__type__".to_string(), Value::String("FtpConnection".to_string()));
                        result.insert("__host__".to_string(), Value::String(addr));
                        result.insert("__user__".to_string(), Value::String(user));
                        result.insert("__pass__".to_string(), Value::String(pass));
                        let _ = ftp.quit();
                        Ok(Value::Table(result))
                    }
                    Err(e) => {
                        let mut result = HashMap::new();
                        result.insert("connected".to_string(), Value::Boolean(false));
                        result.insert("error".to_string(), Value::String(format!("Login failed: {}", e)));
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
    #[cfg(not(feature = "ftp"))]
    fn connect(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "FTP not available. Build with: cargo build --features ftp".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    #[cfg(feature = "ftp")]
    fn list_files(args: &[Value]) -> MintasResult<Value> {
        use suppaftp::FtpStream;
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "ftp.list".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let conn = match &args[0] {
            Value::Table(map) => map,
            _ => return Err(MintasError::TypeError {
                message: "ftp.list requires a connection table".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let path = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            _ => ".".to_string(),
        };
        let host = Self::get_string(conn, "__host__", "");
        let user = Self::get_string(conn, "__user__", "anonymous");
        let pass = Self::get_string(conn, "__pass__", "");
        match FtpStream::connect(&host) {
            Ok(mut ftp) => {
                if ftp.login(&user, &pass).is_err() {
                    return Err(MintasError::RuntimeError {
                        message: "FTP login failed".to_string(),
                        location: SourceLocation::new(0, 0),
                    });
                }
                match ftp.nlst(Some(&path)) {
                    Ok(files) => {
                        let file_list: Vec<Value> = files.iter()
                            .map(|f| Value::String(f.clone()))
                            .collect();
                        let _ = ftp.quit();
                        let mut result = HashMap::new();
                        result.insert("path".to_string(), Value::String(path));
                        result.insert("files".to_string(), Value::Array(file_list));
                        Ok(Value::Table(result))
                    }
                    Err(e) => {
                        let _ = ftp.quit();
                        Err(MintasError::RuntimeError {
                            message: format!("Failed to list files: {}", e),
                            location: SourceLocation::new(0, 0),
                        })
                    }
                }
            }
            Err(e) => Err(MintasError::RuntimeError {
                message: format!("Connection failed: {}", e),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    #[cfg(not(feature = "ftp"))]
    fn list_files(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "FTP not available. Build with: cargo build --features ftp".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    #[cfg(feature = "ftp")]
    fn download(args: &[Value]) -> MintasResult<Value> {
        use suppaftp::FtpStream;
        use std::io::Read;
        if args.len() < 3 {
            return Err(MintasError::InvalidArgumentCount {
                function: "ftp.download".to_string(),
                expected: 3,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let conn = match &args[0] {
            Value::Table(map) => map,
            _ => return Err(MintasError::TypeError {
                message: "ftp.download requires a connection table".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let remote_path = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Remote path must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let local_path = match &args[2] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Local path must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let host = Self::get_string(conn, "__host__", "");
        let user = Self::get_string(conn, "__user__", "anonymous");
        let pass = Self::get_string(conn, "__pass__", "");
        match FtpStream::connect(&host) {
            Ok(mut ftp) => {
                if ftp.login(&user, &pass).is_err() {
                    return Err(MintasError::RuntimeError {
                        message: "FTP login failed".to_string(),
                        location: SourceLocation::new(0, 0),
                    });
                }
                match ftp.retr_as_buffer(&remote_path) {
                    Ok(mut cursor) => {
                        let mut buffer = Vec::new();
                        if cursor.read_to_end(&mut buffer).is_ok() {
                            if std::fs::write(&local_path, &buffer).is_ok() {
                                let _ = ftp.quit();
                                let mut result = HashMap::new();
                                result.insert("success".to_string(), Value::Boolean(true));
                                result.insert("remote".to_string(), Value::String(remote_path));
                                result.insert("local".to_string(), Value::String(local_path));
                                result.insert("size".to_string(), Value::Number(buffer.len() as f64));
                                return Ok(Value::Table(result));
                            }
                        }
                        let _ = ftp.quit();
                        Err(MintasError::RuntimeError {
                            message: "Failed to save file".to_string(),
                            location: SourceLocation::new(0, 0),
                        })
                    }
                    Err(e) => {
                        let _ = ftp.quit();
                        Err(MintasError::RuntimeError {
                            message: format!("Download failed: {}", e),
                            location: SourceLocation::new(0, 0),
                        })
                    }
                }
            }
            Err(e) => Err(MintasError::RuntimeError {
                message: format!("Connection failed: {}", e),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    #[cfg(not(feature = "ftp"))]
    fn download(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "FTP not available. Build with: cargo build --features ftp".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    #[cfg(feature = "ftp")]
    fn upload(args: &[Value]) -> MintasResult<Value> {
        use suppaftp::FtpStream;
        use std::io::Cursor;
        if args.len() < 3 {
            return Err(MintasError::InvalidArgumentCount {
                function: "ftp.upload".to_string(),
                expected: 3,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let conn = match &args[0] {
            Value::Table(map) => map,
            _ => return Err(MintasError::TypeError {
                message: "ftp.upload requires a connection table".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let local_path = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Local path must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let remote_path = match &args[2] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Remote path must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let host = Self::get_string(conn, "__host__", "");
        let user = Self::get_string(conn, "__user__", "anonymous");
        let pass = Self::get_string(conn, "__pass__", "");
        let content = match std::fs::read(&local_path) {
            Ok(c) => c,
            Err(e) => return Err(MintasError::RuntimeError {
                message: format!("Failed to read local file: {}", e),
                location: SourceLocation::new(0, 0),
            }),
        };
        match FtpStream::connect(&host) {
            Ok(mut ftp) => {
                if ftp.login(&user, &pass).is_err() {
                    return Err(MintasError::RuntimeError {
                        message: "FTP login failed".to_string(),
                        location: SourceLocation::new(0, 0),
                    });
                }
                let mut cursor = Cursor::new(content.clone());
                match ftp.put_file(&remote_path, &mut cursor) {
                    Ok(_) => {
                        let _ = ftp.quit();
                        let mut result = HashMap::new();
                        result.insert("success".to_string(), Value::Boolean(true));
                        result.insert("local".to_string(), Value::String(local_path));
                        result.insert("remote".to_string(), Value::String(remote_path));
                        result.insert("size".to_string(), Value::Number(content.len() as f64));
                        Ok(Value::Table(result))
                    }
                    Err(e) => {
                        let _ = ftp.quit();
                        Err(MintasError::RuntimeError {
                            message: format!("Upload failed: {}", e),
                            location: SourceLocation::new(0, 0),
                        })
                    }
                }
            }
            Err(e) => Err(MintasError::RuntimeError {
                message: format!("Connection failed: {}", e),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    #[cfg(not(feature = "ftp"))]
    fn upload(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "FTP not available. Build with: cargo build --features ftp".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    fn delete_file(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "ftp.delete not yet implemented".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    fn make_dir(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "ftp.mkdir not yet implemented".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    fn print_working_dir(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "ftp.pwd not yet implemented".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    fn change_dir(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "ftp.cd not yet implemented".to_string(),
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