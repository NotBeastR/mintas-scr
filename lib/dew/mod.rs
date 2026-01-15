#![allow(dead_code)]
use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::Utc;
use serde_json;
use serde_json::{Value as JsonValue, Number as JsonNumber};
#[cfg(feature = "database")]
use rusqlite::{Connection, params};
#[cfg(feature = "database")]
use postgres::{Client, NoTls};
#[cfg(feature = "database")]
use redis::Commands;
#[cfg(feature = "magic")]
use uuid::Uuid;
#[cfg(feature = "magic")]
use bcrypt::{DEFAULT_COST, hash, verify};
#[cfg(feature = "magic")]
use sha2::{Sha256, Digest};
#[cfg(feature = "magic")]
use hmac::{Hmac, Mac};
#[cfg(feature = "magic")]
use hex;
#[cfg(feature = "magic")]
use csv;

fn value_to_json(v: &crate::evaluator::Value) -> JsonValue {
    use crate::evaluator::Value as V;
    match v {
        V::Number(n) => JsonNumber::from_f64(*n).map(JsonValue::Number).unwrap_or(JsonValue::Null),
        V::String(s) => JsonValue::String(s.clone()),
        V::Boolean(b) => JsonValue::Bool(*b),
        V::Null => JsonValue::Null,
        V::Array(arr) => JsonValue::Array(arr.iter().map(value_to_json).collect()),
        V::Table(t) => {
            let mut obj = serde_json::Map::new();
            for (k, vv) in t {
                obj.insert(k.clone(), value_to_json(vv));
            }
            JsonValue::Object(obj)
        }
        _ => JsonValue::Null,
    }
}

fn table_to_json_string(t: &HashMap<String, crate::evaluator::Value>) -> String {
    let mut obj = serde_json::Map::new();
    for (k, v) in t {
        obj.insert(k.clone(), value_to_json(v));
    }
    JsonValue::Object(obj).to_string()
}

pub struct DewModule;
impl DewModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "main" => Self::create_server(args),
            "serve" => Self::serve(args),
            "database" => Self::database(args),
            "query" => Self::query(args), // Added query function
            "use" => Self::use_middleware(args),
            "cors" => Self::cors(args),
            "auth" => Self::auth(args),
            "rate_limit" => Self::rate_limit(args),
            "compress" => Self::compress(args),
            "logger" => Self::logger(args),
            "static" => Self::static_files(args),
            "inview" => Self::inview(args),
            "render" => Self::render(args),
            "session" => Self::session(args),
            "session_set" => Self::session_set(args),
            "session_get" => Self::session_get(args),
            "session_destroy" => Self::session_destroy(args),
            "cookie" => Self::cookie(args),
            "set_cookie" => Self::set_cookie(args),
            "upload" => Self::upload(args),
            "save_upload" => Self::save_upload(args),
            "validate" => Self::validate(args),
            "websocket" => Self::websocket(args),
            "ws_send" => Self::ws_send(args),
            "ws_broadcast" => Self::ws_broadcast(args),
            "test_get" => Self::test_get(args),
            "test_post" => Self::test_post(args),
            "test_put" => Self::test_put(args),
            "test_delete" => Self::test_delete(args),
            "config" => Self::config(args),
            "dotenv" => Self::dotenv(args),
            "env" => Self::env(args),
            "job" => Self::job(args),
            "queue" => Self::queue(args),
            "task" => Self::task(args),
            "schedule" => Self::schedule(args),
            "chunk_upload" => Self::chunk_upload(args),
            "chunk_complete" => Self::chunk_complete(args),
            "protect" => Self::protect(args),
            "csrf_token" => Self::csrf_token(args),
            "csrf_verify" => Self::csrf_verify(args),
            "sanitize" => Self::sanitize(args),
            "ws_on_connect" => Self::ws_on_connect(args),
            "ws_on_disconnect" => Self::ws_on_disconnect(args),
            "ws_on_message" => Self::ws_on_message(args),
            "ws_on_error" => Self::ws_on_error(args),
            "ws_join" => Self::ws_join(args),
            "ws_leave" => Self::ws_leave(args),
            "ws_room_broadcast" => Self::ws_room_broadcast(args),
            "ws_rooms" => Self::ws_rooms(args),
            "ws_clients" => Self::ws_clients(args),
            "text" => Self::response_text(args),
            "html" => Self::response_html(args),
            "json" => Self::response_json(args),
            "redirect" => Self::response_redirect(args),
            "file" => Self::response_file(args),
            // WebRTC Features
            "webrtc_peer" => Self::webrtc_peer(args),
            "webrtc_offer" => Self::webrtc_offer(args),
            "webrtc_answer" => Self::webrtc_answer(args),
            "webrtc_datachannel" => Self::webrtc_datachannel(args),
            "webrtc_send" => Self::webrtc_send(args),
            "webrtc_on_message" => Self::webrtc_on_message(args),
            "webrtc_close" => Self::webrtc_close(args),
            "webrtc_stats" => Self::webrtc_stats(args),
            // JavaScript Event Handling
            "js_onclick" => Self::js_onclick(args),
            "js_onchange" => Self::js_onchange(args),
            "js_oninput" => Self::js_oninput(args),
            "js_onsubmit" => Self::js_onsubmit(args),
            "js_onfocus" => Self::js_onfocus(args),
            "js_onblur" => Self::js_onblur(args),
            "js_onkeypress" => Self::js_onkeypress(args),
            "js_onkeydown" => Self::js_onkeydown(args),
            "js_onkeyup" => Self::js_onkeyup(args),
            "js_onmouseover" => Self::js_onmouseover(args),
            "js_onmouseout" => Self::js_onmouseout(args),
            "js_onload" => Self::js_onload(args),
            "js_query" => Self::js_query(args),
            "js_query_all" => Self::js_query_all(args),
            "js_set_attr" => Self::js_set_attr(args),
            "js_get_attr" => Self::js_get_attr(args),
            "js_set_class" => Self::js_set_class(args),
            "js_add_class" => Self::js_add_class(args),
            "js_remove_class" => Self::js_remove_class(args),
            "js_set_html" => Self::js_set_html(args),
            "js_set_text" => Self::js_set_text(args),
            "js_show" => Self::js_show(args),
            "js_hide" => Self::js_hide(args),
            "js_toggle" => Self::js_toggle(args),
            "js_validate_form" => Self::js_validate_form(args),
            // Magical Features
            "uuid" => Self::uuid(args),
            "hash_password" => Self::hash_password(args),
            "verify_password" => Self::verify_password(args),
            "sha256" => Self::sha256(args),
            "csv_parse" => Self::csv_parse(args),
            "csv_stringify" => Self::csv_stringify(args),
            "redis_get" => Self::redis_get(args),
            "redis_set" => Self::redis_set(args),
            _ => Err(MintasError::UnknownFunction {
                name: format!("dew.{}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn create_server(_args: &[Value]) -> MintasResult<Value> {
        let server = DewServer::new();
        let id = SERVERS.lock().unwrap().register(server);
        Ok(Value::Table({
            let mut map = HashMap::new();
            map.insert("__dew_server_id__".to_string(), Value::Number(id as f64));
            map.insert("__type__".to_string(), Value::String("DewServer".to_string()));
            map
        }))
    }
    fn serve(args: &[Value]) -> MintasResult<Value> {
        let (port, host, server_id, options) = if let Some(Value::Table(config)) = args.get(0) {
            let port = match config.get("port") {
                Some(Value::Number(p)) => *p as u16,
                _ => 3000,
            };
            let host = match config.get("ip").or(config.get("host")) {
                Some(Value::String(h)) => h.clone(),
                _ => "127.0.0.1".to_string(),
            };
            let server_id = match config.get("server_id") {
                Some(Value::Number(id)) => *id as usize,
                _ => 0,
            };
            (port, host, server_id, config.clone())
        } else {
            let port = match args.get(0) {
                Some(Value::Number(p)) => *p as u16,
                _ => 3000,
            };
            let host = match args.get(1) {
                Some(Value::String(h)) => h.clone(),
                _ => "127.0.0.1".to_string(),
            };
            let server_id = match args.get(2) {
                Some(Value::Number(id)) => *id as usize,
                _ => 0,
            };
            (port, host, server_id, HashMap::new())
        };
        let timeout = match options.get("timeout") {
            Some(Value::Number(t)) => Some(*t as u64),
            Some(Value::String(s)) => parse_duration_string(s),
            _ => None,
        };
        let debug = match options.get("debug") {
            Some(Value::Boolean(b)) => *b,
            _ => false,
        };
        let security = match options.get("security") {
            Some(Value::Boolean(b)) => *b,
            _ => true,
        };
        let fast_reload = match options.get("fast_reload").or(options.get("reload")) {
            Some(Value::Boolean(b)) => *b,
            _ => false,
        };
        let mut servers = SERVERS.lock().unwrap();
        if let Some(server) = servers.get_mut(server_id) {
            server.security.sql_injection_protection = security;
            server.security.xss_protection = security;
            server.security.csrf_protection = security;
            server.security.ddos_protection = security;
            server.config.insert("debug".to_string(), Value::Boolean(debug));
            server.config.insert("fast_reload".to_string(), Value::Boolean(fast_reload));
            if let Some(t) = timeout {
                server.config.insert("timeout".to_string(), Value::Number(t as f64));
            }
            if debug {
                println!("🐛 Debug mode enabled");
            }
            if fast_reload {
                println!("🔄 Fast reload enabled");
            }
            if let Some(t) = timeout {
                println!("⏱️  Request timeout: {}ms", t);
            }
            if !security {
                println!("⚠️  Security protections disabled");
            }
            let server_clone = server.clone();
            drop(servers); 
            return start_server(&server_clone, port, &host);
        }
        Err(MintasError::RuntimeError {
            message: "Server not found".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    fn database(args: &[Value]) -> MintasResult<Value> {
        let connection_string = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "sqlite:///app.db".to_string(),
        };
        let mut db = HashMap::new();
        db.insert("connection".to_string(), Value::String(connection_string.clone()));
        db.insert("__type__".to_string(), Value::String("DewDatabase".to_string()));
        if connection_string.starts_with("sqlite:") {
            db.insert("driver".to_string(), Value::String("sqlite".to_string()));
            let path = connection_string.trim_start_matches("sqlite:///");
            db.insert("path".to_string(), Value::String(path.to_string()));
        } else if connection_string.starts_with("postgres:") {
            db.insert("driver".to_string(), Value::String("postgres".to_string()));
        } else if connection_string.starts_with("mysql:") {
            db.insert("driver".to_string(), Value::String("mysql".to_string()));
        }
        Ok(Value::Table(db))
    }

    #[cfg(feature = "database")]
    fn query(args: &[Value]) -> MintasResult<Value> {
        let db_config = match args.get(0) {
            Some(Value::Table(t)) => t,
            _ => return Err(MintasError::RuntimeError {
                message: "Expected database config as first argument".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let sql = match args.get(1) {
            Some(Value::String(s)) => s,
            _ => return Err(MintasError::RuntimeError {
                message: "Expected SQL query string as second argument".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let params = match args.get(2) {
            Some(Value::Array(a)) => a,
            Some(Value::Table(_)) => return Err(MintasError::RuntimeError {
                message: "Named parameters not yet supported for SQLite query".to_string(),
                location: SourceLocation::new(0, 0),
            }),
            None => &Vec::new(),
            _ => return Err(MintasError::RuntimeError {
                message: "Expected parameters array".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };

        if let Some(Value::String(driver)) = db_config.get("driver") {
            if driver == "sqlite" {
                if let Some(Value::String(path)) = db_config.get("path") {
                    let conn = Connection::open(path).map_err(|e| MintasError::RuntimeError {
                        message: format!("SQLite connection error: {}", e),
                        location: SourceLocation::new(0, 0),
                    })?;
                    
                    let mut stmt = conn.prepare(sql).map_err(|e| MintasError::RuntimeError {
                        message: format!("SQLite prepare error: {}", e),
                        location: SourceLocation::new(0, 0),
                    })?;

                    let mut rows_result = Vec::new();
                    let col_names: Vec<String> = stmt.column_names().into_iter().map(|s| s.to_string()).collect();

                    let mut rows = stmt.query(rusqlite::params_from_iter(params.iter().map(|v| {
                        match v {
                            Value::String(s) => Box::new(s.clone()) as Box<dyn rusqlite::ToSql>,
                            Value::Number(n) => Box::new(*n) as Box<dyn rusqlite::ToSql>,
                            Value::Boolean(b) => Box::new(*b) as Box<dyn rusqlite::ToSql>,
                            Value::Null => Box::new(rusqlite::types::Null) as Box<dyn rusqlite::ToSql>,
                            _ => Box::new(value_to_string(v)) as Box<dyn rusqlite::ToSql>,
                        }
                    }))).map_err(|e| MintasError::RuntimeError {
                        message: format!("SQLite query error: {}", e),
                        location: SourceLocation::new(0, 0),
                    })?;

                    while let Some(row) = rows.next().map_err(|e| MintasError::RuntimeError {
                         message: format!("SQLite row error: {}", e),
                         location: SourceLocation::new(0, 0),
                    })? {
                        let mut map = HashMap::new();
                        for (i, name) in col_names.iter().enumerate() {
                            let val: rusqlite::types::Value = row.get(i).unwrap_or(rusqlite::types::Value::Null);
                            let mintas_val = match val {
                                rusqlite::types::Value::Null => Value::Null,
                                rusqlite::types::Value::Integer(i) => Value::Number(i as f64),
                                rusqlite::types::Value::Real(f) => Value::Number(f),
                                rusqlite::types::Value::Text(s) => Value::String(s),
                                rusqlite::types::Value::Blob(b) => {
                                    use base64::{engine::general_purpose::STANDARD, Engine as _};
                                    Value::String(STANDARD.encode(b))
                                }, // Basic blob handling
                            };
                            map.insert(name.clone(), mintas_val);
                        }
                        rows_result.push(Value::Table(map));
                    }
                    return Ok(Value::Array(rows_result));
                }
            } else if driver == "postgres" {
                if let Some(Value::String(conn_str)) = db_config.get("connection") {
                    let mut client = Client::connect(conn_str, NoTls).map_err(|e| MintasError::RuntimeError {
                        message: format!("Postgres connection error: {}", e),
                        location: SourceLocation::new(0, 0),
                    })?;
                    
                    // Use simple_query for multiple statements/protocol query
                    let rows = client.simple_query(sql).map_err(|e| MintasError::RuntimeError {
                        message: format!("Postgres query error: {}", e),
                        location: SourceLocation::new(0, 0),
                    })?;

                    let mut result_rows = Vec::new();
                    // simple_query returns SimpleQueryMessage::Row or CommandComplete
                    // We only support basic strings return in simple_query
                    for msg in rows {
                        if let postgres::SimpleQueryMessage::Row(r) = msg {
                            let mut map = HashMap::new();
                            for i in 0..r.len() {
                                if let Some(val) = r.get(i) {
                                     map.insert(format!("col_{}", i), Value::String(val.to_string()));
                                }
                            }
                            result_rows.push(Value::Table(map));
                        }
                    }
                    return Ok(Value::Array(result_rows));
                }
            }
        }
        
        Err(MintasError::RuntimeError {
            message: "Unsupported database driver or missing configuration".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }

    #[cfg(not(feature = "database"))]
    fn query(_args: &[Value]) -> MintasResult<Value> {
        Err(MintasError::RuntimeError {
            message: "Database feature not enabled".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    fn use_middleware(args: &[Value]) -> MintasResult<Value> {
        let middleware_name = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Ok(Value::Boolean(false)),
        };
        let server_id = match args.get(1) {
            Some(Value::Number(id)) => *id as usize,
            _ => 0,
        };
        let mut servers = SERVERS.lock().unwrap();
        if let Some(server) = servers.get_mut(server_id) {
            server.add_middleware(&middleware_name, None);
            println!("🔧 Middleware enabled: {}", middleware_name);
            return Ok(Value::Boolean(true));
        }
        Ok(Value::Boolean(false))
    }
    fn cors(args: &[Value]) -> MintasResult<Value> {
        let origins = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Array(arr)) => arr.iter()
                .filter_map(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
                .collect::<Vec<_>>()
                .join(", "),
            _ => "*".to_string(),
        };
        let methods = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            _ => "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string(),
        };
        let headers = match args.get(2) {
            Some(Value::String(s)) => s.clone(),
            _ => "Content-Type, Authorization, X-Requested-With".to_string(),
        };
        let mut cors_config = HashMap::new();
        cors_config.insert("origins".to_string(), Value::String(origins));
        cors_config.insert("methods".to_string(), Value::String(methods));
        cors_config.insert("headers".to_string(), Value::String(headers));
        cors_config.insert("credentials".to_string(), Value::Boolean(true));
        cors_config.insert("max_age".to_string(), Value::Number(86400.0));
        Ok(Value::Table(cors_config))
    }
    fn auth(args: &[Value]) -> MintasResult<Value> {
        let auth_type = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "bearer".to_string(),
        };
        let secret = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            _ => "default_secret_change_me".to_string(),
        };
        let mut auth_config = HashMap::new();
        auth_config.insert("type".to_string(), Value::String(auth_type));
        auth_config.insert("secret".to_string(), Value::String(secret));
        auth_config.insert("__type__".to_string(), Value::String("AuthConfig".to_string()));
        Ok(Value::Table(auth_config))
    }
    fn rate_limit(args: &[Value]) -> MintasResult<Value> {
        let requests = match args.get(0) {
            Some(Value::Number(n)) => *n as u32,
            _ => 100,
        };
        let window = match args.get(1) {
            Some(Value::Number(n)) => *n as u32,
            _ => 60,
        };
        let mut config = HashMap::new();
        config.insert("requests".to_string(), Value::Number(requests as f64));
        config.insert("window_seconds".to_string(), Value::Number(window as f64));
        config.insert("__type__".to_string(), Value::String("RateLimitConfig".to_string()));
        Ok(Value::Table(config))
    }
    fn compress(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Boolean(true))
    }
    fn logger(args: &[Value]) -> MintasResult<Value> {
        let format = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "combined".to_string(),
        };
        println!("📝 Logger enabled: {}", format);
        Ok(Value::Boolean(true))
    }
    fn static_files(args: &[Value]) -> MintasResult<Value> {
        let url_path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "/static".to_string(),
        };
        let dir_path = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            _ => "static/".to_string(),
        };
        let server_id = match args.get(2) {
            Some(Value::Number(id)) => *id as usize,
            _ => 0,
        };
        let mut servers = SERVERS.lock().unwrap();
        if let Some(server) = servers.get_mut(server_id) {
            server.add_static_dir(&url_path, &dir_path);
            println!("📁 Static files: {} -> {}", url_path, dir_path);
        }
        Ok(Value::Boolean(true))
    }
    fn inview(args: &[Value]) -> MintasResult<Value> {
        let template_path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(MintasError::RuntimeError {
                message: "Template path required".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let data = match args.get(1) {
            Some(Value::Table(t)) => t.clone(),
            _ => HashMap::new(),
        };
        let template_content = fs::read_to_string(&template_path)
            .or_else(|_| fs::read_to_string(format!("templates/{}", template_path)))
            .or_else(|_| fs::read_to_string(format!("views/{}", template_path)))
            .unwrap_or_else(|_| format!("<!-- Template not found: {} -->", template_path));
        let rendered = render_template(&template_content, &data);
        let mut response = HashMap::new();
        response.insert("__type__".to_string(), Value::String("DewResponse".to_string()));
        response.insert("response_type".to_string(), Value::String("html".to_string()));
        response.insert("body".to_string(), Value::String(rendered));
        response.insert("status".to_string(), Value::Number(200.0));
        Ok(Value::Table(response))
    }
    fn render(args: &[Value]) -> MintasResult<Value> {
        Self::inview(args)
    }
    fn session(args: &[Value]) -> MintasResult<Value> {
        let key = match args.get(0) {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        };
        if let Some(k) = key {
            let sessions = SESSIONS.lock().unwrap();
            if let Some(session_data) = sessions.get("current") {
                if let Some(value) = session_data.get(&k) {
                    return Ok(value.clone());
                }
            }
            Ok(Value::Empty)
        } else {
            let mut session = HashMap::new();
            session.insert("__type__".to_string(), Value::String("Session".to_string()));
            let sessions = SESSIONS.lock().unwrap();
            if let Some(session_data) = sessions.get("current") {
                for (k, v) in session_data {
                    session.insert(k.clone(), v.clone());
                }
            }
            Ok(Value::Table(session))
        }
    }
    fn session_set(args: &[Value]) -> MintasResult<Value> {
        let key = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Ok(Value::Boolean(false)),
        };
        let value = args.get(1).cloned().unwrap_or(Value::Empty);
        let mut sessions = SESSIONS.lock().unwrap();
        let session_data = sessions.entry("current".to_string()).or_insert_with(HashMap::new);
        session_data.insert(key, value);
        Ok(Value::Boolean(true))
    }
    fn session_get(args: &[Value]) -> MintasResult<Value> {
        Self::session(args)
    }
    fn session_destroy(_args: &[Value]) -> MintasResult<Value> {
        let mut sessions = SESSIONS.lock().unwrap();
        sessions.remove("current");
        Ok(Value::Boolean(true))
    }
    fn cookie(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Ok(Value::Empty),
        };
        let cookies = COOKIES.lock().unwrap();
        if let Some(value) = cookies.get(&name) {
            Ok(Value::String(value.clone()))
        } else {
            Ok(Value::Empty)
        }
    }
    fn set_cookie(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Ok(Value::Boolean(false)),
        };
        let value = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        let max_age = match args.get(2) {
            Some(Value::Number(n)) => *n as u64,
            _ => 3600,
        };
        let path = match args.get(3) {
            Some(Value::String(s)) => s.clone(),
            _ => "/".to_string(),
        };
        let http_only = match args.get(4) {
            Some(Value::Boolean(b)) => *b,
            _ => true,
        };
        let mut cookie = HashMap::new();
        cookie.insert("name".to_string(), Value::String(name.clone()));
        cookie.insert("value".to_string(), Value::String(value.clone()));
        cookie.insert("max_age".to_string(), Value::Number(max_age as f64));
        cookie.insert("path".to_string(), Value::String(path));
        cookie.insert("http_only".to_string(), Value::Boolean(http_only));
        cookie.insert("__type__".to_string(), Value::String("SetCookie".to_string()));
        let mut cookies = COOKIES.lock().unwrap();
        cookies.insert(name, value);
        Ok(Value::Table(cookie))
    }
    fn upload(args: &[Value]) -> MintasResult<Value> {
        let field_name = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "file".to_string(),
        };
        let uploads = UPLOADS.lock().unwrap();
        if let Some(file_info) = uploads.get(&field_name) {
            return Ok(file_info.clone());
        }
        let mut file_info = HashMap::new();
        file_info.insert("field".to_string(), Value::String(field_name));
        file_info.insert("filename".to_string(), Value::String(String::new()));
        file_info.insert("size".to_string(), Value::Number(0.0));
        file_info.insert("content_type".to_string(), Value::String(String::new()));
        file_info.insert("__type__".to_string(), Value::String("UploadedFile".to_string()));
        Ok(Value::Table(file_info))
    }
    fn save_upload(args: &[Value]) -> MintasResult<Value> {
        let file = match args.get(0) {
            Some(Value::Table(t)) => t.clone(),
            _ => return Ok(Value::Boolean(false)),
        };
        let dest_path = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            _ => "uploads/".to_string(),
        };
        fs::create_dir_all(&dest_path).ok();
        let filename = match file.get("filename") {
            Some(Value::String(s)) => s.clone(),
            _ => format!("upload_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
        };
        let full_path = format!("{}/{}", dest_path.trim_end_matches('/'), filename);
        Ok(Value::String(full_path))
    }
    fn validate(args: &[Value]) -> MintasResult<Value> {
        let data = match args.get(0) {
            Some(Value::Table(t)) => t.clone(),
            _ => HashMap::new(),
        };
        let rules = match args.get(1) {
            Some(Value::Table(t)) => t.clone(),
            _ => return Ok(Value::Boolean(true)),
        };
        let mut errors: HashMap<String, Value> = HashMap::new();
        let mut is_valid = true;
        for (field, rule) in &rules {
            if let Value::String(rule_str) = rule {
                let field_value = data.get(field);
                let rule_parts: Vec<&str> = rule_str.split('|').collect();
                for part in rule_parts {
                    let validation_result = validate_field(field_value, part);
                    if let Some(error_msg) = validation_result {
                        is_valid = false;
                        errors.insert(field.clone(), Value::String(error_msg));
                        break;
                    }
                }
            }
        }
        if is_valid {
            let mut result = HashMap::new();
            result.insert("valid".to_string(), Value::Boolean(true));
            result.insert("data".to_string(), Value::Table(data));
            Ok(Value::Table(result))
        } else {
            let mut result = HashMap::new();
            result.insert("valid".to_string(), Value::Boolean(false));
            result.insert("errors".to_string(), Value::Table(errors));
            Ok(Value::Table(result))
        }
    }
    fn websocket(args: &[Value]) -> MintasResult<Value> {
        let path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "/ws".to_string(),
        };
        let server_id = match args.get(1) {
            Some(Value::Number(id)) => *id as usize,
            _ => 0,
        };
        let mut servers = SERVERS.lock().unwrap();
        if let Some(server) = servers.get_mut(server_id) {
            server.websocket_paths.push(path.clone());
            println!("🔌 WebSocket endpoint: {}", path);
        }
        let mut ws = HashMap::new();
        ws.insert("path".to_string(), Value::String(path));
        ws.insert("__type__".to_string(), Value::String("WebSocket".to_string()));
        Ok(Value::Table(ws))
    }
    fn ws_send(args: &[Value]) -> MintasResult<Value> {
        let _client_id = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            _ => return Ok(Value::Boolean(false)),
        };
        let message = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Table(t)) => value_to_json(&Value::Table(t.clone())),
            _ => return Ok(Value::Boolean(false)),
        };
        println!("📤 WS Send: {}", message);
        Ok(Value::Boolean(true))
    }
    fn ws_broadcast(args: &[Value]) -> MintasResult<Value> {
        let message = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Table(t)) => value_to_json(&Value::Table(t.clone())),
            _ => return Ok(Value::Boolean(false)),
        };
        println!("📢 WS Broadcast: {}", message);
        Ok(Value::Boolean(true))
    }
    fn test_get(args: &[Value]) -> MintasResult<Value> {
        let path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "/".to_string(),
        };
        let headers = match args.get(1) {
            Some(Value::Table(t)) => t.clone(),
            _ => HashMap::new(),
        };
        let mut response = HashMap::new();
        response.insert("status".to_string(), Value::Number(200.0));
        response.insert("method".to_string(), Value::String("GET".to_string()));
        response.insert("path".to_string(), Value::String(path));
        response.insert("headers".to_string(), Value::Table(headers));
        response.insert("body".to_string(), Value::String(String::new()));
        Ok(Value::Table(response))
    }
    fn test_post(args: &[Value]) -> MintasResult<Value> {
        let path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "/".to_string(),
        };
        let body = args.get(1).cloned().unwrap_or(Value::Empty);
        let mut response = HashMap::new();
        response.insert("status".to_string(), Value::Number(200.0));
        response.insert("method".to_string(), Value::String("POST".to_string()));
        response.insert("path".to_string(), Value::String(path));
        response.insert("body".to_string(), body);
        Ok(Value::Table(response))
    }
    fn test_put(args: &[Value]) -> MintasResult<Value> {
        let path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "/".to_string(),
        };
        let body = args.get(1).cloned().unwrap_or(Value::Empty);
        let mut response = HashMap::new();
        response.insert("status".to_string(), Value::Number(200.0));
        response.insert("method".to_string(), Value::String("PUT".to_string()));
        response.insert("path".to_string(), Value::String(path));
        response.insert("body".to_string(), body);
        Ok(Value::Table(response))
    }
    fn test_delete(args: &[Value]) -> MintasResult<Value> {
        let path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "/".to_string(),
        };
        let mut response = HashMap::new();
        response.insert("status".to_string(), Value::Number(200.0));
        response.insert("method".to_string(), Value::String("DELETE".to_string()));
        response.insert("path".to_string(), Value::String(path));
        Ok(Value::Table(response))
    }
    fn config(args: &[Value]) -> MintasResult<Value> {
        let config_path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "config.yaml".to_string(),
        };
        if Path::new(&config_path).exists() {
            let content = fs::read_to_string(&config_path).unwrap_or_default();
            let mut config = HashMap::new();
            for line in content.lines() {
                if let Some(idx) = line.find(':') {
                    let key = line[..idx].trim().to_string();
                    let value = line[idx + 1..].trim().to_string();
                    config.insert(key, Value::String(value));
                }
            }
            Ok(Value::Table(config))
        } else {
            Ok(Value::Empty)
        }
    }
    fn dotenv(args: &[Value]) -> MintasResult<Value> {
        let env_path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => ".env".to_string(),
        };
        if Path::new(&env_path).exists() {
            let content = fs::read_to_string(&env_path).unwrap_or_default();
            let mut loaded = 0;
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') { continue; }
                if let Some(eq_pos) = line.find('=') {
                    let key = line[..eq_pos].trim().to_string();
                    let value = line[eq_pos + 1..].trim().trim_matches('"').trim_matches('\'').to_string();
                    std::env::set_var(&key, &value);
                    loaded += 1;
                }
            }
            println!("📄 Loaded {} environment variables from {}", loaded, env_path);
            Ok(Value::Boolean(true))
        } else {
            Ok(Value::Boolean(false))
        }
    }
    fn env(args: &[Value]) -> MintasResult<Value> {
        let key = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Ok(Value::Empty),
        };
        let default = args.get(1).cloned();
        match std::env::var(&key) {
            Ok(value) => Ok(Value::String(value)),
            Err(_) => Ok(default.unwrap_or(Value::Empty)),
        }
    }
    fn job(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(MintasError::RuntimeError {
                message: "Job name required".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let delay_ms = match args.get(1) {
            Some(Value::Number(n)) => *n as u64,
            _ => 0,
        };
        let job_id = generate_job_id();
        let mut jobs = JOBS.lock().unwrap();
        jobs.insert(job_id.clone(), JobInfo {
            id: job_id.clone(),
            name: name.clone(),
            status: "pending".to_string(),
            created_at: current_timestamp(),
            scheduled_at: current_timestamp() + delay_ms,
            data: args.get(2).cloned().unwrap_or(Value::Empty),
        });
        println!("📋 Job created: {} ({})", name, job_id);
        let mut result = HashMap::new();
        result.insert("id".to_string(), Value::String(job_id));
        result.insert("name".to_string(), Value::String(name));
        result.insert("status".to_string(), Value::String("pending".to_string()));
        result.insert("__type__".to_string(), Value::String("Job".to_string()));
        Ok(Value::Table(result))
    }
    fn queue(args: &[Value]) -> MintasResult<Value> {
        let queue_name = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "default".to_string(),
        };
        let mut queues = QUEUES.lock().unwrap();
        if !queues.contains_key(&queue_name) {
            queues.insert(queue_name.clone(), Vec::new());
            println!("📬 Queue created: {}", queue_name);
        }
        if let Some(data) = args.get(1) {
            if let Some(queue) = queues.get_mut(&queue_name) {
                queue.push(data.clone());
                println!("📬 Added item to queue: {}", queue_name);
            }
        }
        let mut result = HashMap::new();
        result.insert("name".to_string(), Value::String(queue_name.clone()));
        result.insert("size".to_string(), Value::Number(queues.get(&queue_name).map(|q| q.len()).unwrap_or(0) as f64));
        result.insert("__type__".to_string(), Value::String("Queue".to_string()));
        Ok(Value::Table(result))
    }
    fn task(args: &[Value]) -> MintasResult<Value> {
        let name = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(MintasError::RuntimeError {
                message: "Task name required".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let task_id = generate_job_id();
        println!("⚡ Task scheduled: {} ({})", name, task_id);
        let mut result = HashMap::new();
        result.insert("id".to_string(), Value::String(task_id));
        result.insert("name".to_string(), Value::String(name));
        result.insert("status".to_string(), Value::String("scheduled".to_string()));
        result.insert("__type__".to_string(), Value::String("Task".to_string()));
        Ok(Value::Table(result))
    }
    fn schedule(args: &[Value]) -> MintasResult<Value> {
        let cron_expr = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(MintasError::RuntimeError {
                message: "Cron expression required".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let task_name = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            _ => "scheduled_task".to_string(),
        };
        println!("⏰ Scheduled task: {} with cron '{}'", task_name, cron_expr);
        let mut result = HashMap::new();
        result.insert("cron".to_string(), Value::String(cron_expr));
        result.insert("task".to_string(), Value::String(task_name));
        result.insert("__type__".to_string(), Value::String("ScheduledTask".to_string()));
        Ok(Value::Table(result))
    }
    fn chunk_upload(args: &[Value]) -> MintasResult<Value> {
        let upload_id = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => generate_job_id(),
        };
        let chunk_index = match args.get(1) {
            Some(Value::Number(n)) => *n as usize,
            _ => 0,
        };
        let chunk_data = match args.get(2) {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        let total_chunks = match args.get(3) {
            Some(Value::Number(n)) => *n as usize,
            _ => 1,
        };
        let mut chunks = CHUNK_UPLOADS.lock().unwrap();
        let upload = chunks.entry(upload_id.clone()).or_insert_with(|| ChunkUpload {
            id: upload_id.clone(),
            chunks: HashMap::new(),
            total_chunks,
            filename: String::new(),
            content_type: String::new(),
        });
        upload.chunks.insert(chunk_index, chunk_data);
        let received = upload.chunks.len();
        println!("📦 Chunk {}/{} received for upload {}", chunk_index + 1, total_chunks, upload_id);
        let mut result = HashMap::new();
        result.insert("upload_id".to_string(), Value::String(upload_id));
        result.insert("chunk_index".to_string(), Value::Number(chunk_index as f64));
        result.insert("received".to_string(), Value::Number(received as f64));
        result.insert("total".to_string(), Value::Number(total_chunks as f64));
        result.insert("complete".to_string(), Value::Boolean(received >= total_chunks));
        result.insert("__type__".to_string(), Value::String("ChunkUpload".to_string()));
        Ok(Value::Table(result))
    }
    fn chunk_complete(args: &[Value]) -> MintasResult<Value> {
        let upload_id = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Ok(Value::Boolean(false)),
        };
        let dest_path = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            _ => "uploads/".to_string(),
        };
        let mut chunks = CHUNK_UPLOADS.lock().unwrap();
        if let Some(upload) = chunks.remove(&upload_id) {
            let mut combined = String::new();
            for i in 0..upload.total_chunks {
                if let Some(chunk) = upload.chunks.get(&i) {
                    combined.push_str(chunk);
                }
            }
            fs::create_dir_all(&dest_path).ok();
            let filename = if upload.filename.is_empty() {
                format!("upload_{}", upload_id)
            } else {
                upload.filename.clone()
            };
            let full_path = format!("{}/{}", dest_path.trim_end_matches('/'), filename);
            if fs::write(&full_path, &combined).is_ok() {
                println!("✅ Chunked upload complete: {}", full_path);
                return Ok(Value::String(full_path));
            }
        }
        Ok(Value::Boolean(false))
    }
    fn protect(args: &[Value]) -> MintasResult<Value> {
        let protection_type = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "all".to_string(),
        };
        let mut config = HashMap::new();
        match protection_type.as_str() {
            "csrf" => {
                config.insert("csrf".to_string(), Value::Boolean(true));
                println!("🛡️  CSRF protection enabled");
            }
            "xss" => {
                config.insert("xss".to_string(), Value::Boolean(true));
                println!("🛡️  XSS protection enabled");
            }
            "sql" => {
                config.insert("sql_injection".to_string(), Value::Boolean(true));
                println!("🛡️  SQL injection protection enabled");
            }
            "ddos" => {
                config.insert("ddos".to_string(), Value::Boolean(true));
                println!("🛡️  DDoS protection enabled");
            }
            "all" | _ => {
                config.insert("csrf".to_string(), Value::Boolean(true));
                config.insert("xss".to_string(), Value::Boolean(true));
                config.insert("sql_injection".to_string(), Value::Boolean(true));
                config.insert("ddos".to_string(), Value::Boolean(true));
                println!("🛡️  All security protections enabled");
            }
        }
        config.insert("__type__".to_string(), Value::String("SecurityConfig".to_string()));
        Ok(Value::Table(config))
    }
    fn csrf_token(_args: &[Value]) -> MintasResult<Value> {
        let token = generate_csrf_token();
        let mut sessions = SESSIONS.lock().unwrap();
        let session = sessions.entry("current".to_string()).or_insert_with(HashMap::new);
        session.insert("_csrf_token".to_string(), Value::String(token.clone()));
        Ok(Value::String(token))
    }
    fn csrf_verify(args: &[Value]) -> MintasResult<Value> {
        let provided_token = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Ok(Value::Boolean(false)),
        };
        let sessions = SESSIONS.lock().unwrap();
        if let Some(session) = sessions.get("current") {
            if let Some(Value::String(stored_token)) = session.get("_csrf_token") {
                return Ok(Value::Boolean(&provided_token == stored_token));
            }
        }
        Ok(Value::Boolean(false))
    }
    fn sanitize(args: &[Value]) -> MintasResult<Value> {
        let input = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Ok(Value::String(String::new())),
        };
        let sanitize_type = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            _ => "html".to_string(),
        };
        let sanitized = match sanitize_type.as_str() {
            "html" => sanitize_html(&input),
            "sql" => sanitize_sql(&input),
            "js" => sanitize_js(&input),
            "url" => sanitize_url(&input),
            _ => sanitize_html(&input),
        };
        Ok(Value::String(sanitized))
    }
    fn ws_on_connect(args: &[Value]) -> MintasResult<Value> {
        let path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "/ws".to_string(),
        };
        println!("🔌 WebSocket on_connect handler registered for {}", path);
        let mut result = HashMap::new();
        result.insert("event".to_string(), Value::String("connect".to_string()));
        result.insert("path".to_string(), Value::String(path));
        result.insert("__type__".to_string(), Value::String("WSEventHandler".to_string()));
        Ok(Value::Table(result))
    }
    fn ws_on_disconnect(args: &[Value]) -> MintasResult<Value> {
        let path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "/ws".to_string(),
        };
        println!("🔌 WebSocket on_disconnect handler registered for {}", path);
        let mut result = HashMap::new();
        result.insert("event".to_string(), Value::String("disconnect".to_string()));
        result.insert("path".to_string(), Value::String(path));
        result.insert("__type__".to_string(), Value::String("WSEventHandler".to_string()));
        Ok(Value::Table(result))
    }
    fn ws_on_message(args: &[Value]) -> MintasResult<Value> {
        let path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "/ws".to_string(),
        };
        println!("🔌 WebSocket on_message handler registered for {}", path);
        let mut result = HashMap::new();
        result.insert("event".to_string(), Value::String("message".to_string()));
        result.insert("path".to_string(), Value::String(path));
        result.insert("__type__".to_string(), Value::String("WSEventHandler".to_string()));
        Ok(Value::Table(result))
    }
    fn ws_on_error(args: &[Value]) -> MintasResult<Value> {
        let path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "/ws".to_string(),
        };
        println!("🔌 WebSocket on_error handler registered for {}", path);
        let mut result = HashMap::new();
        result.insert("event".to_string(), Value::String("error".to_string()));
        result.insert("path".to_string(), Value::String(path));
        result.insert("__type__".to_string(), Value::String("WSEventHandler".to_string()));
        Ok(Value::Table(result))
    }
    fn ws_join(args: &[Value]) -> MintasResult<Value> {
        let room = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(MintasError::RuntimeError {
                message: "Room name required".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let client_id = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            _ => "anonymous".to_string(),
        };
        let mut rooms = WS_ROOMS.lock().unwrap();
        let room_clients = rooms.entry(room.clone()).or_insert_with(Vec::new);
        if !room_clients.contains(&client_id) {
            room_clients.push(client_id.clone());
        }
        println!("🚪 Client {} joined room {}", client_id, room);
        Ok(Value::Boolean(true))
    }
    fn ws_leave(args: &[Value]) -> MintasResult<Value> {
        let room = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Ok(Value::Boolean(false)),
        };
        let client_id = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            _ => return Ok(Value::Boolean(false)),
        };
        let mut rooms = WS_ROOMS.lock().unwrap();
        if let Some(room_clients) = rooms.get_mut(&room) {
            room_clients.retain(|c| c != &client_id);
            println!("🚪 Client {} left room {}", client_id, room);
            return Ok(Value::Boolean(true));
        }
        Ok(Value::Boolean(false))
    }
    fn ws_room_broadcast(args: &[Value]) -> MintasResult<Value> {
        let room = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Ok(Value::Boolean(false)),
        };
        let message = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Table(t)) => value_to_json(&Value::Table(t.clone())),
            _ => return Ok(Value::Boolean(false)),
        };
        let rooms = WS_ROOMS.lock().unwrap();
        if let Some(clients) = rooms.get(&room) {
            println!("📢 Broadcasting to room {} ({} clients): {}", room, clients.len(), message);
            return Ok(Value::Number(clients.len() as f64));
        }
        Ok(Value::Number(0.0))
    }
    fn ws_rooms(_args: &[Value]) -> MintasResult<Value> {
        let rooms = WS_ROOMS.lock().unwrap();
        let room_list: Vec<Value> = rooms.keys()
            .map(|k| Value::String(k.clone()))
            .collect();
        Ok(Value::Array(room_list))
    }
    fn ws_clients(args: &[Value]) -> MintasResult<Value> {
        let room = match args.get(0) {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        };
        let rooms = WS_ROOMS.lock().unwrap();
        if let Some(room_name) = room {
            if let Some(clients) = rooms.get(&room_name) {
                let client_list: Vec<Value> = clients.iter()
                    .map(|c| Value::String(c.clone()))
                    .collect();
                return Ok(Value::Array(client_list));
            }
            return Ok(Value::Array(Vec::new()));
        }
        let mut all_clients: Vec<String> = Vec::new();
        for clients in rooms.values() {
            for client in clients {
                if !all_clients.contains(client) {
                    all_clients.push(client.clone());
                }
            }
        }
        let client_list: Vec<Value> = all_clients.iter()
            .map(|c| Value::String(c.clone()))
            .collect();
        Ok(Value::Array(client_list))
    }
    fn response_text(args: &[Value]) -> MintasResult<Value> {
        let body = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            Some(v) => format!("{:?}", v),
            _ => String::new(),
        };
        let status = match args.get(1) {
            Some(Value::Number(n)) => *n as u16,
            _ => 200,
        };
        let mut response = HashMap::new();
        response.insert("__type__".to_string(), Value::String("DewResponse".to_string()));
        response.insert("response_type".to_string(), Value::String("text".to_string()));
        response.insert("body".to_string(), Value::String(body));
        response.insert("status".to_string(), Value::Number(status as f64));
        Ok(Value::Table(response))
    }
    fn response_html(args: &[Value]) -> MintasResult<Value> {
        let body = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        let status = match args.get(1) {
            Some(Value::Number(n)) => *n as u16,
            _ => 200,
        };
        let mut response = HashMap::new();
        response.insert("__type__".to_string(), Value::String("DewResponse".to_string()));
        response.insert("response_type".to_string(), Value::String("html".to_string()));
        response.insert("body".to_string(), Value::String(body));
        response.insert("status".to_string(), Value::Number(status as f64));
        Ok(Value::Table(response))
    }
    fn response_json(args: &[Value]) -> MintasResult<Value> {
        let body = match args.get(0) {
            Some(Value::Table(t)) => value_to_json(&Value::Table(t.clone())),
            Some(Value::Array(a)) => value_to_json(&Value::Array(a.clone())),
            Some(Value::String(s)) => s.clone(),
            _ => "{}".to_string(),
        };
        let status = match args.get(1) {
            Some(Value::Number(n)) => *n as u16,
            _ => 200,
        };
        let mut response = HashMap::new();
        response.insert("__type__".to_string(), Value::String("DewResponse".to_string()));
        response.insert("response_type".to_string(), Value::String("json".to_string()));
        response.insert("body".to_string(), Value::String(body));
        response.insert("status".to_string(), Value::Number(status as f64));
        Ok(Value::Table(response))
    }
    fn response_redirect(args: &[Value]) -> MintasResult<Value> {
        let location = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "/".to_string(),
        };
        let permanent = match args.get(1) {
            Some(Value::Boolean(b)) => *b,
            _ => false,
        };
        let mut response = HashMap::new();
        response.insert("__type__".to_string(), Value::String("DewResponse".to_string()));
        response.insert("response_type".to_string(), Value::String("redirect".to_string()));
        response.insert("location".to_string(), Value::String(location));
        response.insert("status".to_string(), Value::Number(if permanent { 301.0 } else { 302.0 }));
        Ok(Value::Table(response))
    }
    fn response_file(args: &[Value]) -> MintasResult<Value> {
        let file_path = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(MintasError::RuntimeError {
                message: "File path required".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        if Path::new(&file_path).exists() {
            let content = fs::read(&file_path).unwrap_or_default();
            let content_type = get_mime_type(&file_path);
            let filename = Path::new(&file_path).file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "download".to_string());
            let mut response = HashMap::new();
            response.insert("__type__".to_string(), Value::String("DewResponse".to_string()));
            response.insert("response_type".to_string(), Value::String("file".to_string()));
            response.insert("body".to_string(), Value::String(String::from_utf8_lossy(&content).to_string()));
            response.insert("content_type".to_string(), Value::String(content_type));
            response.insert("filename".to_string(), Value::String(filename));
            response.insert("status".to_string(), Value::Number(200.0));
            Ok(Value::Table(response))
        } else {
            Err(MintasError::RuntimeError {
                message: format!("File not found: {}", file_path),
                location: SourceLocation::new(0, 0),
            })
        }
    }

    // ==================== MAGICAL FEATURES IMPLEMENTATION ====================

    #[cfg(feature = "magic")]
    fn uuid(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::String(uuid::Uuid::new_v4().to_string()))
    }
    #[cfg(not(feature = "magic"))]
    fn uuid(_args: &[Value]) -> MintasResult<Value> { Err(MintasError::RuntimeError { message: "Magic feature not enabled".to_string(), location: SourceLocation::new(0,0) }) }

    #[cfg(feature = "magic")]
    fn hash_password(args: &[Value]) -> MintasResult<Value> {
        let password = match args.get(0) { Some(Value::String(s)) => s, _ => return Ok(Value::Null) };
        match bcrypt::hash(password, bcrypt::DEFAULT_COST) {
            Ok(h) => Ok(Value::String(h)),
            Err(e) => Err(MintasError::RuntimeError { message: format!("Hashing failed: {}", e), location: SourceLocation::new(0,0) }),
        }
    }
    #[cfg(not(feature = "magic"))]
    fn hash_password(_args: &[Value]) -> MintasResult<Value> { Err(MintasError::RuntimeError { message: "Magic feature not enabled".to_string(), location: SourceLocation::new(0,0) }) }

    #[cfg(feature = "magic")]
    fn verify_password(args: &[Value]) -> MintasResult<Value> {
        let password = match args.get(0) { Some(Value::String(s)) => s, _ => return Ok(Value::Boolean(false)) };
        let hash_str = match args.get(1) { Some(Value::String(s)) => s, _ => return Ok(Value::Boolean(false)) };
        match bcrypt::verify(password, hash_str) {
            Ok(valid) => Ok(Value::Boolean(valid)),
            Err(_) => Ok(Value::Boolean(false)),
        }
    }
    #[cfg(not(feature = "magic"))]
    fn verify_password(_args: &[Value]) -> MintasResult<Value> { Err(MintasError::RuntimeError { message: "Magic feature not enabled".to_string(), location: SourceLocation::new(0,0) }) }

    #[cfg(feature = "magic")]
    fn sha256(args: &[Value]) -> MintasResult<Value> {
        let input = match args.get(0) { Some(Value::String(s)) => s, _ => "" };
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        Ok(Value::String(hex::encode(result)))
    }
    #[cfg(not(feature = "magic"))]
    fn sha256(_args: &[Value]) -> MintasResult<Value> { Err(MintasError::RuntimeError { message: "Magic feature not enabled".to_string(), location: SourceLocation::new(0,0) }) }

    #[cfg(feature = "magic")]
    fn csv_parse(args: &[Value]) -> MintasResult<Value> {
        let content = match args.get(0) { Some(Value::String(s)) => s, _ => "" };
        let mut rdr = csv::Reader::from_reader(content.as_bytes());
        let mut result = Vec::new();
        // Headers
        let headers: Vec<String> = if let Ok(h) = rdr.headers() {
            h.iter().map(|s| s.to_string()).collect()
        } else {
             return Ok(Value::Array(vec![]));
        };
        for rec in rdr.records() {
            if let Ok(record) = rec {
                let mut map = HashMap::new();
                for (i, field) in record.iter().enumerate() {
                    if i < headers.len() {
                        map.insert(headers[i].clone(), Value::String(field.to_string()));
                    }
                }
                result.push(Value::Table(map));
            }
        }
        Ok(Value::Array(result))
    }
    #[cfg(not(feature = "magic"))]
    fn csv_parse(_args: &[Value]) -> MintasResult<Value> { Err(MintasError::RuntimeError { message: "Magic feature not enabled".to_string(), location: SourceLocation::new(0,0) }) }

    #[cfg(feature = "magic")]
    fn csv_stringify(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::String("Not implemented".to_string()))
    }
    #[cfg(not(feature = "magic"))]
    fn csv_stringify(_args: &[Value]) -> MintasResult<Value> { Err(MintasError::RuntimeError { message: "Magic feature not enabled".to_string(), location: SourceLocation::new(0,0) }) }

    // ==================== REDIS ====================

    #[cfg(feature = "database")]
    fn redis_get(args: &[Value]) -> MintasResult<Value> {
        let url = match args.get(0) { Some(Value::String(s)) => s, _ => "redis://127.0.0.1/" };
        let key = match args.get(1) { Some(Value::String(s)) => s, _ => return Ok(Value::Null) };
        
        // This blocks, but it's okay for now
        let client = redis::Client::open(url.as_ref()).map_err(|e| MintasError::RuntimeError { message: format!("Redis Error: {}",e), location: SourceLocation::new(0,0)})?;
        let mut con = client.get_connection().map_err(|e| MintasError::RuntimeError { message: format!("Redis Error: {}",e), location: SourceLocation::new(0,0)})?;
        let result: Option<String> = con.get(key).ok();
        
        match result {
            Some(s) => Ok(Value::String(s)),
            None => Ok(Value::Null),
        }
    }
    #[cfg(not(feature = "database"))]
    fn redis_get(_args: &[Value]) -> MintasResult<Value> { Err(MintasError::RuntimeError { message: "Database feature not enabled".to_string(), location: SourceLocation::new(0,0) }) }

    // WebRTC Implementation
    fn webrtc_peer(args: &[Value]) -> MintasResult<Value> {
        // Creates a new WebRTC peer connection
        // Usage: let peer = dew.webrtc_peer({id: "peer1", config: {...}})
        let config = match args.get(0) {
            Some(Value::Table(map)) => map.clone(),
            _ => HashMap::new(),
        };
        
        let peer_id = config.get("id").and_then(|v| match v { Value::String(s) => Some(s.clone()), _ => None })
            .unwrap_or_else(|| format!("peer_{}", std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()));
        
        let mut peer_config = HashMap::new();
        peer_config.insert("id".to_string(), Value::String(peer_id));
        peer_config.insert("state".to_string(), Value::String("new".to_string()));
        peer_config.insert("connection_state".to_string(), Value::String("new".to_string()));
        peer_config.insert("ice_connection_state".to_string(), Value::String("new".to_string()));
        peer_config.insert("data_channels".to_string(), Value::Table(HashMap::new()));
        peer_config.insert("created_at".to_string(), Value::String(chrono::Local::now().to_rfc3339()));
        
        Ok(Value::Table(peer_config))
    }

    fn webrtc_offer(args: &[Value]) -> MintasResult<Value> {
        // Creates an offer for a peer connection
        // Usage: let offer = dew.webrtc_offer(peer)
        match args.get(0) {
            Some(Value::Table(peer)) => {
                let peer_id = peer.get("id").and_then(|v| match v { Value::String(s) => Some(s.clone()), _ => None })
                    .unwrap_or_else(|| "unknown".to_string());
                
                let mut offer = HashMap::new();
                offer.insert("type".to_string(), Value::String("offer".to_string()));
                offer.insert("peer_id".to_string(), Value::String(peer_id));
                offer.insert("sdp".to_string(), Value::String(format!("v=0\r\no=mintas {} 0 IN IP4 127.0.0.1\r\ns=Mintas WebRTC\r\nt=0 0\r\n", 
                    std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())));
                offer.insert("timestamp".to_string(), Value::String(chrono::Local::now().to_rfc3339()));
                
                Ok(Value::Table(offer))
            },
            _ => Err(MintasError::RuntimeError { 
                message: "webrtc_offer requires a peer object".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        }
    }

    fn webrtc_answer(args: &[Value]) -> MintasResult<Value> {
        // Creates an answer for a received offer
        // Usage: let answer = dew.webrtc_answer(offer)
        match args.get(0) {
            Some(Value::Table(offer)) => {
                let peer_id = offer.get("peer_id").and_then(|v| match v { Value::String(s) => Some(s.clone()), _ => None })
                    .unwrap_or_else(|| "unknown".to_string());
                
                let mut answer = HashMap::new();
                answer.insert("type".to_string(), Value::String("answer".to_string()));
                answer.insert("peer_id".to_string(), Value::String(peer_id));
                answer.insert("sdp".to_string(), Value::String(format!("v=0\r\no=mintas {} 0 IN IP4 127.0.0.1\r\ns=Mintas WebRTC\r\nt=0 0\r\n", 
                    std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())));
                answer.insert("timestamp".to_string(), Value::String(chrono::Local::now().to_rfc3339()));
                
                Ok(Value::Table(answer))
            },
            _ => Err(MintasError::RuntimeError { 
                message: "webrtc_answer requires an offer object".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        }
    }

    fn webrtc_datachannel(args: &[Value]) -> MintasResult<Value> {
        // Creates a data channel for peer communication
        // Usage: let dc = dew.webrtc_datachannel(peer, {label: "chat", ordered: true})
        match (args.get(0), args.get(1)) {
            (Some(Value::Table(_peer)), Some(Value::Table(config))) => {
                let label = config.get("label").and_then(|v| match v { Value::String(s) => Some(s.clone()), _ => None })
                    .unwrap_or_else(|| "data".to_string());
                
                let mut channel = HashMap::new();
                channel.insert("label".to_string(), Value::String(label));
                channel.insert("state".to_string(), Value::String("connecting".to_string()));
                channel.insert("buffered_amount".to_string(), Value::Number(0.0));
                channel.insert("ordered".to_string(), config.get("ordered").cloned().unwrap_or(Value::Boolean(true)));
                channel.insert("max_retransmits".to_string(), Value::Number(3.0));
                channel.insert("max_packet_lifetime".to_string(), Value::Number(3000.0));
                
                Ok(Value::Table(channel))
            },
            _ => Err(MintasError::RuntimeError { 
                message: "webrtc_datachannel requires a peer object and config".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        }
    }

    fn webrtc_send(args: &[Value]) -> MintasResult<Value> {
        // Sends data through a data channel
        // Usage: dew.webrtc_send(datachannel, "hello")
        match (args.get(0), args.get(1)) {
            (Some(Value::Table(_channel)), Some(data)) => {
                let message = match data {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Boolean(b) => b.to_string(),
                    Value::Table(t) => table_to_json_string(&t),
                    _ => "null".to_string(),
                };
                
                let mut result = HashMap::new();
                result.insert("sent".to_string(), Value::Boolean(true));
                result.insert("bytes".to_string(), Value::Number(message.len() as f64));
                result.insert("timestamp".to_string(), Value::String(chrono::Local::now().to_rfc3339()));
                
                Ok(Value::Table(result))
            },
            _ => Err(MintasError::RuntimeError { 
                message: "webrtc_send requires a datachannel and data".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        }
    }

    fn webrtc_on_message(_args: &[Value]) -> MintasResult<Value> {
        // Registers a message handler for incoming data
        // Usage: dew.webrtc_on_message(datachannel, fn(data) { ... })
        Ok(Value::String("Message handler registered".to_string()))
    }

    fn webrtc_close(args: &[Value]) -> MintasResult<Value> {
        // Closes a peer connection or data channel
        // Usage: dew.webrtc_close(peer) or dew.webrtc_close(datachannel)
        match args.get(0) {
            Some(Value::Table(obj)) => {
                let obj_type = if obj.contains_key("data_channels") { "peer" } else { "datachannel" };
                
                let mut result = HashMap::new();
                result.insert("closed".to_string(), Value::Boolean(true));
                result.insert("type".to_string(), Value::String(obj_type.to_string()));
                result.insert("timestamp".to_string(), Value::String(chrono::Local::now().to_rfc3339()));
                
                Ok(Value::Table(result))
            },
            _ => Err(MintasError::RuntimeError { 
                message: "webrtc_close requires a peer or datachannel object".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        }
    }

    fn webrtc_stats(_args: &[Value]) -> MintasResult<Value> {
        // Gets statistics for a peer connection
        // Usage: let stats = dew.webrtc_stats(peer)
        let mut stats = HashMap::new();
        stats.insert("bytes_sent".to_string(), Value::Number(0.0));
        stats.insert("bytes_received".to_string(), Value::Number(0.0));
        stats.insert("packets_lost".to_string(), Value::Number(0.0));
        stats.insert("jitter".to_string(), Value::Number(0.0));
        stats.insert("round_trip_time".to_string(), Value::Number(0.0));
        stats.insert("current_round_trip_time".to_string(), Value::Number(0.0));
        stats.insert("available_outgoing_bitrate".to_string(), Value::Number(0.0));
        stats.insert("available_incoming_bitrate".to_string(), Value::Number(0.0));
        stats.insert("timestamp".to_string(), Value::String(chrono::Local::now().to_rfc3339()));
        
        Ok(Value::Table(stats))
    }

    // JavaScript Event Handling Implementation
    fn js_onclick(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for click events
        // Usage: let code = dew.js_onclick("button", fn(e) { ... })
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "button".to_string(),
        };
        
        let js_code = format!(
            "document.addEventListener('click', function(e) {{\n  \
            if (e.target.matches('{}')) {{\n    \
            mintas_handler_onclick(e);\n  \
            }}\n}});",
            selector
        );
        
        Ok(Value::String(js_code))
    }

    fn js_onchange(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for change events
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "input, select, textarea".to_string(),
        };
        
        let js_code = format!(
            "document.addEventListener('change', function(e) {{\n  \
            if (e.target.matches('{}')) {{\n    \
            mintas_handler_onchange(e);\n  \
            }}\n}});",
            selector
        );
        
        Ok(Value::String(js_code))
    }

    fn js_oninput(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for input events
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "input, textarea".to_string(),
        };
        
        let js_code = format!(
            "document.addEventListener('input', function(e) {{\n  \
            if (e.target.matches('{}')) {{\n    \
            mintas_handler_oninput(e);\n  \
            }}\n}});",
            selector
        );
        
        Ok(Value::String(js_code))
    }

    fn js_onsubmit(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for form submission
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "form".to_string(),
        };
        
        let js_code = format!(
            "document.addEventListener('submit', function(e) {{\n  \
            if (e.target.matches('{}')) {{\n    \
            e.preventDefault();\n    \
            mintas_handler_onsubmit(e);\n  \
            }}\n}});",
            selector
        );
        
        Ok(Value::String(js_code))
    }

    fn js_onfocus(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for focus events
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "input, textarea".to_string(),
        };
        
        let js_code = format!(
            "document.addEventListener('focus', function(e) {{\n  \
            if (e.target.matches('{}')) {{\n    \
            mintas_handler_onfocus(e);\n  \
            }}\n}}, true);",
            selector
        );
        
        Ok(Value::String(js_code))
    }

    fn js_onblur(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for blur events
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "input, textarea".to_string(),
        };
        
        let js_code = format!(
            "document.addEventListener('blur', function(e) {{\n  \
            if (e.target.matches('{}')) {{\n    \
            mintas_handler_onblur(e);\n  \
            }}\n}}, true);",
            selector
        );
        
        Ok(Value::String(js_code))
    }

    fn js_onkeypress(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for keypress events
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "input, textarea".to_string(),
        };
        
        let js_code = format!(
            "document.addEventListener('keypress', function(e) {{\n  \
            if (e.target.matches('{}')) {{\n    \
            mintas_handler_onkeypress(e);\n  \
            }}\n}});",
            selector
        );
        
        Ok(Value::String(js_code))
    }

    fn js_onkeydown(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for keydown events
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "input, textarea".to_string(),
        };
        
        let js_code = format!(
            "document.addEventListener('keydown', function(e) {{\n  \
            if (e.target.matches('{}')) {{\n    \
            mintas_handler_onkeydown(e);\n  \
            }}\n}});",
            selector
        );
        
        Ok(Value::String(js_code))
    }

    fn js_onkeyup(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for keyup events
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "input, textarea".to_string(),
        };
        
        let js_code = format!(
            "document.addEventListener('keyup', function(e) {{\n  \
            if (e.target.matches('{}')) {{\n    \
            mintas_handler_onkeyup(e);\n  \
            }}\n}});",
            selector
        );
        
        Ok(Value::String(js_code))
    }

    fn js_onmouseover(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for mouseover events
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "*".to_string(),
        };
        
        let js_code = format!(
            "document.addEventListener('mouseover', function(e) {{\n  \
            if (e.target.matches('{}')) {{\n    \
            mintas_handler_onmouseover(e);\n  \
            }}\n}});",
            selector
        );
        
        Ok(Value::String(js_code))
    }

    fn js_onmouseout(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for mouseout events
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "*".to_string(),
        };
        
        let js_code = format!(
            "document.addEventListener('mouseout', function(e) {{\n  \
            if (e.target.matches('{}')) {{\n    \
            mintas_handler_onmouseout(e);\n  \
            }}\n}});",
            selector
        );
        
        Ok(Value::String(js_code))
    }

    fn js_onload(_args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for page load events
        let js_code = "window.addEventListener('load', function(e) {\n  mintas_handler_onload(e);\n});".to_string();
        Ok(Value::String(js_code))
    }

    fn js_query(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for DOM element queries
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(MintasError::RuntimeError { 
                message: "js_query requires a selector string".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        };
        
        let js_code = format!("document.querySelector('{}')", selector);
        Ok(Value::String(js_code))
    }

    fn js_query_all(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for querying multiple DOM elements
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(MintasError::RuntimeError { 
                message: "js_query_all requires a selector string".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        };
        
        let js_code = format!("document.querySelectorAll('{}')", selector);
        Ok(Value::String(js_code))
    }

    fn js_set_attr(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for setting element attributes
        let (selector, attr, value) = match (args.get(0), args.get(1), args.get(2)) {
            (Some(Value::String(s)), Some(Value::String(a)), Some(Value::String(v))) => 
                (s.clone(), a.clone(), v.clone()),
            _ => return Err(MintasError::RuntimeError { 
                message: "js_set_attr requires selector, attribute, and value strings".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        };
        
        let js_code = format!("document.querySelector('{}').setAttribute('{}', '{}')", selector, attr, value);
        Ok(Value::String(js_code))
    }

    fn js_get_attr(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for getting element attributes
        let (selector, attr) = match (args.get(0), args.get(1)) {
            (Some(Value::String(s)), Some(Value::String(a))) => (s.clone(), a.clone()),
            _ => return Err(MintasError::RuntimeError { 
                message: "js_get_attr requires selector and attribute strings".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        };
        
        let js_code = format!("document.querySelector('{}').getAttribute('{}')", selector, attr);
        Ok(Value::String(js_code))
    }

    fn js_set_class(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for setting element class
        let (selector, class_name) = match (args.get(0), args.get(1)) {
            (Some(Value::String(s)), Some(Value::String(c))) => (s.clone(), c.clone()),
            _ => return Err(MintasError::RuntimeError { 
                message: "js_set_class requires selector and class name strings".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        };
        
        let js_code = format!("document.querySelector('{}').className = '{}'", selector, class_name);
        Ok(Value::String(js_code))
    }

    fn js_add_class(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for adding element class
        let (selector, class_name) = match (args.get(0), args.get(1)) {
            (Some(Value::String(s)), Some(Value::String(c))) => (s.clone(), c.clone()),
            _ => return Err(MintasError::RuntimeError { 
                message: "js_add_class requires selector and class name strings".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        };
        
        let js_code = format!("document.querySelector('{}').classList.add('{}')", selector, class_name);
        Ok(Value::String(js_code))
    }

    fn js_remove_class(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for removing element class
        let (selector, class_name) = match (args.get(0), args.get(1)) {
            (Some(Value::String(s)), Some(Value::String(c))) => (s.clone(), c.clone()),
            _ => return Err(MintasError::RuntimeError { 
                message: "js_remove_class requires selector and class name strings".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        };
        
        let js_code = format!("document.querySelector('{}').classList.remove('{}')", selector, class_name);
        Ok(Value::String(js_code))
    }

    fn js_set_html(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for setting element inner HTML
        let (selector, html) = match (args.get(0), args.get(1)) {
            (Some(Value::String(s)), Some(Value::String(h))) => (s.clone(), h.clone()),
            _ => return Err(MintasError::RuntimeError { 
                message: "js_set_html requires selector and HTML strings".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        };
        
        let safe_html = html.replace("'", "\\'");
        let js_code = format!("document.querySelector('{}').innerHTML = '{}'", selector, safe_html);
        Ok(Value::String(js_code))
    }

    fn js_set_text(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for setting element text content
        let (selector, text) = match (args.get(0), args.get(1)) {
            (Some(Value::String(s)), Some(Value::String(t))) => (s.clone(), t.clone()),
            _ => return Err(MintasError::RuntimeError { 
                message: "js_set_text requires selector and text strings".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        };
        
        let safe_text = text.replace("'", "\\'");
        let js_code = format!("document.querySelector('{}').textContent = '{}'", selector, safe_text);
        Ok(Value::String(js_code))
    }

    fn js_show(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for showing an element
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(MintasError::RuntimeError { 
                message: "js_show requires a selector string".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        };
        
        let js_code = format!("document.querySelector('{}').style.display = 'block'", selector);
        Ok(Value::String(js_code))
    }

    fn js_hide(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for hiding an element
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(MintasError::RuntimeError { 
                message: "js_hide requires a selector string".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        };
        
        let js_code = format!("document.querySelector('{}').style.display = 'none'", selector);
        Ok(Value::String(js_code))
    }

    fn js_toggle(args: &[Value]) -> MintasResult<Value> {
        // Generates JS code for toggling element visibility
        let selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => return Err(MintasError::RuntimeError { 
                message: "js_toggle requires a selector string".to_string(), 
                location: SourceLocation::new(0,0) 
            })
        };
        
        let js_code = format!(
            "document.querySelector('{}').style.display = document.querySelector('{}').style.display === 'none' ? 'block' : 'none'",
            selector, selector
        );
        Ok(Value::String(js_code))
    }

    fn js_validate_form(args: &[Value]) -> MintasResult<Value> {
        // Generates comprehensive JS form validation code
        let form_selector = match args.get(0) {
            Some(Value::String(s)) => s.clone(),
            _ => "form".to_string(),
        };
        
        let validation_rules = match args.get(1) {
            Some(Value::Table(t)) => t.clone(),
            _ => HashMap::new(),
        };
        
        let mut js_code = format!(
            "function validateForm(form) {{\n  \
            let errors = [];\n  \
            const fields = form.querySelectorAll('[data-validate]');\n  \
            fields.forEach(field => {{\n    \
            const rules = field.getAttribute('data-validate').split('|');\n    \
            rules.forEach(rule => {{\n"
        );
        
        js_code.push_str(&format!(
            "      if (rule === 'required' && !field.value) {{\n        \
            errors.push(field.name + ' is required');\n      \
            }}\n      \
            if (rule.startsWith('min:')) {{\n        \
            const min = parseInt(rule.substring(4));\n        \
            if (field.value.length < min) {{\n          \
            errors.push(field.name + ' must be at least ' + min + ' characters');\n        \
            }}\n      \
            }}\n      \
            if (rule.startsWith('max:')) {{\n        \
            const max = parseInt(rule.substring(4));\n        \
            if (field.value.length > max) {{\n          \
            errors.push(field.name + ' must be at most ' + max + ' characters');\n        \
            }}\n      \
            }}\n      \
            if (rule === 'email' && field.value && !/^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$/.test(field.value)) {{\n        \
            errors.push(field.name + ' must be a valid email');\n      \
            }}\n    \
            }});\n  \
            }});\n  \
            return errors;\n\
            }}\n\
            document.querySelector('{}').addEventListener('submit', function(e) {{\n  \
            const errors = validateForm(this);\n  \
            if (errors.length > 0) {{\n    \
            e.preventDefault();\n    \
            mintas_handler_validation_error(errors);\n  \
            }}\n\
            }});",
            form_selector
        ));
        
        Ok(Value::String(js_code))
    }

    #[cfg(feature = "database")]
    fn redis_set(args: &[Value]) -> MintasResult<Value> {
        let url = match args.get(0) { Some(Value::String(s)) => s, _ => "redis://127.0.0.1/" };
        let key = match args.get(1) { Some(Value::String(s)) => s, _ => return Ok(Value::Boolean(false)) };
        let val = match args.get(2) { Some(Value::String(s)) => s, _ => return Ok(Value::Boolean(false)) };
        
        let client = redis::Client::open(url.as_ref()).map_err(|e| MintasError::RuntimeError { message: format!("Redis Error: {}",e), location: SourceLocation::new(0,0)})?;
        let mut con = client.get_connection().map_err(|e| MintasError::RuntimeError { message: format!("Redis Error: {}",e), location: SourceLocation::new(0,0)})?;
        let _: () = con.set(key, val).map_err(|e| MintasError::RuntimeError { message: format!("Redis Error: {}",e), location: SourceLocation::new(0,0)})?;
        Ok(Value::Boolean(true))
    }
    #[cfg(not(feature = "database"))]
    fn redis_set(_args: &[Value]) -> MintasResult<Value> { Err(MintasError::RuntimeError { message: "Database feature not enabled".to_string(), location: SourceLocation::new(0,0) }) }
}
lazy_static::lazy_static! {
    static ref SERVERS: Mutex<ServerRegistry> = Mutex::new(ServerRegistry::new());
    static ref SESSIONS: Mutex<HashMap<String, HashMap<String, Value>>> = Mutex::new(HashMap::new());
    static ref COOKIES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref UPLOADS: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());
    static ref JOBS: Mutex<HashMap<String, JobInfo>> = Mutex::new(HashMap::new());
    static ref QUEUES: Mutex<HashMap<String, Vec<Value>>> = Mutex::new(HashMap::new());
    static ref CHUNK_UPLOADS: Mutex<HashMap<String, ChunkUpload>> = Mutex::new(HashMap::new());
    static ref WS_ROOMS: Mutex<HashMap<String, Vec<String>>> = Mutex::new(HashMap::new());
}
struct ServerRegistry {
    servers: Vec<DewServer>,
}
impl ServerRegistry {
    fn new() -> Self {
        Self { servers: Vec::new() }
    }
    fn register(&mut self, server: DewServer) -> usize {
        let id = self.servers.len();
        self.servers.push(server);
        id
    }
    fn get(&self, id: usize) -> Option<&DewServer> {
        self.servers.get(id)
    }
    fn get_mut(&mut self, id: usize) -> Option<&mut DewServer> {
        self.servers.get_mut(id)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method {
    GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD,
}
impl Method {
    pub fn from_str(s: &str) -> Option<Method> {
        match s.to_uppercase().as_str() {
            "GET" => Some(Method::GET),
            "POST" => Some(Method::POST),
            "PUT" => Some(Method::PUT),
            "DELETE" => Some(Method::DELETE),
            "PATCH" => Some(Method::PATCH),
            "OPTIONS" => Some(Method::OPTIONS),
            "HEAD" => Some(Method::HEAD),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::PATCH => "PATCH",
            Method::OPTIONS => "OPTIONS",
            Method::HEAD => "HEAD",
        }
    }
}
/// Getback - Request object for Mintas handlers
#[derive(Debug, Clone)]
pub struct Getback {
    pub method: String,
    pub path: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub body: String,
    pub ip: String,
    pub cookies: HashMap<String, String>,
}
impl Getback {
    pub fn new() -> Self {
        Getback {
            method: String::new(),
            path: String::new(),
            url: String::new(),
            headers: HashMap::new(),
            query: HashMap::new(),
            params: HashMap::new(),
            body: String::new(),
            ip: String::new(),
            cookies: HashMap::new(),
        }
    }
    pub fn to_value(&self) -> Value {
        let mut map = HashMap::new();
        map.insert("method".to_string(), Value::String(self.method.clone()));
        map.insert("path".to_string(), Value::String(self.path.clone()));
        map.insert("url".to_string(), Value::String(self.url.clone()));
        map.insert("body".to_string(), Value::String(self.body.clone()));
        map.insert("ip".to_string(), Value::String(self.ip.clone()));
        // Headers as table
        let headers_map: HashMap<String, Value> = self.headers
            .iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect();
        map.insert("headers".to_string(), Value::Table(headers_map));
        // Query params as table
        let query_map: HashMap<String, Value> = self.query
            .iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect();
        map.insert("query".to_string(), Value::Table(query_map));
        // Path params as table
        let params_map: HashMap<String, Value> = self.params
            .iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect();
        map.insert("params".to_string(), Value::Table(params_map.clone()));
        map.insert("param".to_string(), Value::Table(params_map));
        // Cookies as table
        let cookies_map: HashMap<String, Value> = self.cookies
            .iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect();
        map.insert("cookies".to_string(), Value::Table(cookies_map));
        // JSON body parser
        if self.headers.get("content-type").map(|ct| ct.contains("application/json")).unwrap_or(false) {
            if let Ok(json_val) = parse_json_to_value(&self.body) {
                map.insert("json".to_string(), json_val);
            }
        }
        // Form data parser
        if self.headers.get("content-type").map(|ct| ct.contains("application/x-www-form-urlencoded")).unwrap_or(false) {
            let form_data = parse_form_data(&self.body);
            map.insert("form".to_string(), Value::Table(form_data));
        }
        map.insert("__type__".to_string(), Value::String("Getback".to_string()));
        Value::Table(map)
    }
    pub fn json(&self) -> Value {
        parse_json_to_value(&self.body).unwrap_or(Value::Empty)
    }
    pub fn form(&self) -> HashMap<String, Value> {
        parse_form_data(&self.body)
    }
    pub fn text(&self) -> String {
        self.body.clone()
    }
}
/// Response builder
#[derive(Debug, Clone)]
pub struct DewResponse {
    pub status: u16,
    pub content_type: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub cookies: Vec<String>,
}
impl DewResponse {
    pub fn text(body: &str, status: Option<u16>) -> Self {
        DewResponse {
            status: status.unwrap_or(200),
            content_type: "text/plain; charset=utf-8".to_string(),
            headers: HashMap::new(),
            body: body.to_string(),
            cookies: Vec::new(),
        }
    }
    pub fn html(body: &str, status: Option<u16>) -> Self {
        DewResponse {
            status: status.unwrap_or(200),
            content_type: "text/html; charset=utf-8".to_string(),
            headers: HashMap::new(),
            body: body.to_string(),
            cookies: Vec::new(),
        }
    }
    pub fn json(body: &str, status: Option<u16>) -> Self {
        DewResponse {
            status: status.unwrap_or(200),
            content_type: "application/json; charset=utf-8".to_string(),
            headers: HashMap::new(),
            body: body.to_string(),
            cookies: Vec::new(),
        }
    }
    pub fn redirect(location: &str, permanent: bool) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Location".to_string(), location.to_string());
        DewResponse {
            status: if permanent { 301 } else { 302 },
            content_type: "text/plain".to_string(),
            headers,
            body: String::new(),
            cookies: Vec::new(),
        }
    }
    pub fn file(content: &[u8], content_type: &str, filename: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Disposition".to_string(), format!("attachment; filename=\"{}\"", filename));
        DewResponse {
            status: 200,
            content_type: content_type.to_string(),
            headers,
            body: String::from_utf8_lossy(content).to_string(),
            cookies: Vec::new(),
        }
    }
}
/// Middleware handler
#[derive(Clone)]
pub struct Middleware {
    pub name: String,
    pub handler_body: Option<Vec<crate::parser::Expr>>,
}
/// Error handler
#[derive(Clone)]
pub struct ErrorHandler {
    pub status_code: u16,
    pub handler_body: Vec<crate::parser::Expr>,
}
/// Route handler
#[derive(Clone)]
pub struct RouteHandler {
    pub handler_body: Vec<crate::parser::Expr>,
}
/// Route definition
#[derive(Clone)]
pub struct Route {
    pub method: Method,
    pub path: String,
    pub handler: RouteHandler,
    pub validation: Option<HashMap<String, String>>,
}
/// Route group
#[derive(Clone)]
pub struct RouteGroup {
    pub prefix: String,
    pub routes: Vec<Route>,
    pub middleware: Vec<String>,
}
// ==================== PHASE 6 CONFIG STRUCTS ====================
/// Database configuration
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub driver: String,
    pub connection_string: String,
    pub pool_size: u32,
    pub timeout: u32,
}
/// Session configuration
#[derive(Clone, Debug)]
pub struct SessionConfig {
    pub secret: String,
    pub max_age: u64,
    pub cookie_name: String,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: String,
}
/// Rate limit configuration
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub requests_per_window: u32,
    pub window_seconds: u32,
    pub by_ip: bool,
    pub by_user: bool,
}
/// Security configuration
#[derive(Clone, Debug)]
pub struct SecurityConfig {
    pub sql_injection_protection: bool,
    pub xss_protection: bool,
    pub csrf_protection: bool,
    pub ddos_protection: bool,
    pub max_request_size: usize,
    pub allowed_hosts: Vec<String>,
}
impl Default for SecurityConfig {
    fn default() -> Self {
        SecurityConfig {
            sql_injection_protection: true,
            xss_protection: true,
            csrf_protection: true,
            ddos_protection: true,
            max_request_size: 10 * 1024 * 1024, // 10MB
            allowed_hosts: vec!["*".to_string()],
        }
    }
}
/// Phase 6: Job information
#[derive(Clone, Debug)]
pub struct JobInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: u64,
    pub scheduled_at: u64,
    pub data: Value,
}
/// Phase 6: Chunked upload state
#[derive(Clone, Debug)]
pub struct ChunkUpload {
    pub id: String,
    pub chunks: HashMap<usize, String>,
    pub total_chunks: usize,
    pub filename: String,
    pub content_type: String,
}
/// Dew Server
#[derive(Clone)]
pub struct DewServer {
    pub routes: Vec<Route>,
    pub static_dirs: Vec<(String, String)>,
    pub middleware: Vec<Middleware>,
    pub before_handlers: Vec<Vec<crate::parser::Expr>>,
    pub after_handlers: Vec<Vec<crate::parser::Expr>>,
    pub error_handlers: HashMap<u16, ErrorHandler>,
    pub groups: Vec<RouteGroup>,
    pub websocket_paths: Vec<String>,
    pub cors_config: Option<HashMap<String, String>>,
    // Phase 6 additions
    pub config: HashMap<String, Value>,
    pub database: Option<DatabaseConfig>,
    pub session_config: Option<SessionConfig>,
    pub rate_limit: Option<RateLimitConfig>,
    pub security: SecurityConfig,
}
impl DewServer {
    pub fn new() -> Self {
        DewServer {
            routes: Vec::new(),
            static_dirs: Vec::new(),
            middleware: Vec::new(),
            before_handlers: Vec::new(),
            after_handlers: Vec::new(),
            error_handlers: HashMap::new(),
            groups: Vec::new(),
            websocket_paths: Vec::new(),
            cors_config: None,
            // Phase 6 additions
            config: HashMap::new(),
            database: None,
            session_config: None,
            rate_limit: None,
            security: SecurityConfig::default(),
        }
    }
    pub fn add_route(&mut self, method: Method, path: &str, handler: RouteHandler) {
        self.routes.push(Route {
            method,
            path: path.to_string(),
            handler,
            validation: None,
        });
    }
    pub fn add_route_with_validation(&mut self, method: Method, path: &str, handler: RouteHandler, validation: HashMap<String, String>) {
        self.routes.push(Route {
            method,
            path: path.to_string(),
            handler,
            validation: Some(validation),
        });
    }
    pub fn add_middleware(&mut self, name: &str, handler: Option<Vec<crate::parser::Expr>>) {
        self.middleware.push(Middleware {
            name: name.to_string(),
            handler_body: handler,
        });
    }
    pub fn add_before_handler(&mut self, handler: Vec<crate::parser::Expr>) {
        self.before_handlers.push(handler);
    }
    pub fn add_after_handler(&mut self, handler: Vec<crate::parser::Expr>) {
        self.after_handlers.push(handler);
    }
    pub fn add_error_handler(&mut self, status_code: u16, handler: Vec<crate::parser::Expr>) {
        self.error_handlers.insert(status_code, ErrorHandler {
            status_code,
            handler_body: handler,
        });
    }
    pub fn add_static_dir(&mut self, url_path: &str, dir_path: &str) {
        self.static_dirs.push((url_path.to_string(), dir_path.to_string()));
    }
    pub fn find_route(&self, method: &str, path: &str) -> Option<(&Route, HashMap<String, String>)> {
        let method_enum = Method::from_str(method)?;
        // Check direct routes first
        for route in &self.routes {
            if route.method == method_enum {
                if let Some(params) = match_path(&route.path, path) {
                    return Some((route, params));
                }
            }
        }
        // Check grouped routes
        for group in &self.groups {
            for route in &group.routes {
                if route.method == method_enum {
                    let full_path = format!("{}{}", group.prefix, route.path);
                    if let Some(params) = match_path(&full_path, path) {
                        return Some((route, params));
                    }
                }
            }
        }
        None
    }
    pub fn find_static_file(&self, path: &str) -> Option<String> {
        for (url_prefix, dir_path) in &self.static_dirs {
            if path.starts_with(url_prefix) {
                let file_path = path.strip_prefix(url_prefix).unwrap_or("");
                let file_path = file_path.trim_start_matches('/');
                let full_path = if dir_path.ends_with('/') {
                    format!("{}{}", dir_path, file_path)
                } else {
                    format!("{}/{}", dir_path, file_path)
                };
                if Path::new(&full_path).exists() && Path::new(&full_path).is_file() {
                    return Some(full_path);
                }
            }
        }
        None
    }
}
fn match_path(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if pattern_parts.len() != path_parts.len() {
        return None;
    }
    let mut params = HashMap::new();
    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
        if pattern_part.starts_with('>') {
            // Path parameter: />id captures the value
            let param_name = &pattern_part[1..];
            params.insert(param_name.to_string(), path_part.to_string());
        } else if *pattern_part != *path_part {
            return None;
        }
    }
    Some(params)
}
/// Add route to server (called from evaluator)
pub fn add_server_route(server_id: usize, method: &str, path: &str, handler_body: Vec<crate::parser::Expr>) -> MintasResult<()> {
    let method_enum = Method::from_str(method).ok_or_else(|| MintasError::RuntimeError {
        message: format!("Invalid HTTP method: {}", method),
        location: SourceLocation::new(0, 0),
    })?;
    // Get current group prefix and prepend to path
    let group_prefix = get_current_group_prefix(server_id);
    let full_path = if group_prefix.is_empty() {
        path.to_string()
    } else {
        format!("{}{}", group_prefix, path)
    };
    let mut servers = SERVERS.lock().unwrap();
    if let Some(server) = servers.get_mut(server_id) {
        server.add_route(method_enum, &full_path, RouteHandler { handler_body });
        Ok(())
    } else {
        Err(MintasError::RuntimeError {
            message: "Server not found".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
}
/// Add validated route to server (called from evaluator for routes with ==> validate({...}))
pub fn add_server_validated_route(
    server_id: usize, 
    method: &str, 
    path: &str, 
    validation_rules: Value,
    handler_body: Vec<crate::parser::Expr>
) -> MintasResult<()> {
    let method_enum = Method::from_str(method).ok_or_else(|| MintasError::RuntimeError {
        message: format!("Invalid HTTP method: {}", method),
        location: SourceLocation::new(0, 0),
    })?;
    // Get current group prefix and prepend to path
    let group_prefix = get_current_group_prefix(server_id);
    let full_path = if group_prefix.is_empty() {
        path.to_string()
    } else {
        format!("{}{}", group_prefix, path)
    };
    // Convert validation rules Value to HashMap<String, String>
    let validation = match validation_rules {
        Value::Table(map) => {
            let mut rules = HashMap::new();
            for (key, value) in map {
                if let Value::String(rule) = value {
                    rules.insert(key, rule);
                }
            }
            rules
        }
        _ => HashMap::new(),
    };
    let mut servers = SERVERS.lock().unwrap();
    if let Some(server) = servers.get_mut(server_id) {
        server.add_route_with_validation(method_enum, &full_path, RouteHandler { handler_body }, validation);
        Ok(())
    } else {
        Err(MintasError::RuntimeError {
            message: "Server not found".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
}
/// Add error handler to server
pub fn add_server_error_handler(server_id: usize, status_code: u16, handler_body: Vec<crate::parser::Expr>) -> MintasResult<()> {
    let mut servers = SERVERS.lock().unwrap();
    if let Some(server) = servers.get_mut(server_id) {
        server.add_error_handler(status_code, handler_body);
        Ok(())
    } else {
        Err(MintasError::RuntimeError {
            message: "Server not found".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
}
/// Add before handler to server
pub fn add_server_before_handler(server_id: usize, handler_body: Vec<crate::parser::Expr>) -> MintasResult<()> {
    let mut servers = SERVERS.lock().unwrap();
    if let Some(server) = servers.get_mut(server_id) {
        server.add_before_handler(handler_body);
        Ok(())
    } else {
        Err(MintasError::RuntimeError {
            message: "Server not found".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
}
/// Add after handler to server
pub fn add_server_after_handler(server_id: usize, handler_body: Vec<crate::parser::Expr>) -> MintasResult<()> {
    let mut servers = SERVERS.lock().unwrap();
    if let Some(server) = servers.get_mut(server_id) {
        server.add_after_handler(handler_body);
        Ok(())
    } else {
        Err(MintasError::RuntimeError {
            message: "Server not found".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
}
/// Add static directory to server
pub fn add_server_static(server_id: usize, url_path: &str, dir_path: &str) -> MintasResult<()> {
    let mut servers = SERVERS.lock().unwrap();
    if let Some(server) = servers.get_mut(server_id) {
        server.add_static_dir(url_path, dir_path);
        println!("📁 Static files: {} -> {}", url_path, dir_path);
        Ok(())
    } else {
        Err(MintasError::RuntimeError {
            message: "Server not found".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
}
/// Add middleware to server
pub fn add_server_middleware(server_id: usize, middleware_name: &str) -> MintasResult<()> {
    let mut servers = SERVERS.lock().unwrap();
    if let Some(server) = servers.get_mut(server_id) {
        server.add_middleware(middleware_name, None);
        println!("🔧 Middleware enabled: {}", middleware_name);
        Ok(())
    } else {
        Err(MintasError::RuntimeError {
            message: "Server not found".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
}
// Global state for route groups
lazy_static::lazy_static! {
    static ref CURRENT_GROUP_PREFIX: Mutex<HashMap<usize, String>> = Mutex::new(HashMap::new());
}
/// Start a route group context
pub fn start_route_group(server_id: usize, prefix: &str) -> MintasResult<()> {
    let mut group_prefixes = CURRENT_GROUP_PREFIX.lock().unwrap();
    // Append to existing prefix if nested
    let current = group_prefixes.get(&server_id).cloned().unwrap_or_default();
    let new_prefix = format!("{}{}", current, prefix);
    group_prefixes.insert(server_id, new_prefix.clone());
    println!("📂 Route group: {}", new_prefix);
    Ok(())
}
/// End a route group context
pub fn end_route_group(server_id: usize) -> MintasResult<()> {
    let mut group_prefixes = CURRENT_GROUP_PREFIX.lock().unwrap();
    // Remove the last segment of the prefix
    if let Some(prefix) = group_prefixes.get_mut(&server_id) {
        // Find the last '/' and remove everything after it
        if let Some(last_slash) = prefix.rfind('/') {
            if last_slash > 0 {
                prefix.truncate(last_slash);
            } else {
                prefix.clear();
            }
        } else {
            prefix.clear();
        }
    }
    Ok(())
}
/// Get current group prefix for a server
pub fn get_current_group_prefix(server_id: usize) -> String {
    CURRENT_GROUP_PREFIX.lock().unwrap()
        .get(&server_id)
        .cloned()
        .unwrap_or_default()
}
/// Get server for serving
pub fn get_server(server_id: usize) -> Option<DewServer> {
    SERVERS.lock().unwrap().get(server_id).cloned()
}
// ==================== PHASE 6: CONFIG, DATABASE, SESSIONS, RATE LIMITING ====================
/// Load configuration from file (YAML, JSON, or .env)
pub fn load_server_config(server_id: usize, config_path: &str) -> MintasResult<()> {
    let mut servers = SERVERS.lock().unwrap();
    if let Some(server) = servers.get_mut(server_id) {
        // Read config file
        let content = fs::read_to_string(config_path)
            .or_else(|_| fs::read_to_string(format!("config/{}", config_path)))
            .map_err(|e| MintasError::RuntimeError {
                message: format!("Failed to read config file '{}': {}", config_path, e),
                location: SourceLocation::new(0, 0),
            })?;
        // Parse based on file extension
        if config_path.ends_with(".env") || config_path == ".env" {
            // Parse .env format
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') { continue; }
                if let Some(eq_pos) = line.find('=') {
                    let key = line[..eq_pos].trim().to_string();
                    let value = line[eq_pos + 1..].trim().trim_matches('"').trim_matches('\'').to_string();
                    server.config.insert(key.clone(), Value::String(value.clone()));
                    // Also set as environment variable
                    std::env::set_var(&key, &value);
                }
            }
            println!("📄 Loaded .env config: {}", config_path);
        } else if config_path.ends_with(".yaml") || config_path.ends_with(".yml") {
            // Simple YAML parsing (key: value format)
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') { continue; }
                if let Some(colon_pos) = line.find(':') {
                    let key = line[..colon_pos].trim().to_string();
                    let value = line[colon_pos + 1..].trim().trim_matches('"').trim_matches('\'').to_string();
                    let parsed_value = if value == "true" {
                        Value::Boolean(true)
                    } else if value == "false" {
                        Value::Boolean(false)
                    } else if let Ok(n) = value.parse::<f64>() {
                        Value::Number(n)
                    } else {
                        Value::String(value)
                    };
                    server.config.insert(key, parsed_value);
                }
            }
            println!("📄 Loaded YAML config: {}", config_path);
        } else if config_path.ends_with(".json") {
            if let Ok(parsed) = parse_json_to_value(&content) {
                if let Value::Table(map) = parsed {
                    for (k, v) in map {
                        server.config.insert(k, v);
                    }
                }
            }
            println!("📄 Loaded JSON config: {}", config_path);
        }
        Ok(())
    } else {
        Err(MintasError::RuntimeError {
            message: "Server not found".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
}
pub fn setup_server_database(server_id: usize, connection_string: &str) -> MintasResult<()> {
    let mut servers = SERVERS.lock().unwrap();
    if let Some(server) = servers.get_mut(server_id) {
        let (driver, db_path) = if connection_string.starts_with("sqlite:") {
            ("sqlite", connection_string.trim_start_matches("sqlite:///").to_string())
        } else if connection_string.starts_with("postgres:") || connection_string.starts_with("postgresql:") {
            ("postgres", connection_string.to_string())
        } else if connection_string.starts_with("redis:") {
            ("redis", connection_string.to_string())
        } else {
            ("sqlite", connection_string.to_string())
        };
        server.database = Some(DatabaseConfig {
            driver: driver.to_string(),
            connection_string: db_path.clone(),
            pool_size: 10,
            timeout: 30,
        });
        if driver == "sqlite" {
            if let Ok(_) = create_default_tables(&db_path) {
                println!("🗄️  Database initialized: {} ({})", db_path, driver);
            }
        } else {
            println!("🗄️  Database configured: {} ({})", connection_string, driver);
        }
        Ok(())
    } else {
        Err(MintasError::RuntimeError {
            message: "Server not found".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
}
fn create_default_tables(db_path: &str) -> Result<(), String> {
    if !Path::new(db_path).exists() {
        if let Some(parent) = Path::new(db_path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(db_path, "");
    }
    Ok(())
}
pub fn setup_server_session(server_id: usize, config: Value) -> MintasResult<()> {
    let mut servers = SERVERS.lock().unwrap();
    if let Some(server) = servers.get_mut(server_id) {
        let mut session_config = SessionConfig {
            secret: "change_me_in_production".to_string(),
            max_age: 3600,
            cookie_name: "dew_session".to_string(),
            http_only: true,
            secure: false,
            same_site: "Lax".to_string(),
        };
        if let Value::Table(map) = config {
            if let Some(Value::String(s)) = map.get("secret") {
                session_config.secret = s.clone();
            }
            if let Some(Value::Number(n)) = map.get("max_age") {
                session_config.max_age = *n as u64;
            }
            if let Some(Value::String(s)) = map.get("cookie_name") {
                session_config.cookie_name = s.clone();
            }
            if let Some(Value::Boolean(b)) = map.get("http_only") {
                session_config.http_only = *b;
            }
            if let Some(Value::Boolean(b)) = map.get("secure") {
                session_config.secure = *b;
            }
            if let Some(Value::String(s)) = map.get("same_site") {
                session_config.same_site = s.clone();
            }
        }
        server.session_config = Some(session_config);
        println!("🔐 Sessions enabled");
        Ok(())
    } else {
        Err(MintasError::RuntimeError {
            message: "Server not found".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
}
pub fn setup_server_rate_limit(server_id: usize, requests: u32, window_seconds: u32) -> MintasResult<()> {
    let mut servers = SERVERS.lock().unwrap();
    if let Some(server) = servers.get_mut(server_id) {
        server.rate_limit = Some(RateLimitConfig {
            requests_per_window: requests,
            window_seconds,
            by_ip: true,
            by_user: false,
        });
        println!("🚦 Rate limiting: {} requests per {} seconds", requests, window_seconds);
        Ok(())
    } else {
        Err(MintasError::RuntimeError {
            message: "Server not found".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
}
fn render_template(template: &str, data: &HashMap<String, Value>) -> String {
    let mut rendered = template.to_string();
    rendered = process_template_control_flow(&rendered, data);
    rendered = process_dew_code_blocks(&rendered, data);
    rendered = process_dew_styled_blocks(&rendered, data);
    while let Some(start) = rendered.find("?(") {
        if let Some(end) = rendered[start..].find(")?") {
            let end = start + end + 2;
            let code = &rendered[start + 2..end - 2];
            let result = evaluate_template_expr(code, data);
            rendered = format!("{}{}{}", &rendered[..start], result, &rendered[end..]);
        } else {
            break;
        }
    }
    for (key, value) in data {
        let placeholder = format!("${}", key);
        let replacement = value_to_string(value);
        rendered = rendered.replace(&placeholder, &replacement);
    }
    rendered = inject_dew_frontend_script(&rendered, data);
    rendered
}
fn process_dew_styled_blocks(template: &str, data: &HashMap<String, Value>) -> String {
    let mut result = template.to_string();
    loop {
        if let Some(start) = find_innermost_dew_tag(&result) {
            if let Some(end_offset) = result[start..].find("</dew>") {
                let block_end = start + end_offset + 6;
                let block = &result[start..block_end];
                let html = parse_dew_block_to_html(block, data);
                result = format!("{}{}{}", &result[..start], html, &result[block_end..]);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    while let Some(start) = result.find("#(") {
        if let Some(end) = result[start..].find(")#") {
            let end = start + end + 2;
            result = format!("{}{}", &result[..start], &result[end..]);
        } else {
            break;
        }
    }
    result
}
fn find_innermost_dew_tag(template: &str) -> Option<usize> {
    let mut search_from = 0;
    while let Some(pos) = template[search_from..].find("<dew") {
        let abs_pos = search_from + pos;
        let after_tag = abs_pos + 4;
        if let Some(end_pos) = template[after_tag..].find("</dew>") {
            if let Some(next_dew) = template[after_tag..].find("<dew") {
                if next_dew < end_pos {
                    search_from = after_tag + next_dew;
                    continue;
                }
            }
            return Some(abs_pos);
        }
        search_from = after_tag;
    }
    None
}
fn parse_dew_block_to_html(block: &str, data: &HashMap<String, Value>) -> String {
    let tag_end = block.find('>').unwrap_or(block.len());
    let tag_part = &block[4..tag_end]; 
    let content_start = tag_end + 1;
    let content_end = block.rfind("</dew>").unwrap_or(block.len());
    let content = if content_start < content_end { &block[content_start..content_end] } else { "" };
    let id = extract_dew_attr(tag_part, "id").unwrap_or_default();
    let class = extract_dew_attr(tag_part, "class").unwrap_or_default();
    let (styles, events, inner_content) = parse_dew_dsl_content(content, data);
    let id_attr = if !id.is_empty() { format!(" id=\"{}\"", id) } else { String::new() };
    let class_attr = if !class.is_empty() { 
        format!(" class=\"dew-component {}\"", class) 
    } else { 
        " class=\"dew-component\"".to_string() 
    };
    let style_attr = if !styles.is_empty() { format!(" style=\"{}\"", styles) } else { String::new() };
    let event_attrs = generate_dew_event_attrs(&events);
    format!("<div{}{}{}{}>{}</div>", id_attr, class_attr, style_attr, event_attrs, inner_content)
}
fn extract_dew_attr(tag: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr_name);
    if let Some(start) = tag.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = tag[value_start..].find('"') {
            return Some(tag[value_start..value_start + end].to_string());
        }
    }
    None
}
fn parse_dew_dsl_content(content: &str, data: &HashMap<String, Value>) -> (String, Vec<(String, String)>, String) {
    let mut styles = Vec::new();
    let mut events = Vec::new();
    let mut inner_html = String::new();
    let mut in_event_block = false;
    let mut current_event = String::new();
    let mut event_body = String::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("#(") { continue; }
        if in_event_block && line == "end" {
            events.push((current_event.clone(), event_body.trim().to_string()));
            in_event_block = false;
            current_event.clear();
            event_body.clear();
            continue;
        }
        if in_event_block {
            event_body.push_str(line);
            event_body.push('\n');
            continue;
        }
        if line.starts_with("on-") && line.ends_with(':') {
            current_event = line[3..line.len()-1].to_string();
            in_event_block = true;
            continue;
        }
        if line.starts_with("text:") {
            let text_value = line[5..].trim().trim_matches('"').trim_matches('\'');
            let processed = process_dew_text_vars(text_value, data);
            inner_html.push_str(&processed);
            continue;
        }
        if line.contains(':') && !line.starts_with("on-") && !line.starts_with("text:") {
            if let Some(colon_pos) = line.find(':') {
                let prop = line[..colon_pos].trim();
                let value = line[colon_pos + 1..].trim().trim_matches('"').trim_matches('\'');
                if let Some(css) = convert_dew_prop_to_css(prop, value) {
                    styles.push(css);
                }
            }
            continue;
        }
        if line.contains("?(") && line.contains(")?") {
            let processed = process_dew_inline_code(line, data);
            inner_html.push_str(&processed);
            continue;
        }
        if !line.is_empty() {
            inner_html.push_str(line);
            inner_html.push(' ');
        }
    }
    (styles.join("; "), events, inner_html.trim().to_string())
}
fn process_dew_text_vars(text: &str, data: &HashMap<String, Value>) -> String {
    let mut result = text.to_string();
    for (key, value) in data {
        let placeholder = format!("${}", key);
        result = result.replace(&placeholder, &value_to_string(value));
    }
    result
}
fn process_dew_inline_code(line: &str, data: &HashMap<String, Value>) -> String {
    let mut result = line.to_string();
    while let Some(start) = result.find("?(") {
        if let Some(end) = result[start..].find(")?") {
            let end = start + end + 2;
            let code = result[start + 2..end - 2].trim();
            let evaluated = evaluate_template_expr(code, data);
            result = format!("{}{}{}", &result[..start], evaluated, &result[end..]);
        } else { break; }
    }
    result
}
fn convert_dew_prop_to_css(prop: &str, value: &str) -> Option<String> {
    let css = match prop {
        "display" => format!("display: {}", value),
        "flex" => format!("display: flex; flex: {}", value),
        "flex-direction" | "direction" => format!("flex-direction: {}", value),
        "flex-wrap" | "wrap" => format!("flex-wrap: {}", value),
        "flex-grow" | "grow" => format!("flex-grow: {}", value),
        "flex-shrink" | "shrink" => format!("flex-shrink: {}", value),
        "grid" => format!("display: grid; {}", value),
        "grid-columns" | "columns" => format!("grid-template-columns: {}", value),
        "grid-rows" | "rows" => format!("grid-template-rows: {}", value),
        "grid-area" | "area" => format!("grid-area: {}", value),
        "grid-gap" => format!("grid-gap: {}", value),
        "gap" => format!("gap: {}", value),
        "row-gap" => format!("row-gap: {}", value),
        "col-gap" | "column-gap" => format!("column-gap: {}", value),
        "padding" | "p" => format!("padding: {}", value),
        "padding-x" | "px" => format!("padding-left: {}; padding-right: {}", value, value),
        "padding-y" | "py" => format!("padding-top: {}; padding-bottom: {}", value, value),
        "padding-top" | "pt" => format!("padding-top: {}", value),
        "padding-bottom" | "pb" => format!("padding-bottom: {}", value),
        "padding-left" | "pl" => format!("padding-left: {}", value),
        "padding-right" | "pr" => format!("padding-right: {}", value),
        "margin" | "m" => format!("margin: {}", value),
        "margin-x" | "mx" => format!("margin-left: {}; margin-right: {}", value, value),
        "margin-y" | "my" => format!("margin-top: {}; margin-bottom: {}", value, value),
        "margin-top" | "mt" => format!("margin-top: {}", value),
        "margin-bottom" | "mb" => format!("margin-bottom: {}", value),
        "margin-left" | "ml" => format!("margin-left: {}", value),
        "margin-right" | "mr" => format!("margin-right: {}", value),
        "item-align" | "align" | "align-items" => format!("align-items: {}", value),
        "item-justify" | "justify" | "justify-content" => format!("justify-content: {}", value),
        "self-align" | "align-self" => format!("align-self: {}", value),
        "self-justify" | "justify-self" => format!("justify-self: {}", value),
        "place-items" => format!("place-items: {}", value),
        "place-content" => format!("place-content: {}", value),
        "item-index" | "z" | "z-index" => format!("z-index: {}", value),
        "position" | "pos" => format!("position: {}", value),
        "top" => format!("top: {}", value),
        "bottom" => format!("bottom: {}", value),
        "left" => format!("left: {}", value),
        "right" => format!("right: {}", value),
        "offset-x" => format!("left: {}", value),
        "offset-y" => format!("top: {}", value),
        "inset" => format!("inset: {}", value),
        "overflow" => format!("overflow: {}", value),
        "overflow-x" => format!("overflow-x: {}", value),
        "overflow-y" => format!("overflow-y: {}", value),
        "scroll" => format!("overflow: {}; -webkit-overflow-scrolling: touch", if value == "true" { "auto" } else { value }),
        "scroll-snap" => format!("scroll-snap-type: {}", value),
        "float" => format!("float: {}", value),
        "clear" => format!("clear: {}", value),
        "visibility" => format!("visibility: {}", value),
        "clip" => format!("clip-path: {}", value),
        "width" | "w" => format!("width: {}", value),
        "height" | "h" => format!("height: {}", value),
        "size" => format!("width: {}; height: {}", value, value),
        "min-width" | "min-w" => format!("min-width: {}", value),
        "max-width" | "max-w" => format!("max-width: {}", value),
        "min-height" | "min-h" => format!("min-height: {}", value),
        "max-height" | "max-h" => format!("max-height: {}", value),
        "aspect" | "aspect-ratio" => format!("aspect-ratio: {}", value),
        "fit" | "object-fit" => format!("object-fit: {}", value),
        "object-position" => format!("object-position: {}", value),
        "text-color" | "color" => format!("color: {}", value),
        "text-size" | "font-size" => format!("font-size: {}", value),
        "text-weight" | "font-weight" | "weight" | "bold" => {
            let w = match value {
                "thin" => "100", "light" => "300", "normal" => "400",
                "medium" => "500", "semibold" => "600", "bold" => "700",
                "extrabold" => "800", "black" => "900",
                _ => value,
            };
            format!("font-weight: {}", w)
        },
        "text-align" | "align-text" => format!("text-align: {}", value),
        "text-decoration" | "decoration" => format!("text-decoration: {}", value),
        "text-transform" | "transform-text" | "case" => format!("text-transform: {}", value),
        "text-wrap" | "wrap-text" => format!("white-space: {}", if value == "true" { "normal" } else if value == "false" { "nowrap" } else { value }),
        "text-overflow" => format!("text-overflow: {}; overflow: hidden; white-space: nowrap", value),
        "text-indent" | "indent" => format!("text-indent: {}", value),
        "text-shadow" => format!("text-shadow: {}", value),
        "font" | "font-family" => format!("font-family: {}", value),
        "font-style" | "italic" => format!("font-style: {}", if value == "true" { "italic" } else { value }),
        "letter-spacing" | "tracking" => format!("letter-spacing: {}", value),
        "line-height" | "leading" => format!("line-height: {}", value),
        "word-spacing" => format!("word-spacing: {}", value),
        "word-break" => format!("word-break: {}", value),
        "hyphens" => format!("hyphens: {}", value),
        "vertical-align" | "v-align" => format!("vertical-align: {}", value),
        "writing-mode" => format!("writing-mode: {}", value),
        "text-direction" => format!("direction: {}", value),
        "text-columns" => format!("columns: {}", value),
        "truncate" => "overflow: hidden; text-overflow: ellipsis; white-space: nowrap".to_string(),
        "clamp" => format!("display: -webkit-box; -webkit-line-clamp: {}; -webkit-box-orient: vertical; overflow: hidden", value),
        "bg" | "background" => format!("background: {}", value),
        "bg-color" | "background-color" => format!("background-color: {}", value),
        "bg-gradient" | "gradient" => {
            if value.contains("to ") || value.contains("deg") {
                format!("background: linear-gradient({})", value)
            } else {
                format!("background: linear-gradient(135deg, {})", value)
            }
        },
        "bg-radial" => format!("background: radial-gradient({})", value),
        "bg-conic" => format!("background: conic-gradient({})", value),
        "bg-image" | "bg-img" => format!("background-image: url('{}')", value),
        "bg-size" => format!("background-size: {}", value),
        "bg-position" | "bg-pos" => format!("background-position: {}", value),
        "bg-repeat" => format!("background-repeat: {}", value),
        "bg-attachment" => format!("background-attachment: {}", value),
        "bg-clip" => format!("background-clip: {}", value),
        "bg-blend" => format!("background-blend-mode: {}", value),
        "shape" => match value {
            "circle" => "border-radius: 50%".to_string(),
            "pill" | "rounded-full" => "border-radius: 9999px".to_string(),
            "rect" | "square" => "border-radius: 0".to_string(),
            "rounded" => "border-radius: 8px".to_string(),
            "rounded-sm" => "border-radius: 4px".to_string(),
            "rounded-lg" => "border-radius: 12px".to_string(),
            "rounded-xl" => "border-radius: 16px".to_string(),
            "rounded-2xl" => "border-radius: 24px".to_string(),
            _ => format!("border-radius: {}", value),
        },
        "rounded" | "radius" | "border-radius" => format!("border-radius: {}", value),
        "rounded-top" => format!("border-top-left-radius: {}; border-top-right-radius: {}", value, value),
        "rounded-bottom" => format!("border-bottom-left-radius: {}; border-bottom-right-radius: {}", value, value),
        "rounded-left" => format!("border-top-left-radius: {}; border-bottom-left-radius: {}", value, value),
        "rounded-right" => format!("border-top-right-radius: {}; border-bottom-right-radius: {}", value, value),
        "opacity" | "alpha" => format!("opacity: {}", value),
        "border" => format!("border: {}", value),
        "border-color" => format!("border-color: {}", value),
        "border-width" => format!("border-width: {}", value),
        "border-style" => format!("border-style: {}", value),
        "border-top" => format!("border-top: {}", value),
        "border-bottom" => format!("border-bottom: {}", value),
        "border-left" => format!("border-left: {}", value),
        "border-right" => format!("border-right: {}", value),
        "border-x" => format!("border-left: {}; border-right: {}", value, value),
        "border-y" => format!("border-top: {}; border-bottom: {}", value, value),
        "outline" => format!("outline: {}", value),
        "outline-offset" => format!("outline-offset: {}", value),
        "ring" => format!("box-shadow: 0 0 0 {} currentColor", value),
        "ring-color" => format!("--ring-color: {}", value),
        "shadow" | "box-shadow" => {
            match value {
                "sm" => "box-shadow: 0 1px 2px rgba(0,0,0,0.05)".to_string(),
                "md" => "box-shadow: 0 4px 6px rgba(0,0,0,0.1)".to_string(),
                "lg" => "box-shadow: 0 10px 15px rgba(0,0,0,0.1)".to_string(),
                "xl" => "box-shadow: 0 20px 25px rgba(0,0,0,0.1)".to_string(),
                "2xl" => "box-shadow: 0 25px 50px rgba(0,0,0,0.25)".to_string(),
                "inner" => "box-shadow: inset 0 2px 4px rgba(0,0,0,0.1)".to_string(),
                "none" => "box-shadow: none".to_string(),
                _ => format!("box-shadow: {}", value),
            }
        },
        "drop-shadow" => format!("filter: drop-shadow({})", value),
        "effect" => match value {
            "blur" => "filter: blur(4px)".to_string(),
            "blur-sm" => "filter: blur(2px)".to_string(),
            "blur-lg" => "filter: blur(8px)".to_string(),
            "blur-xl" => "filter: blur(16px)".to_string(),
            "glow" => "box-shadow: 0 0 20px currentColor".to_string(),
            "neon" => "box-shadow: 0 0 10px currentColor, 0 0 20px currentColor, 0 0 40px currentColor".to_string(),
            "glass" | "glassmorphism" => "backdrop-filter: blur(10px); background: rgba(255,255,255,0.1); border: 1px solid rgba(255,255,255,0.2)".to_string(),
            "frost" => "backdrop-filter: blur(20px) saturate(180%); background: rgba(255,255,255,0.7)".to_string(),
            "noise" => "filter: url(#noise)".to_string(),
            "grayscale" => "filter: grayscale(100%)".to_string(),
            "sepia" => "filter: sepia(100%)".to_string(),
            "invert" => "filter: invert(100%)".to_string(),
            "saturate" => "filter: saturate(200%)".to_string(),
            "hue-rotate" => "filter: hue-rotate(90deg)".to_string(),
            _ => return None,
        },
        "filter" => format!("filter: {}", value),
        "blur" => format!("filter: blur({})", value),
        "brightness" => format!("filter: brightness({})", value),
        "contrast" => format!("filter: contrast({})", value),
        "grayscale" => format!("filter: grayscale({})", value),
        "hue-rotate" => format!("filter: hue-rotate({})", value),
        "invert" => format!("filter: invert({})", value),
        "saturate" => format!("filter: saturate({})", value),
        "sepia" => format!("filter: sepia({})", value),
        "backdrop" | "backdrop-filter" => format!("backdrop-filter: {}", value),
        "backdrop-blur" => format!("backdrop-filter: blur({})", value),
        "mix-blend" | "blend" => format!("mix-blend-mode: {}", value),
        "transform" => format!("transform: {}", value),
        "translate" => format!("transform: translate({})", value),
        "translate-x" => format!("transform: translateX({})", value),
        "translate-y" => format!("transform: translateY({})", value),
        "translate-z" => format!("transform: translateZ({})", value),
        "rotate" => format!("transform: rotate({})", value),
        "rotate-x" => format!("transform: rotateX({})", value),
        "rotate-y" => format!("transform: rotateY({})", value),
        "rotate-z" => format!("transform: rotateZ({})", value),
        "scale" => format!("transform: scale({})", value),
        "scale-x" => format!("transform: scaleX({})", value),
        "scale-y" => format!("transform: scaleY({})", value),
        "skew" => format!("transform: skew({})", value),
        "skew-x" => format!("transform: skewX({})", value),
        "skew-y" => format!("transform: skewY({})", value),
        "origin" | "transform-origin" => format!("transform-origin: {}", value),
        "perspective" => format!("perspective: {}", value),
        "perspective-origin" => format!("perspective-origin: {}", value),
        "transform-style" => format!("transform-style: {}", value),
        "backface" | "backface-visibility" => format!("backface-visibility: {}", value),
        "transition" => {
            match value {
                "all" => "transition: all 0.3s ease".to_string(),
                "fast" => "transition: all 0.15s ease".to_string(),
                "slow" => "transition: all 0.5s ease".to_string(),
                "none" => "transition: none".to_string(),
                _ => format!("transition: {}", value),
            }
        },
        "transition-property" => format!("transition-property: {}", value),
        "transition-duration" | "duration" => format!("transition-duration: {}", value),
        "transition-timing" | "ease" | "easing" => format!("transition-timing-function: {}", value),
        "transition-delay" => format!("transition-delay: {}", value),
        "animation" | "anim" => format!("animation: {} 0.5s ease", get_dew_animation(value)),
        "animation-name" => format!("animation-name: {}", value),
        "animation-duration" | "animation-speed" => format!("animation-duration: {}", value),
        "animation-delay" => format!("animation-delay: {}", value),
        "animation-timing" => format!("animation-timing-function: {}", value),
        "animation-iteration" | "animation-loop" => format!("animation-iteration-count: {}", if value == "true" || value == "infinite" { "infinite" } else { value }),
        "animation-direction" => format!("animation-direction: {}", value),
        "animation-fill" => format!("animation-fill-mode: {}", value),
        "animation-play" => format!("animation-play-state: {}", value),
        "cursor" => format!("cursor: {}", value),
        "pointer-events" | "pointer" => format!("pointer-events: {}", value),
        "user-select" | "select" => format!("user-select: {}", value),
        "touch-action" | "touch" => format!("touch-action: {}", value),
        "resize" => format!("resize: {}", value),
        "scroll-behavior" => format!("scroll-behavior: {}", value),
        "scroll-margin" => format!("scroll-margin: {}", value),
        "scroll-padding" => format!("scroll-padding: {}", value),
        "caret-color" | "caret" => format!("caret-color: {}", value),
        "accent-color" | "accent" => format!("accent-color: {}", value),
        "appearance" => format!("appearance: {}", value),
        "will-change" => format!("will-change: {}", value),
        "contain" => format!("contain: {}", value),
        "fill" => format!("fill: {}", value),
        "stroke" => format!("stroke: {}", value),
        "stroke-width" => format!("stroke-width: {}", value),
        "stroke-dasharray" => format!("stroke-dasharray: {}", value),
        "stroke-dashoffset" => format!("stroke-dashoffset: {}", value),
        "stroke-linecap" => format!("stroke-linecap: {}", value),
        "stroke-linejoin" => format!("stroke-linejoin: {}", value),
        "center" => "display: flex; align-items: center; justify-content: center".to_string(),
        "stack" => format!("display: flex; flex-direction: {}; gap: 10px", if value == "horizontal" || value == "row" { "row" } else { "column" }),
        "row" => "display: flex; flex-direction: row".to_string(),
        "col" | "column" => "display: flex; flex-direction: column".to_string(),
        "space-between" => "display: flex; justify-content: space-between".to_string(),
        "space-around" => "display: flex; justify-content: space-around".to_string(),
        "space-evenly" => "display: flex; justify-content: space-evenly".to_string(),
        "stretch" => "display: flex; align-items: stretch".to_string(),
        "absolute-center" => "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%)".to_string(),
        "fixed-center" => "position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%)".to_string(),
        "cover" => "position: absolute; top: 0; left: 0; right: 0; bottom: 0".to_string(),
        "clickable" => "cursor: pointer; user-select: none".to_string(),
        "disabled" => "opacity: 0.5; pointer-events: none; cursor: not-allowed".to_string(),
        "hidden" => "display: none".to_string(),
        "invisible" => "visibility: hidden".to_string(),
        "sr-only" => "position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0); border: 0".to_string(),
        "no-scroll" => "overflow: hidden".to_string(),
        "smooth-scroll" => "scroll-behavior: smooth".to_string(),
        "snap-x" => "scroll-snap-type: x mandatory; overflow-x: auto".to_string(),
        "snap-y" => "scroll-snap-type: y mandatory; overflow-y: auto".to_string(),
        "snap-start" => "scroll-snap-align: start".to_string(),
        "snap-center" => "scroll-snap-align: center".to_string(),
        "snap-end" => "scroll-snap-align: end".to_string(),
        "container" => "width: 100%; max-width: 1200px; margin-left: auto; margin-right: auto; padding-left: 1rem; padding-right: 1rem".to_string(),
        "full" => "width: 100%; height: 100%".to_string(),
        "screen" => "width: 100vw; height: 100vh".to_string(),
        "min-screen" => "min-width: 100vw; min-height: 100vh".to_string(),
        
        // Additional advanced styling properties
        "line-clamp" => format!("display: -webkit-box; -webkit-line-clamp: {}; -webkit-box-orient: vertical", value),
        "text-stroke" | "webkit-text-stroke" => format!("-webkit-text-stroke: {}", value),
        "text-fill" | "webkit-text-fill" => format!("-webkit-text-fill-color: {}", value),
        "background-clip" => format!("background-clip: {}", value),
        "text-background-clip" => "background-clip: text; -webkit-background-clip: text; -webkit-text-fill-color: transparent".to_string(),
        "font-smoothing" => format!("-webkit-font-smoothing: {}; -moz-osx-font-smoothing: {}", 
            if value == "auto" { "auto" } else { "antialiased" },
            if value == "auto" { "auto" } else { "grayscale" }),
        
        // Gradient and background advanced properties
        "gradient-text" => "background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text".to_string(),
        "gradient-slow" => "background: linear-gradient(360deg, #667eea, #764ba2, #f093fb, #4facfe, #667eea); background-size: 400% 400%; animation: gradient 15s ease infinite".to_string(),
        "gradient-fast" => "background: linear-gradient(360deg, #667eea, #764ba2, #f093fb, #4facfe, #667eea); background-size: 400% 400%; animation: gradient 4s ease infinite".to_string(),
        
        // Layout and grid helpers
        "auto-grid" => format!("display: grid; grid-template-columns: repeat(auto-fit, minmax({}, 1fr)); gap: 1rem", value),
        "auto-flow" => format!("display: grid; grid-auto-flow: {}; grid-auto-columns: minmax(0, 1fr)", value),
        "subgrid" => "display: grid; grid-template-columns: subgrid; grid-template-rows: subgrid".to_string(),
        
        // Positioning utilities
        "sticky" => "position: sticky".to_string(),
        "sticky-top" => "position: sticky; top: 0; z-index: 10".to_string(),
        "sticky-bottom" => "position: sticky; bottom: 0; z-index: 10".to_string(),
        "overlay" => "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 50".to_string(),
        "overlay-dark" => "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); z-index: 50".to_string(),
        "overlay-light" => "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(255,255,255,0.8); z-index: 50".to_string(),
        
        // Advanced text properties
        "ligatures" => "-webkit-font-feature-settings: 'liga'; font-feature-settings: 'liga'".to_string(),
        "no-ligatures" => "-webkit-font-feature-settings: 'liga' 0; font-feature-settings: 'liga' 0".to_string(),
        "tabular-nums" => "-webkit-font-feature-settings: 'tnum'; font-feature-settings: 'tnum'".to_string(),
        "slashed-zero" => "-webkit-font-feature-settings: 'zero'; font-feature-settings: 'zero'".to_string(),
        
        // Print utilities
        "print-hide" => "@media print { display: none }".to_string(),
        "print-show" => "@media print { display: block }".to_string(),
        "print-landscape" => "@media print and (orientation: landscape) { transform: rotate(90deg) }".to_string(),
        
        // Motion and reduced motion
        "motion-safe" => "@media (prefers-reduced-motion: no-preference) { transition: all 0.3s ease }".to_string(),
        "motion-reduce" => "@media (prefers-reduced-motion: reduce) { animation: none; transition: none }".to_string(),
        
        // Dark mode utilities
        "dark" => "@media (prefers-color-scheme: dark) { filter: invert(1) }".to_string(),
        "dark-bg" => "@media (prefers-color-scheme: dark) { background-color: #1a1a1a; color: #ffffff }".to_string(),
        "light-bg" => "@media (prefers-color-scheme: light) { background-color: #ffffff; color: #1a1a1a }".to_string(),
        
        // High contrast utilities
        "high-contrast" => "@media (prefers-contrast: more) { border-width: 2px; font-weight: bold }".to_string(),
        
        // Advanced transform combinations
        "flip-x" => "transform: scaleX(-1)".to_string(),
        "flip-y" => "transform: scaleY(-1)".to_string(),
        "flip" => "transform: scale(-1)".to_string(),
        "rotate-45" => "transform: rotate(45deg)".to_string(),
        "rotate-90" => "transform: rotate(90deg)".to_string(),
        "rotate-180" => "transform: rotate(180deg)".to_string(),
        "scale-75" => "transform: scale(0.75)".to_string(),
        "scale-125" => "transform: scale(1.25)".to_string(),
        "scale-150" => "transform: scale(1.5)".to_string(),
        
        // Neumorphism utilities
        "neumorphic" => "background: #e0e5ec; box-shadow: 9px 9px 16px #a3b1c6, -9px -9px 16px #ffffff; border-radius: 20px".to_string(),
        "neumorphic-inset" => "background: #e0e5ec; box-shadow: inset 9px 9px 16px #a3b1c6, inset -9px -9px 16px #ffffff; border-radius: 20px".to_string(),
        
        // Glassmorphism variants
        "glass-thin" => "backdrop-filter: blur(5px); background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1)".to_string(),
        "glass-thick" => "backdrop-filter: blur(20px); background: rgba(255,255,255,0.15); border: 1px solid rgba(255,255,255,0.3)".to_string(),
        
        // Fluid typography
        "text-fluid" => "font-size: clamp(1rem, 2.5vw, 2rem)".to_string(),
        "text-fluid-sm" => "font-size: clamp(0.875rem, 1.5vw, 1.25rem)".to_string(),
        "text-fluid-lg" => "font-size: clamp(1.25rem, 3vw, 2.5rem)".to_string(),
        
        _ => return None,
    };
    Some(css)
}
fn get_dew_animation(anim: &str) -> &str {
    match anim {
        "fade-in" => "dew-fade-in", "fade-out" => "dew-fade-out",
        "pop-in" => "dew-pop-in", "pop-out" => "dew-pop-out",
        "appear" => "dew-fade-in", "disappear" => "dew-fade-out",
        "vanish" => "dew-fade-out", "unvanish" => "dew-fade-in",
        "bounce" => "dew-bounce", "shake" => "dew-shake",
        "wiggle" => "dew-wiggle", "float" => "dew-float",
        "jump" => "dew-jump", "swing" => "dew-swing",
        "wobble" => "dew-wobble", "jello" => "dew-jello",
        "rubber" => "dew-rubber", "tada" => "dew-tada",
        "heartbeat" => "dew-heartbeat", "ping" => "dew-ping",
        "slide-up" => "dew-slide-up", "slide-down" => "dew-slide-down",
        "slide-left" => "dew-slide-left", "slide-right" => "dew-slide-right",
        "slide-fade-in" => "dew-slide-fade-in", "slide-fade-out" => "dew-slide-fade-out",
        "rotate" => "dew-rotate", "spin" => "dew-spin",
        "flip" => "dew-flip", "flip-x" => "dew-flip-x", "flip-y" => "dew-flip-y",
        "scale-up" => "dew-scale-up", "scale-down" => "dew-scale-down",
        "zoom-in" => "dew-zoom-in", "zoom-out" => "dew-zoom-out",
        "grow" => "dew-grow", "shrink" => "dew-shrink",
        "pulse" => "dew-pulse", "glow" => "dew-glow",
        "neon" => "dew-neon", "flash" => "dew-flash",
        "flicker" => "dew-flicker", "highlight" => "dew-highlight",
        "dim" => "dew-dim", "rainbow" => "dew-rainbow",
        "color-cycle" => "dew-color-cycle",
        "thanos" => "dew-thanos", "scatter" => "dew-scatter",
        "assemble" => "dew-assemble", "explode" => "dew-explode",
        "implode" => "dew-implode", "morph" => "dew-morph",
        "blur-in" => "dew-blur-in", "blur-out" => "dew-blur-out",
        "pixelate" => "dew-pixelate", "glitch" => "dew-glitch",
        "typewriter" => "dew-typewriter", "wave" => "dew-wave",
        "drop-in" => "dew-drop-in", "lift-out" => "dew-lift-out",
        "fold-in" => "dew-fold-in", "fold-out" => "dew-fold-out",
        "curtain-open" => "dew-curtain-open", "curtain-close" => "dew-curtain-close",
        "reveal" => "dew-reveal", "conceal" => "dew-conceal",
        "spring" => "dew-spring", "elastic" => "dew-elastic",
        "gravity-drop" => "dew-gravity-drop", "gravity-rise" => "dew-gravity-rise",
        "magnet" => "dew-magnet", "repel" => "dew-repel",
        "tap" => "dew-tap", "press" => "dew-press",
        "hover-rise" => "dew-hover-rise", "hover-glow" => "dew-hover-glow",
        "focus-ring" => "dew-focus-ring", "click-ripple" => "dew-click-ripple",
        _ => anim,
    }
}
fn generate_dew_event_attrs(events: &[(String, String)]) -> String {
    let mut attrs = String::new();
    for (event, body) in events {
        let event_name = match event.as_str() {
            "click" => "onclick",
            "hover" => "onmouseenter",
            "focus" => "onfocus",
            "blur" => "onblur",
            _ => continue,
        };
        let handler = generate_dew_handler(body);
        attrs.push_str(&format!(" {}=\"{}\"", event_name, handler));
    }
    attrs
}
fn generate_dew_handler(body: &str) -> String {
    let mut js = String::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if line == "increment" { js.push_str("Dew.increment(this);"); }
        else if line == "decrement" { js.push_str("Dew.decrement(this);"); }
        else if line.starts_with("effect:") {
            let effect = line[7..].trim();
            js.push_str(&format!("Dew.effect(this,'{}');", effect));
        }
        else if line.starts_with("emit ") {
            let event = line[5..].trim().trim_matches('"');
            js.push_str(&format!("Dew.emit('{}');", event));
        }
        else if line.starts_with("toggle ") {
            let target = line[7..].trim().trim_matches('"');
            js.push_str(&format!("Dew.toggle('{}');", target));
        }
        else if line.starts_with("show ") {
            let target = line[5..].trim().trim_matches('"');
            js.push_str(&format!("Dew.show('{}');", target));
        }
        else if line.starts_with("hide ") {
            let target = line[5..].trim().trim_matches('"');
            js.push_str(&format!("Dew.hide('{}');", target));
        }
        else { js.push_str(line); js.push(';'); }
    }
    js.replace('"', "&quot;")
}
fn inject_dew_frontend_script(template: &str, data: &HashMap<String, Value>) -> String {
    if !template.contains("</body>") {
        return template.to_string();
    }
    let mut js_data = String::from("{\n");
    for (key, value) in data {
        if !key.starts_with("__") {
            js_data.push_str(&format!("    {}: {},\n", key, value_to_js(value)));
        }
    }
    js_data.push_str("  }");
    let mut dew_runtime = String::from(r#"
<style>
/* ==================== DEW ANIMATION KEYFRAMES ==================== */
/* Visibility & Entrance */
@keyframes dew-fade-in {{ from {{ opacity: 0; }} to {{ opacity: 1; }} }}
@keyframes dew-fade-out {{ from {{ opacity: 1; }} to {{ opacity: 0; }} }}
@keyframes dew-pop-in {{ from {{ transform: scale(0); opacity: 0; }} to {{ transform: scale(1); opacity: 1; }} }}
@keyframes dew-pop-out {{ from {{ transform: scale(1); opacity: 1; }} to {{ transform: scale(0); opacity: 0; }} }}
/* Motion */
@keyframes dew-bounce {{ 0%, 100% {{ transform: translateY(0); }} 50% {{ transform: translateY(-20px); }} }}
@keyframes dew-shake {{ 0%, 100% {{ transform: translateX(0); }} 10%, 30%, 50%, 70%, 90% {{ transform: translateX(-5px); }} 20%, 40%, 60%, 80% {{ transform: translateX(5px); }} }}
@keyframes dew-wiggle {{ 0%, 100% {{ transform: rotate(0deg); }} 25% {{ transform: rotate(-5deg); }} 75% {{ transform: rotate(5deg); }} }}
@keyframes dew-float {{ 0%, 100% {{ transform: translateY(0); }} 50% {{ transform: translateY(-10px); }} }}
@keyframes dew-jump {{ 0%, 100% {{ transform: translateY(0); }} 30% {{ transform: translateY(-30px); }} 50% {{ transform: translateY(-15px); }} }}
@keyframes dew-swing {{ 0%, 100% {{ transform: rotate(0deg); }} 20% {{ transform: rotate(15deg); }} 40% {{ transform: rotate(-10deg); }} 60% {{ transform: rotate(5deg); }} 80% {{ transform: rotate(-5deg); }} }}
@keyframes dew-wobble {{ 0%, 100% {{ transform: translateX(0) rotate(0); }} 15% {{ transform: translateX(-25px) rotate(-5deg); }} 30% {{ transform: translateX(20px) rotate(3deg); }} 45% {{ transform: translateX(-15px) rotate(-3deg); }} 60% {{ transform: translateX(10px) rotate(2deg); }} 75% {{ transform: translateX(-5px) rotate(-1deg); }} }}
@keyframes dew-jello {{ 0%, 100% {{ transform: scale3d(1,1,1); }} 30% {{ transform: scale3d(1.25,0.75,1); }} 40% {{ transform: scale3d(0.75,1.25,1); }} 50% {{ transform: scale3d(1.15,0.85,1); }} 65% {{ transform: scale3d(0.95,1.05,1); }} 75% {{ transform: scale3d(1.05,0.95,1); }} }}
@keyframes dew-rubber {{ 0%, 100% {{ transform: scale3d(1,1,1); }} 30% {{ transform: scale3d(1.25,0.75,1); }} 40% {{ transform: scale3d(0.75,1.25,1); }} 50% {{ transform: scale3d(1.15,0.85,1); }} 65% {{ transform: scale3d(0.95,1.05,1); }} 75% {{ transform: scale3d(1.05,0.95,1); }} }}
@keyframes dew-tada {{ 0%, 100% {{ transform: scale(1) rotate(0); }} 10%, 20% {{ transform: scale(0.9) rotate(-3deg); }} 30%, 50%, 70%, 90% {{ transform: scale(1.1) rotate(3deg); }} 40%, 60%, 80% {{ transform: scale(1.1) rotate(-3deg); }} }}
@keyframes dew-heartbeat {{ 0%, 100% {{ transform: scale(1); }} 14% {{ transform: scale(1.3); }} 28% {{ transform: scale(1); }} 42% {{ transform: scale(1.3); }} 70% {{ transform: scale(1); }} }}
@keyframes dew-ping {{ 0% {{ transform: scale(1); opacity: 1; }} 75%, 100% {{ transform: scale(2); opacity: 0; }} }}
/* Slides */
@keyframes dew-slide-up {{ from {{ transform: translateY(100%); opacity: 0; }} to {{ transform: translateY(0); opacity: 1; }} }}
@keyframes dew-slide-down {{ from {{ transform: translateY(-100%); opacity: 0; }} to {{ transform: translateY(0); opacity: 1; }} }}
@keyframes dew-slide-left {{ from {{ transform: translateX(100%); opacity: 0; }} to {{ transform: translateX(0); opacity: 1; }} }}
@keyframes dew-slide-right {{ from {{ transform: translateX(-100%); opacity: 0; }} to {{ transform: translateX(0); opacity: 1; }} }}
@keyframes dew-slide-fade-in {{ from {{ transform: translateY(20px); opacity: 0; }} to {{ transform: translateY(0); opacity: 1; }} }}
@keyframes dew-slide-fade-out {{ from {{ transform: translateY(0); opacity: 1; }} to {{ transform: translateY(20px); opacity: 0; }} }}
/* Transforms */
@keyframes dew-rotate {{ from {{ transform: rotate(0deg); }} to {{ transform: rotate(360deg); }} }}
@keyframes dew-spin {{ from {{ transform: rotate(0deg); }} to {{ transform: rotate(360deg); }} }}
@keyframes dew-flip {{ 0% {{ transform: perspective(400px) rotateY(0); }} 100% {{ transform: perspective(400px) rotateY(360deg); }} }}
@keyframes dew-flip-x {{ 0% {{ transform: perspective(400px) rotateX(0); }} 100% {{ transform: perspective(400px) rotateX(360deg); }} }}
@keyframes dew-flip-y {{ 0% {{ transform: perspective(400px) rotateY(0); }} 100% {{ transform: perspective(400px) rotateY(360deg); }} }}
@keyframes dew-scale-up {{ from {{ transform: scale(0.5); opacity: 0; }} to {{ transform: scale(1); opacity: 1; }} }}
@keyframes dew-scale-down {{ from {{ transform: scale(1.5); opacity: 0; }} to {{ transform: scale(1); opacity: 1; }} }}
@keyframes dew-zoom-in {{ from {{ transform: scale(0); }} to {{ transform: scale(1); }} }}
@keyframes dew-zoom-out {{ from {{ transform: scale(1); }} to {{ transform: scale(0); }} }}
@keyframes dew-grow {{ from {{ transform: scale(1); }} to {{ transform: scale(1.1); }} }}
@keyframes dew-shrink {{ from {{ transform: scale(1); }} to {{ transform: scale(0.9); }} }}
/* Color & Light */
@keyframes dew-pulse {{ 0%, 100% {{ opacity: 1; }} 50% {{ opacity: 0.5; }} }}
@keyframes dew-glow {{ 0%, 100% {{ box-shadow: 0 0 5px currentColor; }} 50% {{ box-shadow: 0 0 25px currentColor, 0 0 50px currentColor; }} }}
@keyframes dew-neon {{ 0%, 100% {{ box-shadow: 0 0 5px #fff, 0 0 10px #fff, 0 0 20px var(--neon-color, #0ff), 0 0 40px var(--neon-color, #0ff); }} 50% {{ box-shadow: 0 0 10px #fff, 0 0 20px #fff, 0 0 40px var(--neon-color, #0ff), 0 0 80px var(--neon-color, #0ff); }} }}
@keyframes dew-flash {{ 0%, 50%, 100% {{ opacity: 1; }} 25%, 75% {{ opacity: 0; }} }}
@keyframes dew-flicker {{ 0%, 100% {{ opacity: 1; }} 33% {{ opacity: 0.8; }} 66% {{ opacity: 0.4; }} }}
@keyframes dew-highlight {{ 0% {{ background-color: transparent; }} 50% {{ background-color: rgba(255,255,0,0.3); }} 100% {{ background-color: transparent; }} }}
@keyframes dew-dim {{ from {{ opacity: 1; }} to {{ opacity: 0.3; }} }}
@keyframes dew-rainbow {{ 0% {{ filter: hue-rotate(0deg); }} 100% {{ filter: hue-rotate(360deg); }} }}
@keyframes dew-color-cycle {{ 0% {{ color: #ff0000; }} 17% {{ color: #ff8000; }} 33% {{ color: #ffff00; }} 50% {{ color: #00ff00; }} 67% {{ color: #0080ff; }} 83% {{ color: #8000ff; }} 100% {{ color: #ff0000; }} }}
/* Fun & Special */
@keyframes dew-thanos {{ 0% {{ opacity: 1; filter: blur(0); transform: scale(1); }} 100% {{ opacity: 0; filter: blur(10px); transform: scale(0.5) rotate(15deg); }} }}
@keyframes dew-scatter {{ 0% {{ opacity: 1; transform: scale(1); }} 100% {{ opacity: 0; transform: scale(2) rotate(180deg); filter: blur(5px); }} }}
@keyframes dew-assemble {{ 0% {{ opacity: 0; transform: scale(2) rotate(-180deg); filter: blur(5px); }} 100% {{ opacity: 1; transform: scale(1) rotate(0); filter: blur(0); }} }}
@keyframes dew-explode {{ 0% {{ transform: scale(1); opacity: 1; }} 100% {{ transform: scale(3); opacity: 0; }} }}
@keyframes dew-implode {{ 0% {{ transform: scale(3); opacity: 0; }} 100% {{ transform: scale(1); opacity: 1; }} }}
@keyframes dew-morph {{ 0%, 100% {{ border-radius: 10%; }} 50% {{ border-radius: 50%; }} }}
@keyframes dew-blur-in {{ from {{ filter: blur(20px); opacity: 0; }} to {{ filter: blur(0); opacity: 1; }} }}
@keyframes dew-blur-out {{ from {{ filter: blur(0); opacity: 1; }} to {{ filter: blur(20px); opacity: 0; }} }}
@keyframes dew-pixelate {{ 0%, 100% {{ filter: blur(0); }} 50% {{ filter: blur(2px); image-rendering: pixelated; }} }}
@keyframes dew-glitch {{ 0%, 100% {{ transform: translate(0); }} 20% {{ transform: translate(-2px, 2px); filter: hue-rotate(90deg); }} 40% {{ transform: translate(-2px, -2px); }} 60% {{ transform: translate(2px, 2px); filter: hue-rotate(-90deg); }} 80% {{ transform: translate(2px, -2px); }} }}
@keyframes dew-wave {{ 0%, 100% {{ transform: translateY(0); }} 25% {{ transform: translateY(-5px); }} 75% {{ transform: translateY(5px); }} }}
/* Cinematic */
@keyframes dew-drop-in {{ 0% {{ transform: translateY(-500px); opacity: 0; }} 60% {{ transform: translateY(25px); opacity: 1; }} 75% {{ transform: translateY(-10px); }} 90% {{ transform: translateY(5px); }} 100% {{ transform: translateY(0); }} }}
@keyframes dew-lift-out {{ 0% {{ transform: translateY(0); opacity: 1; }} 100% {{ transform: translateY(-500px); opacity: 0; }} }}
@keyframes dew-fold-in {{ 0% {{ transform: perspective(400px) rotateX(90deg); opacity: 0; }} 100% {{ transform: perspective(400px) rotateX(0); opacity: 1; }} }}
@keyframes dew-fold-out {{ 0% {{ transform: perspective(400px) rotateX(0); opacity: 1; }} 100% {{ transform: perspective(400px) rotateX(90deg); opacity: 0; }} }}
@keyframes dew-curtain-open {{ 0% {{ transform: scaleX(0); transform-origin: left; }} 100% {{ transform: scaleX(1); transform-origin: left; }} }}
@keyframes dew-curtain-close {{ 0% {{ transform: scaleX(1); transform-origin: right; }} 100% {{ transform: scaleX(0); transform-origin: right; }} }}
@keyframes dew-reveal {{ 0% {{ clip-path: inset(0 100% 0 0); }} 100% {{ clip-path: inset(0 0 0 0); }} }}
@keyframes dew-conceal {{ 0% {{ clip-path: inset(0 0 0 0); }} 100% {{ clip-path: inset(0 100% 0 0); }} }}
/* Physics */
@keyframes dew-spring {{ 0% {{ transform: scale(0); }} 30% {{ transform: scale(1.2); }} 50% {{ transform: scale(0.9); }} 70% {{ transform: scale(1.05); }} 85% {{ transform: scale(0.98); }} 100% {{ transform: scale(1); }} }}
@keyframes dew-elastic {{ 0% {{ transform: scale(0); }} 55% {{ transform: scale(1); }} 70% {{ transform: scale(1.3); }} 85% {{ transform: scale(0.9); }} 100% {{ transform: scale(1); }} }}
@keyframes dew-gravity-drop {{ 0% {{ transform: translateY(-200px); }} 60% {{ transform: translateY(30px); }} 80% {{ transform: translateY(-10px); }} 100% {{ transform: translateY(0); }} }}
@keyframes dew-gravity-rise {{ 0% {{ transform: translateY(200px); }} 60% {{ transform: translateY(-30px); }} 80% {{ transform: translateY(10px); }} 100% {{ transform: translateY(0); }} }}
@keyframes dew-magnet {{ 0% {{ transform: translateX(-100px); }} 70% {{ transform: translateX(10px); }} 100% {{ transform: translateX(0); }} }}
@keyframes dew-repel {{ 0% {{ transform: translateX(0); }} 100% {{ transform: translateX(100px); opacity: 0; }} }}
/* Micro-interactions */
@keyframes dew-tap {{ 0%, 100% {{ transform: scale(1); }} 50% {{ transform: scale(0.95); }} }}
@keyframes dew-press {{ 0% {{ transform: scale(1); }} 100% {{ transform: scale(0.9); }} }}
@keyframes dew-hover-rise {{ 0% {{ transform: translateY(0); }} 100% {{ transform: translateY(-5px); }} }}
@keyframes dew-hover-glow {{ 0% {{ box-shadow: 0 0 0 transparent; }} 100% {{ box-shadow: 0 5px 20px rgba(0,0,0,0.2); }} }}
@keyframes dew-focus-ring {{ 0% {{ box-shadow: 0 0 0 0 rgba(59,130,246,0.5); }} 100% {{ box-shadow: 0 0 0 4px rgba(59,130,246,0.5); }} }}
@keyframes dew-click-ripple {{ 0% {{ transform: scale(0); opacity: 1; }} 100% {{ transform: scale(4); opacity: 0; }} }}
/* Typewriter */
@keyframes dew-typewriter {{ from {{ width: 0; }} to {{ width: 100%; }} }}
@keyframes dew-blink {{ 0%, 100% {{ border-color: transparent; }} 50% {{ border-color: currentColor; }} }}
/* Utility classes */
.dew-component {{ transition: all 0.3s ease; }}
.dew-hidden {{ display: none !important; }}
.dew-invisible {{ visibility: hidden !important; }}
.dew-no-transition {{ transition: none !important; }}
</style>
<script>
// ==================== DEW FRONTEND RUNTIME v2.0 ====================
window.Dew = (function() {{
  var D = {{
    data: {{}},
    _listeners: {{}},
    _watchers: {{}},
    version: '2.0.0',
    // ==================== STATE MANAGEMENT ====================
    get: function(key) {{ return D.data[key]; }},
    set: function(key, value) {{
      var old = D.data[key];
      D.data[key] = value;
      D.update();
      D.emit('state:' + key, {{ value: value, old: old }});
      if (D._watchers[key]) D._watchers[key].forEach(function(cb) {{ cb(value, old); }});
    }},
    watch: function(key, callback) {{
      if (!D._watchers[key]) D._watchers[key] = [];
      D._watchers[key].push(callback);
    }},
    increment: function(el, amount) {{
      var key = (typeof el === 'string') ? el : (el.getAttribute('dew-bind') || 'count');
      D.set(key, (D.get(key) || 0) + (amount || 1));
    }},
    decrement: function(el, amount) {{
      var key = (typeof el === 'string') ? el : (el.getAttribute('dew-bind') || 'count');
      D.set(key, (D.get(key) || 0) - (amount || 1));
    }},
    toggle: function(key) {{
      if (typeof key === 'string' && !key.startsWith('#') && !key.startsWith('.')) {{
        D.set(key, !D.get(key));
      }} else {{
        D.toggleVisibility(key);
      }}
    }},
    reset: function(key) {{ D.set(key, D._defaults[key] || null); }},
    // ==================== DOM UPDATES ====================
    update: function() {{
      document.querySelectorAll('[dew-bind]').forEach(function(el) {{
        var key = el.getAttribute('dew-bind');
        if (D.data[key] !== undefined) {{
          if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {{
            el.value = D.data[key];
          }} else {{
            el.textContent = D.data[key];
          }}
        }}
      }});
      document.querySelectorAll('[dew-html]').forEach(function(el) {{
        var key = el.getAttribute('dew-html');
        if (D.data[key] !== undefined) el.innerHTML = D.data[key];
      }});
      document.querySelectorAll('[dew-show]').forEach(function(el) {{
        var key = el.getAttribute('dew-show');
        el.style.display = D.data[key] ? '' : 'none';
      }});
      document.querySelectorAll('[dew-hide]').forEach(function(el) {{
        var key = el.getAttribute('dew-hide');
        el.style.display = D.data[key] ? 'none' : '';
      }});
      document.querySelectorAll('[dew-class]').forEach(function(el) {{
        var expr = el.getAttribute('dew-class');
        try {{
          var classes = JSON.parse(expr.replace(/'/g, '"'));
          Object.keys(classes).forEach(function(cls) {{
            el.classList.toggle(cls, !!D.data[classes[cls]]);
          }});
        }} catch(e) {{}}
      }});
    }},
    // ==================== VISIBILITY ====================
    show: function(sel, anim) {{
      D.$(sel).forEach(function(el) {{
        el.style.display = '';
        el.style.opacity = '1';
        if (anim) D.effect(el, anim);
      }});
    }},
    hide: function(sel, anim) {{
      D.$(sel).forEach(function(el) {{
        if (anim) {{
          D.effect(el, anim);
          setTimeout(function() {{ el.style.display = 'none'; }}, 500);
        }} else {{
          el.style.display = 'none';
        }}
      }});
    }},
    toggleVisibility: function(sel) {{
      D.$(sel).forEach(function(el) {{
        el.style.display = el.style.display === 'none' ? '' : 'none';
      }});
    }},
    fadeIn: function(sel, duration) {{
      D.$(sel).forEach(function(el) {{
        el.style.transition = 'opacity ' + (duration || 300) + 'ms';
        el.style.opacity = '0';
        el.style.display = '';
        setTimeout(function() {{ el.style.opacity = '1'; }}, 10);
      }});
    }},
    fadeOut: function(sel, duration) {{
      D.$(sel).forEach(function(el) {{
        el.style.transition = 'opacity ' + (duration || 300) + 'ms';
        el.style.opacity = '0';
        setTimeout(function() {{ el.style.display = 'none'; }}, duration || 300);
      }});
    }},
    // ==================== EFFECTS & ANIMATIONS ====================
    effect: function(el, name, opts) {{
      var target = typeof el === 'string' ? document.querySelector(el) : el;
      if (!target) return;
      opts = opts || {{}};
      var duration = opts.duration || opts.speed || '0.5s';
      var delay = opts.delay || '0s';
      var loop = opts.loop ? 'infinite' : '1';
      var ease = opts.ease || 'ease';
      var anims = {{
        'fade-in': 'dew-fade-in', 'fade-out': 'dew-fade-out',
        'pop-in': 'dew-pop-in', 'pop-out': 'dew-pop-out',
        'bounce': 'dew-bounce', 'shake': 'dew-shake',
        'wiggle': 'dew-wiggle', 'float': 'dew-float',
        'jump': 'dew-jump', 'swing': 'dew-swing',
        'wobble': 'dew-wobble', 'jello': 'dew-jello',
        'rubber': 'dew-rubber', 'tada': 'dew-tada',
        'heartbeat': 'dew-heartbeat', 'ping': 'dew-ping',
        'slide-up': 'dew-slide-up', 'slide-down': 'dew-slide-down',
        'slide-left': 'dew-slide-left', 'slide-right': 'dew-slide-right',
        'slide-fade-in': 'dew-slide-fade-in', 'slide-fade-out': 'dew-slide-fade-out',
        'rotate': 'dew-rotate', 'spin': 'dew-spin',
        'flip': 'dew-flip', 'flip-x': 'dew-flip-x', 'flip-y': 'dew-flip-y',
        'scale-up': 'dew-scale-up', 'scale-down': 'dew-scale-down',
        'zoom-in': 'dew-zoom-in', 'zoom-out': 'dew-zoom-out',
        'grow': 'dew-grow', 'shrink': 'dew-shrink',
        'pulse': 'dew-pulse', 'glow': 'dew-glow',
        'neon': 'dew-neon', 'flash': 'dew-flash',
        'flicker': 'dew-flicker', 'highlight': 'dew-highlight',
        'dim': 'dew-dim', 'rainbow': 'dew-rainbow',
        'color-cycle': 'dew-color-cycle',
        'thanos': 'dew-thanos', 'scatter': 'dew-scatter',
        'assemble': 'dew-assemble', 'explode': 'dew-explode',
        'implode': 'dew-implode', 'morph': 'dew-morph',
        'blur-in': 'dew-blur-in', 'blur-out': 'dew-blur-out',
        'pixelate': 'dew-pixelate', 'glitch': 'dew-glitch',
        'wave': 'dew-wave',
        'drop-in': 'dew-drop-in', 'lift-out': 'dew-lift-out',
        'fold-in': 'dew-fold-in', 'fold-out': 'dew-fold-out',
        'curtain-open': 'dew-curtain-open', 'curtain-close': 'dew-curtain-close',
        'reveal': 'dew-reveal', 'conceal': 'dew-conceal',
        'spring': 'dew-spring', 'elastic': 'dew-elastic',
        'gravity-drop': 'dew-gravity-drop', 'gravity-rise': 'dew-gravity-rise',
        'magnet': 'dew-magnet', 'repel': 'dew-repel',
        'tap': 'dew-tap', 'press': 'dew-press',
        'hover-rise': 'dew-hover-rise', 'hover-glow': 'dew-hover-glow',
        'focus-ring': 'dew-focus-ring', 'click-ripple': 'dew-click-ripple'
      }};
      var animName = anims[name] || name;
      target.style.animation = animName + ' ' + duration + ' ' + ease + ' ' + delay + ' ' + loop;
      if (!opts.loop) {{
        target.addEventListener('animationend', function handler() {{
          target.style.animation = '';
          target.removeEventListener('animationend', handler);
        }});
      }}
    }},
    stopEffect: function(el) {{
      var target = typeof el === 'string' ? document.querySelector(el) : el;
      if (target) target.style.animation = '';
    }},
    // ==================== SIGNALS (JS ↔ Dew Bridge) ====================
    signal: function(id, props) {{
      var el = document.getElementById(id);
      if (!el) return;
      var themes = {{
        'danger': {{ bg: '#ef4444', color: '#fff' }},
        'success': {{ bg: '#22c55e', color: '#fff' }},
        'warning': {{ bg: '#f59e0b', color: '#000' }},
        'info': {{ bg: '#3b82f6', color: '#fff' }},
        'dark': {{ bg: '#1f2937', color: '#fff' }},
        'light': {{ bg: '#f3f4f6', color: '#000' }},
        'primary': {{ bg: '#6366f1', color: '#fff' }},
        'secondary': {{ bg: '#64748b', color: '#fff' }}
      }};
      if (props.theme && themes[props.theme]) {{
        el.style.backgroundColor = themes[props.theme].bg;
        el.style.color = themes[props.theme].color;
      }}
      if (props.animation) D.effect(el, props.animation);
      if (props.text) el.textContent = props.text;
      if (props.html) el.innerHTML = props.html;
      Object.keys(props).forEach(function(k) {{
        if (!['theme', 'animation', 'text', 'html'].includes(k)) {{
          el.style[k] = props[k];
        }}
      }});
      D.emit('signal:' + id, props);
    }},
    // ==================== EVENTS ====================
    emit: function(event, data) {{
      var listeners = D._listeners[event] || [];
      listeners.forEach(function(cb) {{ cb(data); }});
      window.dispatchEvent(new CustomEvent('dew:' + event, {{ detail: data }}));
    }},
    on: function(event, callback) {{
      if (!D._listeners[event]) D._listeners[event] = [];
      D._listeners[event].push(callback);
      return function() {{ D.off(event, callback); }};
    }},
    off: function(event, callback) {{
      if (!D._listeners[event]) return;
      D._listeners[event] = D._listeners[event].filter(function(cb) {{ return cb !== callback; }});
    }},
    once: function(event, callback) {{
      var unsub = D.on(event, function(data) {{
        unsub();
        callback(data);
      }});
    }},
    // ==================== DOM HELPERS ====================
    $: function(sel) {{
      if (typeof sel === 'string') return Array.from(document.querySelectorAll(sel));
      return [sel];
    }},
    $$: function(sel) {{ return document.querySelector(sel); }},
    create: function(tag, attrs, children) {{
      var el = document.createElement(tag);
      if (attrs) Object.keys(attrs).forEach(function(k) {{
        if (k === 'class') el.className = attrs[k];
        else if (k === 'style' && typeof attrs[k] === 'object') Object.assign(el.style, attrs[k]);
        else if (k.startsWith('on')) el.addEventListener(k.slice(2).toLowerCase(), attrs[k]);
        else el.setAttribute(k, attrs[k]);
      }});
      if (children) {{
        if (typeof children === 'string') el.textContent = children;
        else if (Array.isArray(children)) children.forEach(function(c) {{ el.appendChild(c); }});
        else el.appendChild(children);
      }}
      return el;
    }},
    append: function(parent, child) {{
      var p = typeof parent === 'string' ? document.querySelector(parent) : parent;
      if (p) p.appendChild(typeof child === 'string' ? document.createTextNode(child) : child);
    }},
    prepend: function(parent, child) {{
      var p = typeof parent === 'string' ? document.querySelector(parent) : parent;
      if (p) p.insertBefore(typeof child === 'string' ? document.createTextNode(child) : child, p.firstChild);
    }},
    remove: function(sel) {{ D.$(sel).forEach(function(el) {{ el.remove(); }}); }},
    empty: function(sel) {{ D.$(sel).forEach(function(el) {{ el.innerHTML = ''; }}); }},
    text: function(sel, txt) {{
      if (txt === undefined) return D.$$(sel)?.textContent;
      D.$(sel).forEach(function(el) {{ el.textContent = txt; }});
    }},
    html: function(sel, htm) {{
      if (htm === undefined) return D.$$(sel)?.innerHTML;
      D.$(sel).forEach(function(el) {{ el.innerHTML = htm; }});
    }},
    attr: function(sel, name, value) {{
      if (value === undefined) return D.$$(sel)?.getAttribute(name);
      D.$(sel).forEach(function(el) {{ el.setAttribute(name, value); }});
    }},
    // ==================== STYLING ====================
    style: function(sel, styles) {{
      D.$(sel).forEach(function(el) {{ Object.assign(el.style, styles); }});
    }},
    css: function(sel, prop, value) {{
      if (value === undefined) return getComputedStyle(D.$$(sel))[prop];
      D.$(sel).forEach(function(el) {{ el.style[prop] = value; }});
    }},
    addClass: function(sel, cls) {{ D.$(sel).forEach(function(el) {{ el.classList.add(cls); }}); }},
    removeClass: function(sel, cls) {{ D.$(sel).forEach(function(el) {{ el.classList.remove(cls); }}); }},
    toggleClass: function(sel, cls) {{ D.$(sel).forEach(function(el) {{ el.classList.toggle(cls); }}); }},
    hasClass: function(sel, cls) {{ return D.$$(sel)?.classList.contains(cls); }},
    // ==================== HTTP UTILITIES ====================
    fetch: function(url, opts) {{ return fetch(url, opts).then(function(r) {{ return r.json(); }}); }},
    get: function(url, params) {{
      var query = params ? '?' + new URLSearchParams(params).toString() : '';
      return fetch(url + query).then(function(r) {{ return r.json(); }});
    }},
    post: function(url, data) {{
      return fetch(url, {{
        method: 'POST',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify(data)
      }}).then(function(r) {{ return r.json(); }});
    }},
    put: function(url, data) {{
      return fetch(url, {{
        method: 'PUT',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify(data)
      }}).then(function(r) {{ return r.json(); }});
    }},
    patch: function(url, data) {{
      return fetch(url, {{
        method: 'PATCH',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify(data)
      }}).then(function(r) {{ return r.json(); }});
    }},
    delete: function(url) {{
      return fetch(url, {{ method: 'DELETE' }}).then(function(r) {{ return r.json(); }});
    }},
    upload: function(url, file, fieldName) {{
      var fd = new FormData();
      fd.append(fieldName || 'file', file);
      return fetch(url, {{ method: 'POST', body: fd }}).then(function(r) {{ return r.json(); }});
    }},
    // ==================== STORAGE ====================
    save: function(key, value) {{
      try {{ localStorage.setItem('dew:' + key, JSON.stringify(value)); }} catch(e) {{}}
    }},
    load: function(key, defaultVal) {{
      try {{
        var v = localStorage.getItem('dew:' + key);
        return v ? JSON.parse(v) : defaultVal;
      }} catch(e) {{ return defaultVal; }}
    }},
    clear: function(key) {{
      if (key) localStorage.removeItem('dew:' + key);
      else Object.keys(localStorage).filter(function(k) {{ return k.startsWith('dew:'); }}).forEach(function(k) {{ localStorage.removeItem(k); }});
    }},
    session: {{
      save: function(key, value) {{ try {{ sessionStorage.setItem('dew:' + key, JSON.stringify(value)); }} catch(e) {{}} }},
      load: function(key, defaultVal) {{ try {{ var v = sessionStorage.getItem('dew:' + key); return v ? JSON.parse(v) : defaultVal; }} catch(e) {{ return defaultVal; }} }},
      clear: function(key) {{ if (key) sessionStorage.removeItem('dew:' + key); else sessionStorage.clear(); }}
    }},
    // ==================== CLIPBOARD ====================
    copy: function(text) {{
      return navigator.clipboard.writeText(text).then(function() {{ D.emit('copied', text); return true; }});
    }},
    paste: function() {{
      return navigator.clipboard.readText();
    }},
    // ==================== UTILITIES ====================
    debounce: function(fn, delay) {{
      var timer;
      return function() {{
        var args = arguments, ctx = this;
        clearTimeout(timer);
        timer = setTimeout(function() {{ fn.apply(ctx, args); }}, delay || 300);
      }};
    }},
    throttle: function(fn, limit) {{
      var inThrottle;
      return function() {{
        var args = arguments, ctx = this;
        if (!inThrottle) {{
          fn.apply(ctx, args);
          inThrottle = true;
          setTimeout(function() {{ inThrottle = false; }}, limit || 300);
        }}
      }};
    }},
    delay: function(ms) {{
      return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
    }},
    uuid: function() {{
      return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {{
        var r = Math.random() * 16 | 0;
        return (c === 'x' ? r : (r & 0x3 | 0x8)).toString(16);
      }});
    }},
    random: function(min, max) {{
      return Math.floor(Math.random() * (max - min + 1)) + min;
    }},
    clamp: function(val, min, max) {{
      return Math.min(Math.max(val, min), max);
    }},
    lerp: function(start, end, t) {{
      return start + (end - start) * t;
    }},
    // ==================== FORM HELPERS ====================
    serialize: function(form) {{
      var f = typeof form === 'string' ? document.querySelector(form) : form;
      return Object.fromEntries(new FormData(f));
    }},
    validate: function(form, rules) {{
      var data = D.serialize(form);
      var errors = {{}};
      Object.keys(rules).forEach(function(field) {{
        var rule = rules[field];
        var value = data[field] || '';
        if (rule.required && !value) errors[field] = 'Required';
        else if (rule.email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)) errors[field] = 'Invalid email';
        else if (rule.min && value.length < rule.min) errors[field] = 'Min ' + rule.min + ' chars';
        else if (rule.max && value.length > rule.max) errors[field] = 'Max ' + rule.max + ' chars';
        else if (rule.pattern && !new RegExp(rule.pattern).test(value)) errors[field] = rule.message || 'Invalid';
      }});
      return {{ valid: Object.keys(errors).length === 0, errors: errors, data: data }};
    }},
    // ==================== NOTIFICATIONS ====================
    notify: function(message, opts) {{
      opts = opts || {{}};
      var n = D.create('div', {{
        class: 'dew-notify',
        style: {{
          position: 'fixed',
          top: '20px',
          right: '20px',
          padding: '12px 20px',
          borderRadius: '8px',
          backgroundColor: opts.type === 'error' ? '#ef4444' : opts.type === 'success' ? '#22c55e' : opts.type === 'warning' ? '#f59e0b' : '#3b82f6',
          color: '#fff',
          boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
          zIndex: '9999',
          animation: 'dew-slide-left 0.3s ease'
        }}
      }}, message);
      document.body.appendChild(n);
      setTimeout(function() {{
        n.style.animation = 'dew-fade-out 0.3s ease forwards';
        setTimeout(function() {{ n.remove(); }}, 300);
      }}, opts.duration || 3000);
    }},
    // ==================== MODAL ====================
    modal: function(content, opts) {{
      opts = opts || {{}};
      var overlay = D.create('div', {{
        class: 'dew-modal-overlay',
        style: {{
          position: 'fixed', top: '0', left: '0', right: '0', bottom: '0',
          backgroundColor: 'rgba(0,0,0,0.5)', display: 'flex',
          alignItems: 'center', justifyContent: 'center', zIndex: '9998'
        }},
        onclick: function(e) {{ if (e.target === overlay && opts.closable !== false) overlay.remove(); }}
      }});
      var modal = D.create('div', {{
        class: 'dew-modal',
        style: {{
          backgroundColor: '#fff', borderRadius: '12px', padding: '24px',
          maxWidth: opts.width || '500px', width: '90%', maxHeight: '80vh',
          overflow: 'auto', animation: 'dew-scale-up 0.3s ease'
        }}
      }});
      if (typeof content === 'string') modal.innerHTML = content;
      else modal.appendChild(content);
      overlay.appendChild(modal);
      document.body.appendChild(overlay);
      return {{ close: function() {{ overlay.remove(); }}, el: modal }};
    }},
    // ==================== LOADING ====================
    loading: {{
      _el: null,
      show: function(text) {{
        if (D.loading._el) return;
        D.loading._el = D.create('div', {{
          class: 'dew-loading',
          style: {{
            position: 'fixed', top: '0', left: '0', right: '0', bottom: '0',
            backgroundColor: 'rgba(255,255,255,0.9)', display: 'flex',
            alignItems: 'center', justifyContent: 'center', flexDirection: 'column',
            zIndex: '9999'
          }}
        }});
        var spinner = D.create('div', {{
          style: {{
            width: '40px', height: '40px', border: '4px solid #e5e7eb',
            borderTopColor: '#3b82f6', borderRadius: '50%',
            animation: 'dew-spin 1s linear infinite'
          }}
        }});
        D.loading._el.appendChild(spinner);
        if (text) D.loading._el.appendChild(D.create('p', {{ style: {{ marginTop: '16px', color: '#6b7280' }} }}, text));
        document.body.appendChild(D.loading._el);
      }},
      hide: function() {{
        if (D.loading._el) {{ D.loading._el.remove(); D.loading._el = null; }}
      }}
    }}
  }};
  D._defaults = {{}};
  return D;
}})();
// Initialize on DOM ready
document.addEventListener('DOMContentLoaded', function() {{
  Dew.data = DATA_PLACEHOLDER;
  Dew.update();
  // Two-way binding for inputs
  document.querySelectorAll('input[dew-model], textarea[dew-model], select[dew-model]').forEach(function(el) {{
    var key = el.getAttribute('dew-model');
    el.addEventListener('input', function() {{ Dew.set(key, el.value); }});
  }});
  // Trigger on-load handlers
  document.querySelectorAll('[data-onload]').forEach(function(el) {
    try { eval(el.getAttribute('data-onload')); } catch(e) {}
  });
});
</script>
"#);
    // Insert the data into the runtime and fix escaped braces
    dew_runtime = dew_runtime
        .replace("{{", "{")
        .replace("}}", "}")
        .replace("DATA_PLACEHOLDER", &js_data);
    template.replace("</body>", &format!("{}</body>", dew_runtime))
}
/// Convert Value to JavaScript literal
fn value_to_js(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        Value::Boolean(b) => b.to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(value_to_js).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Table(t) => {
            let pairs: Vec<String> = t.iter()
                .filter(|(k, _)| !k.starts_with("__"))
                .map(|(k, v)| format!("{}: {}", k, value_to_js(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        Value::Empty => "null".to_string(),
        _ => "null".to_string(),
    }
}
fn process_dew_code_blocks(template: &str, data: &HashMap<String, Value>) -> String {
    let mut result = template.to_string();
    let dew_pattern = "class=\"dew= '";
    while let Some(start) = result.find(dew_pattern) {
        let code_start = start + dew_pattern.len();
        if let Some(code_end_offset) = result[code_start..].find("'\"") {
            let code_end = code_start + code_end_offset;
            let code = &result[code_start..code_end];
            let tag_start = result[..start].rfind('<').unwrap_or(start);
            if let Some(tag_close_offset) = result[code_end..].find('>') {
                let tag_close = code_end + tag_close_offset + 1;
                let tag_content = &result[tag_start + 1..start];
                let tag_name = tag_content.split_whitespace().next().unwrap_or("div");
                let closing_tag = format!("</{}>", tag_name);
                if let Some(closing_offset) = result[tag_close..].find(&closing_tag) {
                    let closing_end = tag_close + closing_offset + closing_tag.len();
                    let evaluated = evaluate_dew_code(code, data);
                    result = format!("{}{}{}", &result[..tag_start], evaluated, &result[closing_end..]);
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    result
}
fn evaluate_dew_code(code: &str, data: &HashMap<String, Value>) -> String {
    let code = code.trim();
    if let Some(value) = data.get(code) {
        return value_to_string(value);
    }
    if let Some(dot_pos) = code.find('.') {
        let obj_name = &code[..dot_pos];
        let prop_name = &code[dot_pos + 1..];
        if let Some(Value::Table(obj)) = data.get(obj_name) {
            if let Some(value) = obj.get(prop_name) {
                return value_to_string(value);
            }
        }
    }
    if code.contains('+') {
        let parts: Vec<&str> = code.split('+').collect();
        let mut result = String::new();
        for part in parts {
            let part = part.trim();
            if part.starts_with('"') && part.ends_with('"') {
                result.push_str(&part[1..part.len()-1]);
            } else if let Some(value) = data.get(part) {
                result.push_str(&value_to_string(value));
            }
        }
        return result;
    }
    if code.contains('?') && code.contains(':') {
        if let Some(q_pos) = code.find('?') {
            let condition = code[..q_pos].trim();
            let rest = &code[q_pos + 1..];
            if let Some(c_pos) = rest.find(':') {
                let true_val = rest[..c_pos].trim();
                let false_val = rest[c_pos + 1..].trim();
                let cond_result = evaluate_condition(condition, data);
                let result_val = if cond_result { true_val } else { false_val };
                if result_val.starts_with('"') && result_val.ends_with('"') {
                    return result_val[1..result_val.len()-1].to_string();
                } else if let Some(value) = data.get(result_val) {
                    return value_to_string(value);
                }
                return result_val.to_string();
            }
        }
    }
    String::new()
}
fn evaluate_template_expr(code: &str, data: &HashMap<String, Value>) -> String {
    let code = code.trim();
    if let Some(value) = data.get(code) {
        return value_to_string(value);
    }
    if let Some(dot_pos) = code.find('.') {
        let obj_name = &code[..dot_pos];
        let prop_name = &code[dot_pos + 1..];
        if let Some(Value::Table(obj)) = data.get(obj_name) {
            if let Some(value) = obj.get(prop_name) {
                return value_to_string(value);
            }
        }
    }
    String::new()
}
fn process_template_control_flow(template: &str, data: &HashMap<String, Value>) -> String {
    let mut result = template.to_string();
    while let Some(for_start) = result.find("?( for ") {
        if let Some(for_end) = result[for_start..].find(")?") {
            let loop_header_end = for_start + for_end + 2;
            let loop_header = &result[for_start + 7..for_start + for_end].trim();
            let parts: Vec<&str> = loop_header.split(" in ").collect();
            if parts.len() == 2 {
                let item_var = parts[0].trim();
                let items_var = parts[1].trim();
                if let Some(endfor_pos) = result[loop_header_end..].find("?( endfor )?") {
                    let endfor_pos = loop_header_end + endfor_pos;
                    let loop_body = &result[loop_header_end..endfor_pos];
                    let mut loop_output = String::new();
                    if let Some(Value::Array(items)) = data.get(items_var) {
                        for item in items {
                            let mut item_data = data.clone();
                            item_data.insert(item_var.to_string(), item.clone());
                            let rendered_body = render_template(loop_body, &item_data);
                            loop_output.push_str(&rendered_body);
                        }
                    }
                    result = format!("{}{}{}", &result[..for_start], loop_output, &result[endfor_pos + 12..]);
                }
            }
        } else {
            break;
        }
    }
    while let Some(if_start) = result.find("?( if ") {
        if let Some(if_end) = result[if_start..].find(")?") {
            let cond_end = if_start + if_end + 2;
            let condition = &result[if_start + 6..if_start + if_end].trim();
            if let Some(endif_pos) = result[cond_end..].find("?( endif )?") {
                let endif_pos = cond_end + endif_pos;
                let if_body = &result[cond_end..endif_pos];
                let show_content = evaluate_condition(condition, data);
                let output = if show_content { if_body.to_string() } else { String::new() };
                result = format!("{}{}{}", &result[..if_start], output, &result[endif_pos + 11..]);
            }
        } else {
            break;
        }
    }
    result
}
fn evaluate_condition(condition: &str, data: &HashMap<String, Value>) -> bool {
    let condition = condition.trim();
    if let Some(value) = data.get(condition) {
        return match value {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Table(t) => !t.is_empty(),
            Value::Empty => false,
            _ => true,
        };
    }
    if let Some(eq_pos) = condition.find("==") {
        let left = condition[..eq_pos].trim();
        let right = condition[eq_pos + 2..].trim();
        let left_val = data.get(left).map(value_to_string).unwrap_or_default();
        let right_val = right.trim_matches('"').to_string();
        return left_val == right_val;
    }
    false
}
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        Value::Boolean(b) => b.to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(value_to_string).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Table(t) => value_to_json(&Value::Table(t.clone())),
        Value::Empty => String::new(),
        _ => format!("{:?}", value),
    }
}
fn value_to_json(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        Value::Boolean(b) => b.to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(value_to_json).collect();
            format!("[{}]", items.join(","))
        }
        Value::Table(t) => {
            let pairs: Vec<String> = t.iter()
                .filter(|(k, _)| !k.starts_with("__"))
                .map(|(k, v)| format!("\"{}\":{}", k, value_to_json(v)))
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
        Value::Empty => "null".to_string(),
        _ => "null".to_string(),
    }
}
fn parse_json_to_value(json: &str) -> Result<Value, String> {
    let json = json.trim();
    if json.is_empty() {
        return Ok(Value::Empty);
    }
    if json == "null" {
        return Ok(Value::Empty);
    }
    if json == "true" {
        return Ok(Value::Boolean(true));
    }
    if json == "false" {
        return Ok(Value::Boolean(false));
    }
    if let Ok(n) = json.parse::<f64>() {
        return Ok(Value::Number(n));
    }
    if json.starts_with('"') && json.ends_with('"') {
        let s = &json[1..json.len() - 1];
        return Ok(Value::String(s.replace("\\\"", "\"").replace("\\\\", "\\")));
    }
    if json.starts_with('[') && json.ends_with(']') {
        let inner = &json[1..json.len() - 1];
        let mut items = Vec::new();
        let mut depth = 0;
        let mut start = 0;
        for (i, c) in inner.char_indices() {
            match c {
                '[' | '{' => depth += 1,
                ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    let item = inner[start..i].trim();
                    if !item.is_empty() {
                        items.push(parse_json_to_value(item)?);
                    }
                    start = i + 1;
                }
                _ => {}
            }
        }
        let last = inner[start..].trim();
        if !last.is_empty() {
            items.push(parse_json_to_value(last)?);
        }
        return Ok(Value::Array(items));
    }
    if json.starts_with('{') && json.ends_with('}') {
        let inner = &json[1..json.len() - 1];
        let mut map = HashMap::new();
        let mut depth = 0;
        let mut start = 0;
        for (i, c) in inner.char_indices() {
            match c {
                '[' | '{' => depth += 1,
                ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    parse_json_pair(&inner[start..i], &mut map)?;
                    start = i + 1;
                }
                _ => {}
            }
        }
        let last = inner[start..].trim();
        if !last.is_empty() {
            parse_json_pair(last, &mut map)?;
        }
        return Ok(Value::Table(map));
    }
    Ok(Value::String(json.to_string()))
}
fn parse_json_pair(pair: &str, map: &mut HashMap<String, Value>) -> Result<(), String> {
    let pair = pair.trim();
    if pair.is_empty() {
        return Ok(());
    }
    if let Some(colon_pos) = pair.find(':') {
        let key = pair[..colon_pos].trim().trim_matches('"');
        let value = pair[colon_pos + 1..].trim();
        map.insert(key.to_string(), parse_json_to_value(value)?);
    }
    Ok(())
}
fn parse_form_data(body: &str) -> HashMap<String, Value> {
    let mut data = HashMap::new();
    for pair in body.split('&') {
        if let Some(eq_pos) = pair.find('=') {
            let key = url_decode(&pair[..eq_pos]);
            let value = url_decode(&pair[eq_pos + 1..]);
            data.insert(key, Value::String(value));
        }
    }
    data
}
fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}
lazy_static::lazy_static! {
    static ref RATE_LIMIT_STORE: Mutex<HashMap<String, Vec<u64>>> = Mutex::new(HashMap::new());
}
fn check_rate_limit(client_ip: &str, config: &RateLimitConfig) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let window_start = now - config.window_seconds as u64;
    let mut store = RATE_LIMIT_STORE.lock().unwrap();
    let timestamps = store.entry(client_ip.to_string()).or_insert_with(Vec::new);
    timestamps.retain(|&ts| ts > window_start);
    if timestamps.len() < config.requests_per_window as usize {
        timestamps.push(now);
        true
    } else {
        false
    }
}
#[allow(dead_code)]
fn cleanup_rate_limits() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut store = RATE_LIMIT_STORE.lock().unwrap();
    store.retain(|_, timestamps| {
        timestamps.retain(|&ts| ts > now - 3600); 
        !timestamps.is_empty()
    });
}
fn validate_field(value: Option<&Value>, rule: &str) -> Option<String> {
    let rule = rule.trim();
    let (rule_name, rule_param) = if let Some(colon_pos) = rule.find(':') {
        (&rule[..colon_pos], Some(&rule[colon_pos + 1..]))
    } else {
        (rule, None)
    };
    match rule_name {
        "required" => {
            match value {
                None | Some(Value::Empty) => Some("This field is required".to_string()),
                Some(Value::String(s)) if s.is_empty() => Some("This field is required".to_string()),
                _ => None,
            }
        }
        "email" => {
            if let Some(Value::String(s)) = value {
                if !s.contains('@') || !s.contains('.') {
                    return Some("Invalid email format".to_string());
                }
            }
            None
        }
        "min" => {
            if let Some(param) = rule_param {
                if let Ok(min_len) = param.parse::<usize>() {
                    if let Some(Value::String(s)) = value {
                        if s.len() < min_len {
                            return Some(format!("Must be at least {} characters", min_len));
                        }
                    }
                }
            }
            None
        }
        "max" => {
            if let Some(param) = rule_param {
                if let Ok(max_len) = param.parse::<usize>() {
                    if let Some(Value::String(s)) = value {
                        if s.len() > max_len {
                            return Some(format!("Must be at most {} characters", max_len));
                        }
                    }
                }
            }
            None
        }
        "numeric" => {
            if let Some(Value::String(s)) = value {
                if s.parse::<f64>().is_err() {
                    return Some("Must be a number".to_string());
                }
            }
            None
        }
        "alpha" => {
            if let Some(Value::String(s)) = value {
                if !s.chars().all(|c| c.is_alphabetic()) {
                    return Some("Must contain only letters".to_string());
                }
            }
            None
        }
        "alphanumeric" => {
            if let Some(Value::String(s)) = value {
                if !s.chars().all(|c| c.is_alphanumeric()) {
                    return Some("Must contain only letters and numbers".to_string());
                }
            }
            None
        }
        "url" => {
            if let Some(Value::String(s)) = value {
                if !s.starts_with("http://") && !s.starts_with("https://") {
                    return Some("Invalid URL format".to_string());
                }
            }
            None
        }
        "regex" => {
            None
        }
        _ => None,
    }
}
fn get_mime_type(path: &str) -> String {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext.to_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "webp" => "image/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "eot" => "application/vnd.ms-fontobject",
        "pdf" => "application/pdf",
        "xml" => "application/xml",
        "txt" => "text/plain; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "tar" => "application/x-tar",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        _ => "application/octet-stream",
    }.to_string()
}
fn parse_duration_string(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    if s.ends_with("ms") {
        s[..s.len()-2].trim().parse().ok()
    } else if s.ends_with('s') {
        s[..s.len()-1].trim().parse::<u64>().ok().map(|v| v * 1000)
    } else if s.ends_with('m') {
        s[..s.len()-1].trim().parse::<u64>().ok().map(|v| v * 60 * 1000)
    } else if s.ends_with('h') {
        s[..s.len()-1].trim().parse::<u64>().ok().map(|v| v * 60 * 60 * 1000)
    } else {
        s.parse().ok()
    }
}
fn generate_job_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("job_{:x}", timestamp)
}
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn generate_websocket_accept_key(key: &str) -> String {
    use sha1::{Digest, Sha1};
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    BASE64.encode(hasher.finalize())
}

fn generate_csrf_token() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

fn sanitize_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}
fn sanitize_sql(input: &str) -> String {
    input
        .replace('\'', "''")
        .replace('\\', "\\\\")
        .replace('\0', "")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\x1a', "\\Z")
}
fn sanitize_js(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('<', "\\x3c")
        .replace('>', "\\x3e")
}
fn sanitize_url(input: &str) -> String {
    let mut result = String::new();
    for c in input.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            _ => result.push_str(&format!("%{:02X}", c as u32)),
        }
    }
    result
}

fn start_server(server: &DewServer, port: u16, host: &str) -> MintasResult<Value> {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).map_err(|e| MintasError::RuntimeError {
        message: format!("Failed to bind to {}: {}", addr, e),
        location: SourceLocation::new(0, 0),
    })?;
    println!("\n🌿 Dew server running at http://{}", addr);
    println!("   Press Ctrl+C to stop\n");
    println!("   Routes:");
    for route in &server.routes {
        println!("     {} {}", route.method.as_str(), route.path);
    }
    if !server.static_dirs.is_empty() {
        println!("   Static:");
        for (url, dir) in &server.static_dirs {
            println!("     {} -> {}", url, dir);
        }
    }
    println!();
    let server = server.clone();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                stream.set_read_timeout(Some(std::time::Duration::from_secs(30))).ok();
                let mut buffer = vec![0u8; 65536];
                if let Ok(size) = stream.read(&mut buffer) {
                    if size > 0 {
                        let request_str = String::from_utf8_lossy(&buffer[..size]);
                        let (response, log_line) = handle_request(&request_str, &server);
                        println!("{}", log_line);
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    }
                }
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }
    }
    Ok(Value::Empty)
}

fn handle_request(request_str: &str, server: &DewServer) -> (String, String) {
    let start_time = std::time::Instant::now();
    let mut lines = request_str.lines();
    let first_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return (http_response(400, "text/plain", "Bad Request", &[]), 
                "400 Bad Request".to_string());
    }
    let method = parts[0];
    let full_path = parts[1];
    let path = full_path.split('?').next().unwrap_or("/");
    if method == "OPTIONS" {
        let cors_headers = vec![
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS"),
            ("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Requested-With"),
            ("Access-Control-Max-Age", "86400"),
        ];
        return (http_response_with_headers(204, "text/plain", "", &cors_headers),
                format!("OPTIONS {} 204 (CORS preflight)", path));
    }
    // WebSocket Upgrade
    if method == "GET" {
        let mut is_websocket = false;
        let mut sec_ws_key = String::new();
        for line in request_str.lines() {
             if line.to_lowercase().starts_with("upgrade: websocket") { is_websocket = true; }
             if line.to_lowercase().starts_with("sec-websocket-key:") {
                 sec_ws_key = line.split(':').nth(1).unwrap_or("").trim().to_string();
             }
        }
        if is_websocket && !sec_ws_key.is_empty() {
             let accept_key = generate_websocket_accept_key(&sec_ws_key);
             let response = format!(
                 "HTTP/1.1 101 Switching Protocols\r\n\
                 Upgrade: websocket\r\n\
                 Connection: Upgrade\r\n\
                 Sec-WebSocket-Accept: {}\r\n\r\n", accept_key);
             return (response, format!("WEBSOCKET {} 101 (Upgraded)", path));
        }
    }
    if method == "GET" {
        if let Some(file_path) = server.find_static_file(path) {
            if let Ok(content) = fs::read(&file_path) {
                let content_type = get_mime_type(&file_path);
                let elapsed = start_time.elapsed().as_micros();
                return (http_response_binary(200, &content_type, &content),
                        format!("{} {} 200 (static) {}µs", method, path, elapsed));
            }
        }
    }
    let query: HashMap<String, String> = if let Some(qs) = full_path.split('?').nth(1) {
        qs.split('&')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                Some((url_decode(parts.next()?), url_decode(parts.next().unwrap_or(""))))
            })
            .collect()
    } else {
        HashMap::new()
    };
    if let Some(rate_limit) = &server.rate_limit {
        let client_ip = "127.0.0.1"; 
        if !check_rate_limit(client_ip, rate_limit) {
            let elapsed = start_time.elapsed().as_micros();
            return (http_response(429, "application/json", 
                r#"{"error":"Too Many Requests","message":"Rate limit exceeded"}"#, &[]),
                format!("{} {} 429 (rate limited) {}µs", method, path, elapsed));
        }
    }
    if server.security.sql_injection_protection {
        let suspicious_patterns = ["'--", "'; DROP", "1=1", "OR 1=1", "UNION SELECT"];
        let full_input = format!("{} {}", full_path, request_str);
        for pattern in suspicious_patterns {
            if full_input.to_uppercase().contains(pattern) {
                let elapsed = start_time.elapsed().as_micros();
                return (http_response(400, "application/json",
                    r#"{"error":"Bad Request","message":"Potentially malicious input detected"}"#, &[]),
                    format!("{} {} 400 (security) {}µs", method, path, elapsed));
            }
        }
    }
    let mut headers = HashMap::new();
    let mut cookies = HashMap::new();
    let mut body_started = false;
    let mut body_lines = Vec::new();
    let mut content_length = 0usize;
    for line in lines {
        if line.is_empty() {
            body_started = true;
            continue;
        }
        if body_started {
            body_lines.push(line);
        } else if let Some(idx) = line.find(':') {
            let key = line[..idx].trim().to_lowercase();
            let value = line[idx + 1..].trim().to_string();
            if key == "content-length" {
                content_length = value.parse().unwrap_or(0);
            }
            if key == "cookie" {
                for cookie in value.split(';') {
                    if let Some(eq_pos) = cookie.find('=') {
                        let name = cookie[..eq_pos].trim().to_string();
                        let val = cookie[eq_pos + 1..].trim().to_string();
                        cookies.insert(name, val);
                    }
                }
            }
            headers.insert(key, value);
        }
    }
    let body = body_lines.join("\n");
    let _ = content_length; 
    if let Some((route, params)) = server.find_route(method, path) {
        let mut getback = Getback::new();
        getback.method = method.to_string();
        getback.path = path.to_string();
        getback.url = full_path.to_string();
        getback.headers = headers;
        getback.query = query;
        getback.params = params;
        getback.body = body;
        getback.cookies = cookies;
        for before_handler in &server.before_handlers {
            match execute_handler(before_handler, getback.clone()) {
                Ok(response) => {
                    // If middleware returns a response, STOP processing and return it
                    if extract_status_from_response(&response) != 200 {
                        let elapsed = start_time.elapsed().as_micros();
                         let status = extract_status_from_response(&response);
                        return (response, format!("{} {} {} (middleware) {}µs", method, path, status, elapsed));
                    }
                }
                Err(e) => {
                     let elapsed = start_time.elapsed().as_micros();
                     return (http_response(500, "text/plain", &format!("{}", e), &[]), 
                             format!("{} {} 500 (middleware error) {}µs", method, path, elapsed));
                }
            }
        }
        let response = match execute_handler(&route.handler.handler_body, getback.clone()) {
            Ok(res) => res,
            Err(e) => http_response(500, "text/html", &format!("<h1>Error</h1><p>{}</p>", e), &[])
        };
        for after_handler in &server.after_handlers {
            // After handlers run but their return value is currently ignored 
            // In a real framework they might modify the response
            let _ = execute_handler(after_handler, getback.clone());
        }
        let elapsed = start_time.elapsed().as_micros();
        let status = extract_status_from_response(&response);
        (response, format!("{} {} {} {}µs", method, path, status, elapsed))
    } else {
        if let Some(error_handler) = server.error_handlers.get(&404) {
            let mut getback = Getback::new();
            getback.method = method.to_string();
            getback.path = path.to_string();
            let response = execute_handler(&error_handler.handler_body, getback).unwrap_or_else(|e| {
                http_response(500, "text/plain", &format!("Error in error handler: {}", e), &[])
            });
            let elapsed = start_time.elapsed().as_micros();
            return (response, format!("{} {} 404 (custom) {}µs", method, path, elapsed));
        }
        let elapsed = start_time.elapsed().as_micros();
        (http_response(404, "text/html", &format!(
            "<!DOCTYPE html><html><head><title>404 Not Found</title></head>\
            <body style=\"font-family:system-ui;text-align:center;padding:50px\">\
            <h1>404</h1><p>Page not found: {}</p>\
            <p style=\"color:#666\">🌿 Dew</p></body></html>", path
        ), &[]), format!("{} {} 404 {}µs", method, path, elapsed))
    }
}

fn extract_status_from_response(response: &str) -> u16 {
    if let Some(status_line) = response.lines().next() {
        if let Some(status_str) = status_line.split_whitespace().nth(1) {
            return status_str.parse().unwrap_or(200);
        }
    }
    200
}


fn execute_handler(handler_body: &[crate::parser::Expr], getback: Getback) -> MintasResult<String> {
    use crate::evaluator::Evaluator;
    let mut evaluator = Evaluator::new();
    evaluator.set_getback(getback.to_value());
    let mut response_cookies: Vec<String> = Vec::new();
    for stmt in handler_body {
        match evaluator.eval(stmt) {
            Ok(Value::ReturnSignal(boxed_val)) => {
                return Ok(process_return_value(&*boxed_val, &response_cookies));
            }
            Ok(val) => {
                if let Value::Table(ref map) = val {
                    if map.get("__type__").map(|v| matches!(v, Value::String(s) if s == "DewResponse")).unwrap_or(false) {
                        return Ok(process_return_value(&val, &response_cookies));
                    }
                    if map.get("__type__").map(|v| matches!(v, Value::String(s) if s == "SetCookie")).unwrap_or(false) {
                        if let (Some(Value::String(name)), Some(Value::String(value))) = 
                            (map.get("name"), map.get("value")) {
                            let max_age = match map.get("max_age") {
                                Some(Value::Number(n)) => *n as u64,
                                _ => 3600,
                            };
                            let path = match map.get("path") {
                                Some(Value::String(p)) => p.clone(),
                                _ => "/".to_string(),
                            };
                            response_cookies.push(format!(
                                "{}={}; Max-Age={}; Path={}; HttpOnly",
                                name, value, max_age, path
                            ));
                        }
                    }
                }
            }
            Err(e) => return Err(MintasError::RuntimeError { message: format!("{}",e), location: SourceLocation::new(0,0) }),
        }
    }
    Ok(http_response(200, "text/plain", "", &response_cookies))
}

fn process_return_value(value: &Value, cookies: &[String]) -> String {
    if let Value::Table(ref map) = value {
        if map.get("__type__").map(|v| matches!(v, Value::String(s) if s == "DewResponse")).unwrap_or(false) {
            let response_type = match map.get("response_type") {
                Some(Value::String(s)) => s.as_str(),
                _ => "text",
            };
            let body = match map.get("body") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            let status = match map.get("status") {
                Some(Value::Number(n)) => *n as u16,
                _ => 200,
            };
            if response_type == "redirect" {
                let location = match map.get("location") {
                    Some(Value::String(s)) => s.clone(),
                    _ => "/".to_string(),
                };
                return http_response_with_headers(status, "text/plain", "", &[
                    ("Location", &location),
                ]);
            }
            let content_type = match response_type {
                "json" => "application/json; charset=utf-8",
                "html" => "text/html; charset=utf-8",
                "file" => match map.get("content_type") {
                    Some(Value::String(ct)) => ct.as_str(),
                    _ => "application/octet-stream",
                },
                _ => "text/plain; charset=utf-8",
            };
            return http_response(status, content_type, &body, cookies);
        }
    }
    let body = value_to_json(value);
    http_response(200, "application/json; charset=utf-8", &body, cookies)
}

fn http_response(status: u16, content_type: &str, body: &str, cookies: &[String]) -> String {
    let status_text = match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Unknown",
    };
    let mut headers = format!(
        "HTTP/1.1 {} {}\r\n\
        Content-Type: {}\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\
        Access-Control-Allow-Origin: *\r\n",
        status, status_text, content_type, body.len()
    );
    for cookie in cookies {
        headers.push_str(&format!("Set-Cookie: {}\r\n", cookie));
    }
    headers.push_str("\r\n");
    headers.push_str(body);
    headers
}

fn http_response_with_headers(status: u16, content_type: &str, body: &str, extra_headers: &[(&str, &str)]) -> String {
    let status_text = match status {
        200 => "OK", 201 => "Created", 204 => "No Content",
        301 => "Moved Permanently", 302 => "Found", 304 => "Not Modified",
        400 => "Bad Request", 401 => "Unauthorized", 403 => "Forbidden",
        404 => "Not Found", 500 => "Internal Server Error",
        _ => "Unknown",
    };
    let mut response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        status, status_text, content_type, body.len()
    );
    for (key, value) in extra_headers {
        response.push_str(&format!("{}: {}\r\n", key, value));
    }
    response.push_str("\r\n");
    response.push_str(body);
    response
}

fn http_response_binary(status: u16, content_type: &str, body: &[u8]) -> String {
    let status_text = match status { 200 => "OK", 404 => "Not Found", _ => "Unknown" };
    let headers = format!(
        "HTTP/1.1 {} {}\r\n\
        Content-Type: {}\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\
        Cache-Control: public, max-age=31536000\r\n\r\n",
        status, status_text, content_type, body.len()
    );
    format!("{}{}", headers, String::from_utf8_lossy(body))
}
