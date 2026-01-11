use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::parser::{BinaryOp, ClassMember, Expr, UnaryOp};
use std::collections::HashMap;
use std::io::{self, Write, BufWriter, BufRead, BufReader};
use std::sync::Arc;
use std::cell::RefCell;
use std::time::{Instant, Duration};
const MAX_RECURSION_DEPTH: usize = 1000;  
const MAX_MEMORY_ALLOCATION: usize = 100_000_000; 
#[allow(dead_code)]
const MAX_ARRAY_SIZE: usize = 1_000_000; 
#[allow(dead_code)]
const MAX_STRING_LENGTH: usize = 10_000_000; 
#[allow(dead_code)]
const MAX_EXECUTION_TIME_MS: u64 = 30_000; 
#[allow(dead_code)]
const MAX_LOOP_ITERATIONS: usize = 10_000_000; 
#[allow(dead_code)]
const MAX_STACK_FRAMES: usize = 1000; 
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SecurityMonitor {
    recursion_depth: usize,
    memory_allocated: usize,
    execution_start: Instant,
    loop_iterations: usize,
    stack_frames: usize,
    security_violations: Vec<String>,
}
#[allow(unused_imports)]
#[path = "../lib/math/mod.rs"]
mod math_module;
#[allow(unused_imports)]
#[cfg(feature = "datetime")]
#[path = "../lib/datetime/mod.rs"]
mod datetime_module;
#[allow(unused_imports)]
#[cfg(feature = "json")]
#[path = "../lib/json/mod.rs"]
mod json_module;
#[allow(unused_imports)]
#[cfg(feature = "networking")]
#[path = "../lib/requests/mod.rs"]
mod requests_module;
#[allow(unused_imports)]
#[cfg(feature = "networking")]
#[path = "../lib/sockets/mod.rs"]
mod sockets_module;
#[allow(unused_imports)]
#[cfg(feature = "ai")]
#[path = "../lib/openai/mod.rs"]
mod openai_module;
#[allow(unused_imports)]
#[cfg(feature = "database")]
#[path = "../lib/sqlite3/mod.rs"]
mod sqlite3_module;
#[allow(unused_imports)]
#[cfg(feature = "database")]
#[path = "../lib/redis2/mod.rs"]
mod redis2_module;
#[allow(unused_imports)]
#[cfg(feature = "database")]
#[path = "../lib/postsql/mod.rs"]
mod postsql_module;
#[allow(unused_imports)]
#[path = "../lib/dew/mod.rs"]
mod dew_module;
#[allow(unused_imports)]
#[path = "../lib/dns/mod.rs"]
mod dns_module;
#[allow(unused_imports)]
#[path = "../lib/ping/mod.rs"]
mod ping_module;
#[allow(unused_imports)]
#[cfg(feature = "smtp")]
#[path = "../lib/smtp/mod.rs"]
mod smtp_module;
#[allow(unused_imports)]
#[cfg(feature = "ftp")]
#[path = "../lib/ftp/mod.rs"]
mod ftp_module;
#[allow(unused_imports)]
#[cfg(feature = "ssh")]
#[path = "../lib/ssh/mod.rs"]
mod ssh_module;
#[allow(unused_imports)]
#[path = "../lib/os/mod.rs"]
mod os_module;
#[allow(unused_imports)]
#[path = "../lib/env/mod.rs"]
mod env_module;
#[allow(unused_imports)]
#[path = "../lib/path/mod.rs"]
mod path_module;
#[allow(unused_imports)]
#[path = "../lib/sysfiles/mod.rs"]
mod sysfiles_module;
#[allow(unused_imports)]
#[path = "../lib/subprocess/mod.rs"]
mod subprocess_module;
#[allow(unused_imports)]
#[path = "../lib/base64/mod.rs"]
mod base64_module;
#[allow(unused_imports)]
#[path = "../lib/uuid/mod.rs"]
mod uuid_module;
#[allow(unused_imports)]
#[path = "../lib/hash/mod.rs"]
mod hash_module;
#[allow(unused_imports)]
#[path = "../lib/csv/mod.rs"]
mod csv_module;
#[allow(unused_imports)]
#[path = "../lib/colors/mod.rs"]
mod colors_module;
#[allow(unused_imports)]
#[path = "../lib/timer/mod.rs"]
mod timer_module;
#[allow(unused_imports)]
#[path = "../lib/slug/mod.rs"]
mod slug_module;
#[allow(unused_imports)]
#[path = "../lib/validate/mod.rs"]
mod validate_module;
#[allow(unused_imports)]
#[path = "../lib/cache/mod.rs"]
mod cache_module;
#[allow(unused_imports)]
#[path = "../lib/webhook/mod.rs"]
mod webhook_module;
#[allow(unused_imports)]
#[path = "../lib/cron/mod.rs"]
mod cron_module;
#[allow(unused_imports)]
#[path = "../lib/worker/mod.rs"]
mod worker_module;
#[allow(unused_imports)]
#[path = "../lib/cluster/mod.rs"]
mod cluster_module;
#[allow(unused_imports)]
#[path = "../lib/algorithm/mod.rs"]
mod algorithm_module;
#[allow(unused_imports)]
#[path = "../lib/asjokes/mod.rs"]
mod asjokes_module;
#[allow(unused_imports)]
#[path = "../lib/canvas/mod.rs"]
mod canvas_module;
#[allow(unused_imports)]
#[path = "../lib/crypto/mod.rs"]
mod crypto_module;
#[allow(unused_imports)]
#[path = "../lib/compress/mod.rs"]
mod compress_module;
#[allow(unused_imports)]
#[path = "../lib/queue/mod.rs"]
mod queue_module;
#[allow(unused_imports)]
#[path = "../lib/events/mod.rs"]
mod events_module;
#[allow(unused_imports)]
#[path = "../lib/buffer/mod.rs"]
mod buffer_module;
#[allow(unused_imports)]
#[path = "../lib/myqr/mod.rs"]
mod myqr_module;
#[allow(unused_imports)]
#[path = "../lib/mypdf/mod.rs"]
mod mypdf_module;
#[allow(unused_imports)]
#[path = "../lib/myyaml/mod.rs"]
mod myyaml_module;
#[allow(unused_imports)]
#[path = "../lib/archive/mod.rs"]
mod archive_module;
#[allow(unused_imports)]
#[path = "../lib/cert/mod.rs"]
mod cert_module;
#[allow(unused_imports)]
#[path = "../lib/graphql/mod.rs"]
mod graphql_module;
#[allow(unused_imports)]
#[path = "../lib/mqtt/mod.rs"]
mod mqtt_module;
#[allow(unused_imports)]
#[path = "../lib/mycli/mod.rs"]
mod mycli_module;
#[allow(unused_imports)]
#[path = "../lib/xdbx/mod.rs"]
mod xdbx_module;
#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<String>,
    pub body: Vec<Expr>,
    #[allow(dead_code)]
    pub is_lambda: bool,
}
#[derive(Debug, Clone)]
pub enum ClassInheritance {
    None,
    Extends(String),
}
#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub members: Vec<ClassMember>,
    pub inheritance: ClassInheritance,
}
#[derive(Debug, Clone)]
pub struct Instance {
    pub class_name: String,
    pub fields: HashMap<String, Value>,
    pub methods: HashMap<String, Function>,
}
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Maybe,
    Empty,
    Array(Vec<Value>),
    Table(std::collections::HashMap<String, Value>),
    SuperSet(Box<Value>), 
    Function(Box<Function>),
    Class(Box<Class>),
    Instance(Box<Instance>),
    ExitSignal,
    ProceedSignal,
    ReturnSignal(Box<Value>),
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Maybe, Value::Maybe) => true,
            (Value::Empty, Value::Empty) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Table(a), Value::Table(b)) => a == b,
            (Value::SuperSet(a), Value::SuperSet(b)) => a == b,
            (Value::Function(_), Value::Function(_)) => false,
            (Value::Class(_), Value::Class(_)) => false,
            (Value::Instance(a), Value::Instance(b)) => std::ptr::eq(a.as_ref(), b.as_ref()),
            (Value::ExitSignal, Value::ExitSignal) => true,
            (Value::ProceedSignal, Value::ProceedSignal) => true,
            (Value::ReturnSignal(a), Value::ReturnSignal(b)) => a == b,
            _ => false,
        }
    }
}
impl Value {
    #[inline]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Maybe => "boolean",
            Value::Empty => "empty",
            Value::Array(_) => "array",
            Value::Table(_) => "table",
            Value::SuperSet(_) => "superset",
            Value::Function(_) => "function",
            Value::Class(_) => "class",
            Value::Instance(_) => "instance",
            Value::ExitSignal => "exit",
            Value::ProceedSignal => "proceed",
            Value::ReturnSignal(_) => "return",
        }
    }
    #[inline]
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Maybe => false,
            Value::Empty => false,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Table(map) => !map.is_empty(),
            Value::SuperSet(val) => val.is_truthy(),
            Value::Function(_) | Value::Class(_) | Value::Instance(_) => true,
            Value::ExitSignal | Value::ProceedSignal | Value::ReturnSignal(_) => false,
        }
    }
    pub fn is_truthy_in_condition(&self) -> Value {
        match self {
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Maybe => Value::Maybe,
            Value::Empty => Value::Maybe, // Empty in conditions becomes Maybe
            Value::Number(n) => Value::Boolean(*n != 0.0),
            Value::String(s) => Value::Boolean(!s.is_empty()),
            Value::Array(arr) => Value::Boolean(!arr.is_empty()),
            Value::Table(map) => Value::Boolean(!map.is_empty()),
            Value::SuperSet(val) => val.is_truthy_in_condition(),
            Value::Function(_) | Value::Class(_) | Value::Instance(_) => Value::Boolean(true),
            Value::ExitSignal | Value::ProceedSignal | Value::ReturnSignal(_) => Value::Boolean(false),
        }
    }
}
#[derive(Clone)]
pub struct Evaluator {
    variables: HashMap<String, Value>,
    constants: std::collections::HashSet<String>,
    functions: HashMap<String, Function>,
    classes: HashMap<String, Class>,
    this_instance: Option<Box<Instance>>,
    // High-performance I/O buffers
    stdout_buffer: Arc<RefCell<BufWriter<io::Stdout>>>,
    stdin_buffer: Arc<RefCell<BufReader<io::Stdin>>>,
    // Dew web framework - current request context
    current_getback: Option<Value>,
    // Debug mode
    debug_mode: bool,
    // ULTRA-SECURE RUNTIME PROTECTION (Beyond Rust's guarantees)
    security_monitor: SecurityMonitor,
}
impl SecurityMonitor {
    pub fn new() -> Self {
        Self {
            recursion_depth: 0,
            memory_allocated: 0,
            execution_start: Instant::now(),
            loop_iterations: 0,
            stack_frames: 0,
            security_violations: Vec::new(),
        }
    }
    pub fn check_recursion_limit(&mut self) -> MintasResult<()> {
        self.recursion_depth += 1;
        if self.recursion_depth > MAX_RECURSION_DEPTH {
            let violation = format!("SECURITY VIOLATION: Recursion depth {} exceeds limit {}", 
                self.recursion_depth, MAX_RECURSION_DEPTH);
            self.security_violations.push(violation.clone());
            return Err(MintasError::RuntimeError {
                message: format!("Stack overflow protection: Maximum recursion depth ({}) exceeded. This prevents infinite recursion attacks.", MAX_RECURSION_DEPTH),
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(())
    }
    pub fn check_memory_limit(&mut self, size: usize) -> MintasResult<()> {
        self.memory_allocated += size;
        if self.memory_allocated > MAX_MEMORY_ALLOCATION {
            let violation = format!("SECURITY VIOLATION: Memory usage {} exceeds limit {}", 
                self.memory_allocated, MAX_MEMORY_ALLOCATION);
            self.security_violations.push(violation.clone());
            return Err(MintasError::RuntimeError {
                message: format!("Memory exhaustion protection: Maximum memory allocation ({} bytes) exceeded. This prevents memory bomb attacks.", MAX_MEMORY_ALLOCATION),
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(())
    }
    #[allow(dead_code)]
    pub fn check_execution_time(&mut self) -> MintasResult<()> {
        let elapsed = self.execution_start.elapsed();
        if elapsed > Duration::from_millis(MAX_EXECUTION_TIME_MS) {
            let violation = format!("SECURITY VIOLATION: Execution time {}ms exceeds limit {}ms", 
                elapsed.as_millis(), MAX_EXECUTION_TIME_MS);
            self.security_violations.push(violation.clone());
            return Err(MintasError::RuntimeError {
                message: format!("Execution timeout protection: Maximum execution time ({}ms) exceeded. This prevents infinite loop attacks.", MAX_EXECUTION_TIME_MS),
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(())
    }
    #[allow(dead_code)]
    pub fn check_loop_limit(&mut self) -> MintasResult<()> {
        self.loop_iterations += 1;
        if self.loop_iterations > MAX_LOOP_ITERATIONS {
            let violation = format!("SECURITY VIOLATION: Loop iterations {} exceeds limit {}", 
                self.loop_iterations, MAX_LOOP_ITERATIONS);
            self.security_violations.push(violation.clone());
            return Err(MintasError::RuntimeError {
                message: format!("Loop bomb protection: Maximum loop iterations ({}) exceeded. This prevents computational DoS attacks.", MAX_LOOP_ITERATIONS),
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(())
    }
    #[allow(dead_code)]
    pub fn check_array_size(&mut self, size: usize) -> MintasResult<()> {
        if size > MAX_ARRAY_SIZE {
            let violation = format!("SECURITY VIOLATION: Array size {} exceeds limit {}", 
                size, MAX_ARRAY_SIZE);
            self.security_violations.push(violation.clone());
            return Err(MintasError::RuntimeError {
                message: format!("Array overflow protection: Maximum array size ({} elements) exceeded. This prevents buffer overflow attacks.", MAX_ARRAY_SIZE),
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(())
    }
    #[allow(dead_code)]
    pub fn check_string_length(&mut self, length: usize) -> MintasResult<()> {
        if length > MAX_STRING_LENGTH {
            let violation = format!("SECURITY VIOLATION: String length {} exceeds limit {}", 
                length, MAX_STRING_LENGTH);
            self.security_violations.push(violation.clone());
            return Err(MintasError::RuntimeError {
                message: format!("String bomb protection: Maximum string length ({} characters) exceeded. This prevents string-based DoS attacks.", MAX_STRING_LENGTH),
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(())
    }
    pub fn exit_recursion(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }
    #[allow(dead_code)]
    pub fn get_security_report(&self) -> String {
        format!(
            "Security Monitor Report:\n\
            - Recursion Depth: {}/{}\n\
            - Memory Allocated: {}/{} bytes\n\
            - Execution Time: {}ms\n\
            - Loop Iterations: {}/{}\n\
            - Stack Frames: {}/{}\n\
            - Security Violations: {}",
            self.recursion_depth, MAX_RECURSION_DEPTH,
            self.memory_allocated, MAX_MEMORY_ALLOCATION,
            self.execution_start.elapsed().as_millis(),
            self.loop_iterations, MAX_LOOP_ITERATIONS,
            self.stack_frames, MAX_STACK_FRAMES,
            self.security_violations.len()
        )
    }
}
impl Evaluator {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            constants: std::collections::HashSet::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
            this_instance: None,
            stdout_buffer: Arc::new(RefCell::new(BufWriter::with_capacity(8192, io::stdout()))),
            stdin_buffer: Arc::new(RefCell::new(BufReader::with_capacity(8192, io::stdin()))),
            current_getback: None,
            debug_mode: false,
            security_monitor: SecurityMonitor::new(),
        }
    }
    #[allow(dead_code)]
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }
    fn check_recursion_limit(&mut self) -> MintasResult<()> {
        self.security_monitor.check_recursion_limit()
    }
    fn check_memory_limit(&mut self, additional_size: usize) -> MintasResult<()> {
        self.security_monitor.check_memory_limit(additional_size)
    }
    fn estimate_value_size(value: &Value) -> usize {
        match value {
            Value::Number(_) => 8,
            Value::Boolean(_) => 1,
            Value::String(s) => s.len() * 2, 
            Value::Array(arr) => {
                let mut size = 24; 
                for item in arr {
                    size += Self::estimate_value_size(item);
                }
                size
            }
            Value::Table(map) => {
                let mut size = 48; 
                for (key, value) in map {
                    size += key.len() * 2; 
                    size += Self::estimate_value_size(value);
                }
                size
            }
            _ => 16, 
        }
    }
    #[allow(dead_code)]
    pub fn set_getback(&mut self, getback: Value) {
        self.current_getback = Some(getback);
    }
    #[allow(dead_code)]
    pub fn clear_getback(&mut self) {
        self.current_getback = None;
    }
    pub fn get_variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }
    #[allow(dead_code)]
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }
    #[allow(dead_code)]
    pub fn get_constants(&self) -> &std::collections::HashSet<String> {
        &self.constants
    }
    #[allow(dead_code)]
    pub fn eval_line(&mut self, line: &str) -> MintasResult<Value> {
        let mut lexer = crate::lexer::Lexer::new(line);
        let tokens = lexer.tokenize()?;
        if tokens.is_empty() || matches!(tokens[0].token, crate::lexer::Token::EOF) {
            return Ok(Value::Empty);
        }
        let mut parser = crate::parser::Parser::new(tokens);
        let statements = parser.parse()?;
        let mut last_val = Value::Empty;
        for stmt in statements {
            last_val = self.eval(&stmt)?;
        }
        Ok(last_val)
    }
    fn load_module(&mut self, module_name: &str, alias: Option<&str>) -> MintasResult<()> {
        match module_name {
            "math" => {
                return self.load_compiled_module(module_name, alias);
            }
            "datetime" => {
                return self.load_compiled_module(module_name, alias);
            }
            "json" => {
                return self.load_compiled_module(module_name, alias);
            }
            "sqlite3" => {
                return self.load_compiled_module(module_name, alias);
            }
            "redis2" => {
                return self.load_compiled_module(module_name, alias);
            }
            "postsql" => {
                return self.load_compiled_module(module_name, alias);
            }
            "dew" => {
                return self.load_compiled_module(module_name, alias);
            }
            "timer" => {
                return self.load_compiled_module(module_name, alias);
            }
            "uuid" => {
                return self.load_compiled_module(module_name, alias);
            }
            "hash" => {
                return self.load_compiled_module(module_name, alias);
            }
            "base64" => {
                return self.load_compiled_module(module_name, alias);
            }
            "slug" => {
                return self.load_compiled_module(module_name, alias);
            }
            "validate" => {
                return self.load_compiled_module(module_name, alias);
            }
            "colors" => {
                return self.load_compiled_module(module_name, alias);
            }
            "cache" => {
                return self.load_compiled_module(module_name, alias);
            }
            "csv" => {
                return self.load_compiled_module(module_name, alias);
            }
            "os" => {
                return self.load_compiled_module(module_name, alias);
            }
            "env" => {
                return self.load_compiled_module(module_name, alias);
            }
            "path" => {
                return self.load_compiled_module(module_name, alias);
            }
            "sysfiles" => {
                return self.load_compiled_module(module_name, alias);
            }
            "subprocess" => {
                return self.load_compiled_module(module_name, alias);
            }
            "requests" => {
                return self.load_compiled_module(module_name, alias);
            }
            "sockets" => {
                return self.load_compiled_module(module_name, alias);
            }
            "dns" => {
                return self.load_compiled_module(module_name, alias);
            }
            "ping" => {
                return self.load_compiled_module(module_name, alias);
            }
            "smtp" => {
                return self.load_compiled_module(module_name, alias);
            }
            "ftp" => {
                return self.load_compiled_module(module_name, alias);
            }
            "ssh" => {
                return self.load_compiled_module(module_name, alias);
            }
            "openai" => {
                return self.load_compiled_module(module_name, alias);
            }
            "webhook" => {
                return self.load_compiled_module(module_name, alias);
            }
            "cron" => {
                return self.load_compiled_module(module_name, alias);
            }
            "worker" => {
                return self.load_compiled_module(module_name, alias);
            }
            "cluster" => {
                return self.load_compiled_module(module_name, alias);
            }
            "algorithm" => {
                return self.load_compiled_module(module_name, alias);
            }
            "asjokes" => {
                return self.load_compiled_module(module_name, alias);
            }
            "canvas" => {
                return self.load_compiled_module(module_name, alias);
            }
            "crypto" => {
                return self.load_compiled_module(module_name, alias);
            }
            "compress" => {
                return self.load_compiled_module(module_name, alias);
            }
            "queue" => {
                return self.load_compiled_module(module_name, alias);
            }
            "events" => {
                return self.load_compiled_module(module_name, alias);
            }
            "buffer" => {
                return self.load_compiled_module(module_name, alias);
            }
            "myqr" => {
                return self.load_compiled_module(module_name, alias);
            }
            "mypdf" => {
                return self.load_compiled_module(module_name, alias);
            }
            "myyaml" => {
                return self.load_compiled_module(module_name, alias);
            }
            "archive" => {
                return self.load_compiled_module(module_name, alias);
            }
            "cert" => {
                return self.load_compiled_module(module_name, alias);
            }
            "graphql" => {
                return self.load_compiled_module(module_name, alias);
            }
            "mqtt" => {
                return self.load_compiled_module(module_name, alias);
            }
            "mycli" | "cli" => {
                return self.load_compiled_module(module_name, alias);
            }
            "xdbx" | "debug" => {
                return self.load_compiled_module(module_name, alias);
            }
            _ => {}
        }
        let module_paths = vec![
            format!("{}.as", module_name),
            format!("lib/{}.as", module_name),
            format!("lib/{}.mintas", module_name),
        ];
        let mut module_content = None;
        for path in &module_paths {
            if let Ok(content) = std::fs::read_to_string(&path) {
                module_content = Some(content);
                break;
            }
        }
        match module_content {
            Some(content) => {
                let mut module_evaluator = Evaluator::new();
                let mut lexer = crate::lexer::Lexer::new(&content);
                let tokens = lexer.tokenize().map_err(|e| {
                    MintasError::RuntimeError {
                        message: format!("Error lexing module '{}': {}", module_name, e),
                        location: Self::default_location(),
                    }
                })?;
                if !tokens.is_empty() && !matches!(tokens[0].token, crate::lexer::Token::EOF) {
                    let mut parser = crate::parser::Parser::new(tokens);
                    let statements = parser.parse().map_err(|e| {
                        MintasError::RuntimeError {
                            message: format!("Error parsing module '{}': {}", module_name, e),
                            location: Self::default_location(),
                        }
                    })?;
                    for stmt in statements {
                        module_evaluator.eval(&stmt).map_err(|e| {
                            MintasError::RuntimeError {
                                message: format!("Error executing module '{}': {}", module_name, e),
                            location: Self::default_location(),
                        }
                    })?;
                    }
                }
                let use_prefix = alias.is_some();
                let prefix = alias.unwrap_or("");
                for (func_name, func) in &module_evaluator.functions {
                    let full_name = if use_prefix && !prefix.is_empty() {
                        format!("{}.{}", prefix, func_name)
                    } else {
                        func_name.clone()
                    };
                    self.functions.insert(full_name, func.clone());
                }
                for (var_name, var_value) in &module_evaluator.variables {
                    let full_name = if use_prefix && !prefix.is_empty() {
                        format!("{}.{}", prefix, var_name)
                    } else {
                        var_name.clone()
                    };
                    self.variables.insert(full_name, var_value.clone());
                }
                Ok(())
            }
            None => Err(MintasError::RuntimeError {
                message: format!("Module '{}' not found. Searched in current directory and lib/", module_name),
                location: Self::default_location(),
            }),
        }
    }
    fn load_compiled_module(&mut self, module_name: &str, alias: Option<&str>) -> MintasResult<()> {
        let prefix = alias.unwrap_or(module_name);
        match module_name {
            "math" => {
                let math_functions = vec![
                    "sin", "cos", "tan", "asin", "acos", "atan", "atan2",
                    "sinh", "cosh", "tanh", "sqrt", "cbrt", "pow", "exp", "exp2",
                    "ln", "log10", "log2", "abs", "floor", "ceil", "round", "trunc",
                    "min", "max", "random", "pi", "e"
                ];
                for func_name in math_functions {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function {
                        params: vec!["x".to_string()],
                        body: vec![],
                        is_lambda: true,
                    };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            #[cfg(feature = "datetime")]
            "datetime" => {
                let datetime_functions = vec![
                    "now", "today", "utcnow", "timestamp", "fromtimestamp",
                    "strptime", "strftime", "add_days", "add_hours", "add_minutes", "add_seconds",
                    "diff_days", "diff_hours", "diff_minutes", "diff_seconds",
                    "is_leap_year", "days_in_month", "days_in_year", "weekday", "isoformat", "parse"
                ];
                for func_name in datetime_functions {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function {
                        params: vec!["x".to_string()],
                        body: vec![],
                        is_lambda: true,
                    };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            #[cfg(not(feature = "datetime"))]
            "datetime" => {
                return Err(MintasError::RuntimeError {
                    message: "DateTime module not available. Compile with --features datetime".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "json")]
            "json" => {
                let json_functions = vec![
                    "encode", "decode", "pretty", "stringify", "parse",
                    "get", "set", "keys", "values", "has_key", "is_valid",
                    "merge", "to_table", "from_table"
                ];
                for func_name in json_functions {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function {
                        params: vec!["x".to_string()],
                        body: vec![],
                        is_lambda: true,
                    };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            #[cfg(not(feature = "json"))]
            "json" => {
                return Err(MintasError::RuntimeError {
                    message: "JSON module not available. Compile with --features json".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "networking")]
            "requests" => {
                let requests_functions = vec!["get", "post"];
                for func_name in requests_functions {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function {
                        params: vec!["x".to_string()],
                        body: vec![],
                        is_lambda: true,
                    };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            #[cfg(not(feature = "networking"))]
            "requests" => {
                return Err(MintasError::RuntimeError {
                    message: "Requests module not available. Compile with --features networking".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "networking")]
            "sockets" => {
                let sockets_functions = vec!["connect"];
                for func_name in sockets_functions {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function {
                        params: vec!["x".to_string()],
                        body: vec![],
                        is_lambda: true,
                    };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            #[cfg(not(feature = "networking"))]
            "sockets" => {
                return Err(MintasError::RuntimeError {
                    message: "Sockets module not available. Compile with --features networking".to_string(),
                    location: Self::default_location(),
                });
            }
            "openai" => {
                let openai_functions = vec![
                    "completion", "chat_completion", "set_base_url", "get_models", 
                    "embedding", "image_generation", "transcription", "moderation"
                ];
                for func_name in openai_functions {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function {
                        params: vec!["x".to_string()],
                        body: vec![],
                        is_lambda: true,
                    };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            #[cfg(feature = "database")]
            "sqlite3" => {
                let sqlite3_functions = vec![
                    "connect", "create_table", "insert", "select", "update", "delete",
                    "find", "count", "exists", "drop_table", "close"
                ];
                for func_name in sqlite3_functions {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function {
                        params: vec!["x".to_string()],
                        body: vec![],
                        is_lambda: true,
                    };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            #[cfg(not(feature = "database"))]
            "sqlite3" => {
                return Err(MintasError::RuntimeError {
                    message: "SQLite3 module not available. Compile with --features database".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "database")]
            "redis2" => {
                let redis2_functions = vec![
                    "connect", "set", "get", "delete", "exists", "expire", "ttl", "keys", "flush",
                    "increment", "decrement", "push", "pop", "list_range", "hash_set", "hash_get", "hash_get_all"
                ];
                for func_name in redis2_functions {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function {
                        params: vec!["x".to_string()],
                        body: vec![],
                        is_lambda: true,
                    };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            #[cfg(not(feature = "database"))]
            "redis2" => {
                return Err(MintasError::RuntimeError {
                    message: "Redis2 module not available. Compile with --features database".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "database")]
            "postsql" => {
                let postsql_functions = vec![
                    "connect", "create_table", "insert", "select", "update", "delete",
                    "find", "count", "exists", "drop_table", "close", "create_index",
                    "migrate", "transaction", "rollback", "commit"
                ];
                for func_name in postsql_functions {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function {
                        params: vec!["x".to_string()],
                        body: vec![],
                        is_lambda: true,
                    };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            #[cfg(not(feature = "database"))]
            "postsql" => {
                return Err(MintasError::RuntimeError {
                    message: "PostgreSQL module not available. Compile with --features database".to_string(),
                    location: Self::default_location(),
                });
            }
            "dew" => {
                let dew_functions = vec![
                    "main", "serve"
                ];
                for func_name in dew_functions {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function {
                        params: vec!["x".to_string()],
                        body: vec![],
                        is_lambda: true,
                    };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "timer" => {
                let funcs = vec!["now", "start", "stop", "end", "elapsed", "sleep", "wait", "timestamp", "measure"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "uuid" => {
                let funcs = vec!["v4", "v7", "validate", "parse", "nil"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "hash" => {
                let funcs = vec!["md5", "sha1", "sha256", "sha512", "bcrypt", "verify"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "base64" => {
                let funcs = vec!["encode", "decode", "url_encode", "url_decode"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "slug" => {
                let funcs = vec!["create", "from_string", "validate"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "validate" => {
                let funcs = vec!["email", "url", "phone", "credit_card", "ip", "ipv4", "ipv6", "uuid", "json", "number", "alpha", "alphanumeric"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "colors" => {
                let funcs = vec!["red", "green", "blue", "yellow", "cyan", "magenta", "white", "black", "bold", "dim", "italic", "underline", "reset", "rgb", "hex"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "cache" => {
                let funcs = vec!["set", "get", "has", "delete", "clear", "keys", "size", "ttl"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "csv" => {
                let funcs = vec!["parse", "stringify", "read", "write"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "os" => {
                let funcs = vec!["platform", "arch", "user", "hostname", "home", "cwd", "cpus", "memory", "uptime", "exit"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "env" => {
                let funcs = vec!["get", "set", "has", "remove", "all", "load"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "path" => {
                let funcs = vec!["join", "dirname", "basename", "extname", "resolve", "exists", "isfile", "isdir"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "sysfiles" => {
                let funcs = vec!["read", "write", "append", "copy", "move", "remove", "mkdir", "list", "glob", "size"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "subprocess" => {
                let funcs = vec!["run", "shell", "spawn", "output", "call"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "webhook" => {
                let funcs = vec!["create", "send", "verify", "sign", "queue", "retry", "batch"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "cron" => {
                let funcs = vec!["schedule", "parse", "next", "validate", "every", "daily", "hourly", "weekly", "monthly", "cancel", "list"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "worker" => {
                let funcs = vec!["spawn", "start", "stop", "status", "send", "receive", "pool", "terminate", "list"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "cluster" => {
                let funcs = vec!["fork", "is_master", "is_worker", "workers", "broadcast", "send", "on_message", "shutdown", "restart", "cpu_count"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "algorithm" => {
                let funcs = vec!["sort", "binary_search", "linear_search", "bubble_sort", "quick_sort", "merge_sort", "gcd", "lcm", "fibonacci", "factorial", "is_prime", "primes_up_to", "levenshtein", "shuffle", "reverse", "unique", "intersection", "union", "difference"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "asjokes" => {
                let funcs = vec!["joke", "pun", "fortune", "quote", "cowsay", "magic8ball", "dice", "coin", "rps", "trivia", "riddle", "tongue_twister", "compliment", "insult", "excuse", "fact"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "canvas" => {
                let funcs = vec!["create", "update", "is_open", "quit", "clear", "rect", "fill_rect", "circle", "fill_circle", "line", "pixel", "text", "sprite", "set", "get", "move", "move_toward", "draw", "draw_all", "delete", "exists", "count", "list", "collide", "collide_point", "collide_tag", "collide_any", "overlap", "physics", "gravity", "velocity", "accelerate", "friction", "bounce", "wrap", "jump", "platform", "key", "key_down", "key_pressed", "mouse_x", "mouse_y", "mouse", "mouse_down", "mouse_clicked", "click", "camera", "camera_follow", "shake", "width", "height", "rgb", "rgba", "distance", "angle", "random", "random_int", "lerp", "clamp", "delta", "fps", "frame", "sin", "cos"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "crypto" => {
                let funcs = vec!["encrypt", "decrypt", "hash", "hmac", "random_bytes", "random_hex", "random_string", "uuid", "constant_time_compare", "xor"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "compress" => {
                let funcs = vec!["deflate", "inflate", "gzip", "gunzip", "zip", "unzip", "compress", "decompress"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "queue" => {
                let funcs = vec!["create", "push", "enqueue", "pop", "dequeue", "peek", "front", "size", "len", "empty", "is_empty", "clear", "list", "delete"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "events" => {
                let funcs = vec!["on", "listen", "emit", "trigger", "off", "remove", "once", "clear", "listeners", "events"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "buffer" => {
                let funcs = vec!["create", "alloc", "from", "from_hex", "from_base64", "to_string", "to_hex", "to_base64", "concat", "slice", "length", "len", "get", "set", "fill", "copy", "equals", "compare"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "myqr" => {
                let funcs = vec!["generate", "create", "to_ascii", "to_svg", "to_html"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "mypdf" => {
                let funcs = vec!["create", "add_page", "add_text", "add_image", "add_line", "add_rect", "set_font", "save", "to_string"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "myyaml" => {
                let funcs = vec!["parse", "load", "stringify", "dump", "get", "set"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "archive" => {
                let funcs = vec!["create", "add", "extract", "list", "save"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "cert" => {
                let funcs = vec!["generate", "load", "verify", "info", "sign", "self_signed"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "graphql" => {
                let funcs = vec!["query", "mutation", "subscribe", "client", "build_query"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "mqtt" => {
                let funcs = vec!["connect", "publish", "subscribe", "unsubscribe", "disconnect", "on_message"];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "mycli" | "cli" => {
                let funcs = vec![
                    "app", "command", "option", "flag", "run", "version", "help",
                    "print", "println", "success", "error", "warning", "info", "debug", "bold", "dim",
                    "color", "bg", "hex", "style",
                    "progress_bar", "progress_update", "progress_done", "spinner", "spinner_success", "spinner_error",
                    "prompt", "password", "confirm", "select", "multiselect", "number",
                    "table", "clear", "newline", "separator",
                    "args", "parse_args", "subcommand", "required", "default", "env_var",
                    "box", "panel", "divider", "banner", "tree", "list",
                    "task", "task_done", "task_fail", "task_skip",
                    "log", "log_file", "log_level",
                    "autocomplete", "history", "edit"
                ];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            "xdbx" | "debug" => {
                let funcs = vec![
                    "start", "stop", "pause", "resume", "step", "step_over", "step_into", "step_out",
                    "breakpoint", "remove_breakpoint", "list_breakpoints", "enable_breakpoint", 
                    "disable_breakpoint", "conditional_break",
                    "watch", "unwatch", "inspect", "locals", "globals", "stack", "evaluate",
                    "init", "build", "run", "test", "clean", "install", "uninstall", 
                    "list_packages", "update", "publish",
                    "target", "targets", "release", "debug_build",
                    "info", "version", "config"
                ];
                for func_name in funcs {
                    let full_name = format!("{}.{}", prefix, func_name);
                    let dummy_function = Function { params: vec!["x".to_string()], body: vec![], is_lambda: true };
                    self.functions.insert(full_name, dummy_function);
                }
            }
            _ => {
                return Err(MintasError::RuntimeError {
                    message: format!("Unknown compiled module '{}'", module_name),
                    location: Self::default_location(),
                });
            }
        }
        Ok(())
    }
    fn default_location() -> SourceLocation {
        SourceLocation::new(0, 0)
    }
    pub fn eval(&mut self, expr: &Expr) -> MintasResult<Value> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => {
                Ok(Value::String(self.interpolate_string(s)?))
            }
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
            Expr::Maybe => Ok(Value::Maybe),
            Expr::Empty => Ok(Value::Empty),
            Expr::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    let value = self.eval(elem)?;
                    let element_size = Self::estimate_value_size(&value);
                    self.check_memory_limit(element_size)?;
                    values.push(value);
                }
                let array_value = Value::Array(values);
                Ok(array_value)
            }
            Expr::Table(pairs) => {
                let mut map = std::collections::HashMap::new();
                for (key, value_expr) in pairs {
                    let value = self.eval(value_expr)?;
                    let key_size = key.len() * 2; 
                    let value_size = Self::estimate_value_size(&value);
                    self.check_memory_limit(key_size + value_size)?;
                    map.insert(key.clone(), value);
                }
                let table_value = Value::Table(map);
                Ok(table_value)
            }
            Expr::SuperSet(inner_expr) => {
                let inner_value = self.eval(inner_expr)?;
                Ok(Value::SuperSet(Box::new(inner_value)))
            }
            Expr::Variable(name) => {
                self.variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| MintasError::UndefinedVariable {
                        name: name.clone(),
                        location: Self::default_location(),
                    })
            }
            Expr::BinaryOp { op, left, right } => {
                self.eval_binary_op(op, left, right)
            }
            Expr::UnaryOp { op, expr } => {
                self.eval_unary_op(op, expr)
            }
            Expr::Assign { name, value, is_const } => {
                if self.constants.contains(name) {
                    return Err(MintasError::ConstantReassignment {
                        name: name.clone(),
                        location: Self::default_location(),
                    });
                }
                let val = self.eval(value)?;
                if let Value::Function(func) = &val {
                    self.functions.insert(name.clone(), func.as_ref().clone());
                }
                self.variables.insert(name.clone(), val.clone());
                if *is_const {
                    self.constants.insert(name.clone());
                }
                Ok(val)
            }
            Expr::MultiAssign { names, values, is_const } => {
                let mut last_val = Value::Empty;
                for (name, value_expr) in names.iter().zip(values.iter()) {
                    if self.constants.contains(name) {
                        return Err(MintasError::ConstantReassignment {
                            name: name.clone(),
                            location: Self::default_location(),
                        });
                    }
                    let val = self.eval(value_expr)?;
                    if let Value::Function(func) = &val {
                        self.functions.insert(name.clone(), func.as_ref().clone());
                    }
                    self.variables.insert(name.clone(), val.clone());
                    if *is_const {
                        self.constants.insert(name.clone());
                    }
                    last_val = val;
                }
                Ok(last_val)
            }
            Expr::CompoundAssign { name, op, value } => {
                if self.constants.contains(name) {
                    return Err(MintasError::ConstantReassignment {
                        name: name.clone(),
                        location: Self::default_location(),
                    });
                }
                let current = self.variables.get(name).cloned().ok_or_else(|| {
                    MintasError::UndefinedVariable {
                        name: name.clone(),
                        location: Self::default_location(),
                    }
                })?;
                let right_val = self.eval(value)?;
                let result = self.apply_binary_op(op, &current, &right_val)?;
                self.variables.insert(name.clone(), result.clone());
                Ok(result)
            }
            Expr::Call { name, args } => {
                self.eval_call(name, args)
            }
            Expr::IfExpr { condition, then_branch, else_if_branches, else_branch } => {
                let cond_val = self.eval(condition)?;
                let cond_result = cond_val.is_truthy_in_condition();
                match cond_result {
                    Value::Boolean(true) => self.eval_block(then_branch),
                    Value::Maybe => {
                        if let Some(else_body) = else_branch {
                            self.eval_block(else_body)
                        } else {
                            Ok(Value::Empty)
                        }
                    }
                    _ => {
                        for (elif_cond, elif_body) in else_if_branches {
                            let elif_val = self.eval(elif_cond)?;
                            let elif_result = elif_val.is_truthy_in_condition();
                            match elif_result {
                                Value::Boolean(true) => return self.eval_block(elif_body),
                                Value::Maybe => continue, 
                                _ => continue,
                            }
                        }
                        if let Some(else_body) = else_branch {
                            self.eval_block(else_body)
                        } else {
                            Ok(Value::Empty)
                        }
                    }
                }
            }
            Expr::WhileLoop { condition, body } => {
                let mut result = Value::Empty;
                loop {
                    let cond_val = self.eval(condition)?;
                    let cond_result = cond_val.is_truthy_in_condition();
                    match cond_result {
                        Value::Boolean(false) | Value::Maybe => break, 
                        Value::Boolean(true) => {} 
                        _ => break, 
                    }
                    for stmt in body {
                        let val = self.eval(stmt)?;
                        if matches!(val, Value::ExitSignal) {
                            return Ok(result);
                        }
                        if matches!(val, Value::ProceedSignal) {
                            break;
                        }
                        result = val;
                    }
                }
                Ok(result)
            }
            Expr::ForLoop { var, start, end, body } => {
                let start_val = match self.eval(start)? {
                    Value::Number(n) => n as i64,
                    _ => return Err(MintasError::TypeError {
                        message: "For loop start must be a number".to_string(),
                        location: Self::default_location(),
                    }),
                };
                let end_val = match self.eval(end)? {
                    Value::Number(n) => n as i64,
                    _ => return Err(MintasError::TypeError {
                        message: "For loop end must be a number".to_string(),
                        location: Self::default_location(),
                    }),
                };
                let mut result = Value::Empty;
                let ascending = start_val <= end_val;
                let mut i = start_val;
                'outer: loop {
                    if ascending {
                        if i > end_val { break; }
                    } else {
                        if i < end_val { break; }
                    }
                    self.variables.insert(var.clone(), Value::Number(i as f64));
                    for stmt in body {
                        let val = self.eval(stmt)?;
                        if matches!(val, Value::ExitSignal) {
                            break 'outer;
                        }
                        if matches!(val, Value::ProceedSignal) {
                            break;
                        }
                        result = val;
                    }
                    if ascending { i += 1; } else { i -= 1; }
                }
                Ok(result)
            }
            Expr::ForInLoop { var, iterable, body } => {
                let iter_val = self.eval(iterable)?;
                let items: Vec<Value> = match iter_val {
                    Value::Array(arr) => arr,
                    Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
                    Value::Table(map) => map.keys().map(|k| Value::String(k.clone())).collect(),
                    _ => return Err(MintasError::TypeError {
                        message: "For-in loop requires array, string, or table".to_string(),
                        location: Self::default_location(),
                    }),
                };
                let mut result = Value::Empty;
                'outer: for item in items {
                    self.variables.insert(var.clone(), item);
                    for stmt in body {
                        let val = self.eval(stmt)?;
                        if matches!(val, Value::ExitSignal) {
                            break 'outer;
                        }
                        if matches!(val, Value::ProceedSignal) {
                            break;
                        }
                        result = val;
                    }
                }
                Ok(result)
            }
            Expr::Exit => Ok(Value::ExitSignal),
            Expr::Proceed => Ok(Value::ProceedSignal),
            Expr::MethodCall { object, method, args } => {
                self.eval_method_call(object, method, args)
            }
            Expr::Index { object, index } => {
                let obj_val = self.eval(object)?;
                let idx_val = self.eval(index)?;
                match (&obj_val, &idx_val) {
                    (Value::Array(arr), Value::Number(n)) => {
                        let idx = (*n as i64 - 1) as usize;
                        arr.get(idx).cloned().ok_or_else(|| MintasError::RuntimeError {
                            message: format!("Index {} out of bounds", n),
                            location: Self::default_location(),
                        })
                    }
                    (Value::String(s), Value::Number(n)) => {
                        let idx = (*n as i64 - 1) as usize;
                        s.chars().nth(idx).map(|c| Value::String(c.to_string())).ok_or_else(|| MintasError::RuntimeError {
                            message: format!("Index {} out of bounds", n),
                            location: Self::default_location(),
                        })
                    }
                    (Value::Table(map), Value::String(key)) => {
                        map.get(key).cloned().ok_or_else(|| MintasError::RuntimeError {
                            message: format!("Key '{}' not found", key),
                            location: Self::default_location(),
                        })
                    }
                    _ => Err(MintasError::TypeError {
                        message: format!("Cannot index {} with {}", obj_val.type_name(), idx_val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            Expr::Ternary { condition, then_expr, else_expr } => {
                let cond_val = self.eval(condition)?;
                let cond_result = cond_val.is_truthy_in_condition();
                match cond_result {
                    Value::Boolean(true) => self.eval(then_expr),
                    Value::Maybe => self.eval(else_expr), 
                    _ => self.eval(else_expr),
                }
            }
            Expr::SmartCondition { condition, then_branch, else_branch } => {
                let cond_val = self.eval(condition)?;
                if cond_val.is_truthy() {
                    self.eval(then_branch)
                } else {
                    self.eval(else_branch)
                }
            }
            Expr::SmartLoop { var, count, body } => {
                let count_val = self.eval(count)?;
                let count_num = match count_val {
                    Value::Number(n) => n as i64,
                    _ => return Err(MintasError::RuntimeError {
                        message: "Loop count must be a number".to_string(),
                        location: Self::default_location(),
                    }),
                };
                let mut result = Value::Empty;
                'outer: for i in 0..count_num {
                    self.variables.insert(var.clone(), Value::Number(i as f64));
                    for stmt in body {
                        let val = self.eval(stmt)?;
                        if matches!(val, Value::ExitSignal) {
                            break 'outer;
                        }
                        if matches!(val, Value::ProceedSignal) {
                            break;
                        }
                        result = val;
                    }
                }
                Ok(result)
            }
            Expr::Function { name, params, body, is_lambda } => {
                let func = Function {
                    params: params.clone(),
                    body: body.clone(),
                    is_lambda: *is_lambda,
                };
                let func_value = Value::Function(Box::new(func.clone()));
                self.functions.insert(name.clone(), func);
                Ok(func_value)
            }
            Expr::Return { value } => {
                let ret_val = if let Some(expr) = value {
                    self.eval(expr)?
                } else {
                    Value::Empty
                };
                Ok(Value::ReturnSignal(Box::new(ret_val)))
            }
            Expr::Class { name, members, inheritance } => {
                let class = Class {
                    name: name.clone(),
                    members: members.clone(),
                    inheritance: inheritance.clone(),
                };
                let class_value = Value::Class(Box::new(class.clone()));
                self.classes.insert(name.clone(), class);
                Ok(class_value)
            }
            Expr::New { class_name, args: _ } => {
                let class = self.classes.get(class_name.as_str()).ok_or_else(|| {
                    MintasError::RuntimeError {
                        message: format!("Class '{}' not found", class_name),
                        location: Self::default_location(),
                    }
                })?.clone();
                let mut instance = Instance {
                    class_name: class_name.clone(),
                    fields: HashMap::new(),
                    methods: HashMap::new(),
                };
                if let ClassInheritance::Extends(parent_name) = &class.inheritance {
                    let parent_members = self.classes.get(parent_name).map(|pc| pc.members.clone()).unwrap_or_default();
                    for member in parent_members {
                        match member {
                            ClassMember::Property { name, is_public: _, initial_value } => {
                                    let value = if let Some(ref init) = initial_value {
                                        self.eval(init)?
                                    } else {
                                        Value::Empty
                                    };
                                instance.fields.insert(name.clone(), value);
                            }
                            ClassMember::Method { name, is_public: _, params, body } => {
                                let func = Function {
                                    params: params.clone(),
                                    body: body.clone(),
                                    is_lambda: false,
                                };
                                instance.methods.insert(name.clone(), func);
                            }
                        }
                    }
                }
                for member in &class.members {
                    match member {
                        ClassMember::Property { name, is_public: _, initial_value } => {
                                    let value = if let Some(ref init) = initial_value {
                                        self.eval(init)?
                                    } else {
                                        Value::Empty
                                    };
                            instance.fields.insert(name.clone(), value);
                        }
                        ClassMember::Method { name, is_public: _, params, body } => {
                            let func = Function {
                                params: params.clone(),
                                body: body.clone(),
                                is_lambda: false,
                            };
                            instance.methods.insert(name.clone(), func);
                        }
                    }
                }
                let mut instance = Instance {
                    class_name: class_name.clone(),
                    fields: HashMap::new(),
                    methods: HashMap::new(),
                };
                let old_this = self.this_instance.take();
                self.this_instance = Some(Box::new(instance.clone()));
                for member in &class.members {
                    match member {
                        ClassMember::Property { name, is_public: _, initial_value } => {
                                    let value = if let Some(ref init) = initial_value {
                                        self.eval(init)?
                                    } else {
                                        Value::Empty
                                    };
                            instance.fields.insert(name.clone(), value);
                        }
                        ClassMember::Method { name, is_public: _, params, body } => {
                            let func = Function {
                                params: params.clone(),
                                body: body.clone(),
                                is_lambda: false,
                            };
                            instance.methods.insert(name.clone(), func);
                        }
                    }
                }
                self.this_instance = Some(Box::new(instance.clone()));
                let result = Ok(Value::Instance(Box::new(instance)));
                self.this_instance = old_this;
                result
            }
            Expr::This => {
                if let Some(instance) = &self.this_instance {
                    Ok(Value::Instance(Box::new(instance.as_ref().clone())))
                } else {
                    Err(MintasError::RuntimeError {
                        message: "'this' can only be used inside class methods".to_string(),
                        location: Self::default_location(),
                    })
                }
            }
            Expr::Property { object, property } => {
                if let Expr::Variable(var_name) = &**object {
                    if var_name == "math" {
                        return match property.as_str() {
                            "pi" => Ok(Value::Number(std::f64::consts::PI)),
                            "e" => Ok(Value::Number(std::f64::consts::E)),
                            _ => Err(MintasError::RuntimeError {
                                message: format!("Math module has no property '{}'", property),
                                location: Self::default_location(),
                            }),
                        };
                    }
                }
                let obj_val = self.eval(object)?;
                match obj_val {
                    Value::Instance(instance) => {
                        instance.fields.get(property.as_str()).cloned().ok_or_else(|| {
                            MintasError::RuntimeError {
                                message: format!("Property '{}' not found", property),
                                location: Self::default_location(),
                            }
                        })
                    }
                    Value::Table(map) => {
                        map.get(property.as_str()).cloned().ok_or_else(|| {
                            MintasError::RuntimeError {
                                message: format!("Key '{}' not found", property),
                                location: Self::default_location(),
                            }
                        })
                    }
                    _ => Err(MintasError::TypeError {
                        message: format!("Cannot access property on {}", obj_val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            Expr::PropertyAssign { object, property, value } => {
                let new_value = self.eval(value)?;
                if let Expr::Variable(var_name) = &**object {
                    if let Some(Value::Table(mut map)) = self.variables.get(var_name).cloned() {
                        map.insert(property.clone(), new_value.clone());
                        self.variables.insert(var_name.clone(), Value::Table(map));
                        return Ok(new_value);
                    }
                    if let Some(Value::Instance(instance)) = self.variables.get(var_name).cloned() {
                        let mut inst = instance.as_ref().clone();
                        inst.fields.insert(property.clone(), new_value.clone());
                        self.variables.insert(var_name.clone(), Value::Instance(Box::new(inst)));
                        return Ok(new_value);
                    }
                    let mut map = std::collections::HashMap::new();
                    map.insert(property.clone(), new_value.clone());
                    self.variables.insert(var_name.clone(), Value::Table(map));
                    return Ok(new_value);
                }
                Err(MintasError::RuntimeError {
                    message: "Property assignment requires a variable as the object".to_string(),
                    location: Self::default_location(),
                })
            }
            Expr::TryCatch { try_block, catch_block, error_var } => {
                match self.eval_block(try_block) {
                    Ok(val) => Ok(val),
                    Err(err) => {
                        let error_value = Value::String(err.to_string());
                        let old_vars = self.variables.clone();
                        if let Some(var_name) = error_var {
                            self.variables.insert(var_name.to_string(), error_value);
                        }
                        let result = self.eval_block(catch_block);
                        self.variables = old_vars;
                        result
                    }
                }
            }
            Expr::Cond { condition } => {
                let cond_value = self.eval(condition)?;
                let is_true = cond_value.is_truthy();
                let mut cond_table = std::collections::HashMap::new();
                cond_table.insert("__type__".to_string(), Value::String("Condition".to_string()));
                cond_table.insert("value".to_string(), Value::Boolean(is_true));
                Ok(Value::Table(cond_table))
            }
            Expr::Follow { condition, negate } => {
                let cond_value = self.eval(condition)?;
                let bool_value = match &cond_value {
                    Value::Table(map) => {
                        if map.get("__type__").map(|v| matches!(v, Value::String(s) if s == "Condition")).unwrap_or(false) {
                            match map.get("value") {
                                Some(Value::Boolean(b)) => *b,
                                _ => false,
                            }
                        } else {
                            cond_value.is_truthy()
                        }
                    }
                    Value::Boolean(b) => *b,
                    _ => cond_value.is_truthy(),
                };
                Ok(Value::Boolean(if *negate { !bool_value } else { bool_value }))
            }
            Expr::Include { module_name, alias } => {
                self.load_module(module_name, alias.as_deref())?;
                Ok(Value::Empty)
            }
            Expr::Task { name, params, body } => {
                let task_function = Function {
                    params: params.clone(),
                    body: body.clone(),
                    is_lambda: false,
                };
                self.functions.insert(name.clone(), task_function);
                Ok(Value::Empty)
            }
            Expr::Switch { expression, cases, default_case } => {
                let switch_value = self.eval(expression)?;
                for (case_value_expr, case_body) in cases {
                    let case_value = self.eval(case_value_expr)?;
                    if self.values_equal(&switch_value, &case_value) {
                        let mut result = Value::Empty;
                        for stmt in case_body {
                            result = self.eval(&stmt)?;
                            if matches!(result, Value::ExitSignal | Value::ProceedSignal | Value::ReturnSignal(_)) {
                                return Ok(result);
                            }
                        }
                        return Ok(result);
                    }
                }
                if let Some(default_body) = default_case {
                    let mut result = Value::Empty;
                    for stmt in default_body {
                        result = self.eval(&stmt)?;
                        if matches!(result, Value::ExitSignal | Value::ProceedSignal | Value::ReturnSignal(_)) {
                            return Ok(result);
                        }
                    }
                    Ok(result)
                } else {
                    Ok(Value::Empty)
                }
            }
            Expr::DewRoute { server, method, path, body } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                dew_module::add_server_route(server_id, method, path, body.clone())?;
                Ok(Value::Empty)
            }
            Expr::DewServe { server, port, host } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                let port_val = self.eval(port)?;
                let port_num = match port_val {
                    Value::Number(n) => n as u16,
                    _ => 3000,
                };
                let host_str = if let Some(h) = host {
                    match self.eval(h)? {
                        Value::String(s) => s,
                        _ => "127.0.0.1".to_string(),
                    }
                } else {
                    "127.0.0.1".to_string()
                };
                let args = vec![
                    Value::Number(port_num as f64),
                    Value::String(host_str),
                    Value::Number(server_id as f64),
                ];
                dew_module::DewModule::call_function("serve", &args)
            }
            Expr::Getback => {
                if let Some(getback) = &self.current_getback {
                    Ok(getback.clone())
                } else {
                    Ok(Value::Table(HashMap::new()))
                }
            }
            Expr::DewReturn { response_type, body, status, data } => {
                let body_val = self.eval(body)?;
                if response_type == "inview" {
                    let data_val = if let Some(d) = data {
                        self.eval(d)?
                    } else {
                        Value::Table(HashMap::new())
                    };
                    let args = vec![body_val, data_val];
                    let result = dew_module::DewModule::call_function("inview", &args)?;
                    return Ok(Value::ReturnSignal(Box::new(result)));
                }
                let body_str = match body_val {
                    Value::String(s) => s,
                    Value::Table(map) => {
                        self.table_to_json(&map)
                    }
                    other => format!("{:?}", other),
                };
                let status_val = if let Some(s) = status {
                    match self.eval(s)? {
                        Value::Number(n) => Some(n as u16),
                        _ => None,
                    }
                } else {
                    None
                };
                let mut response = HashMap::new();
                response.insert("__type__".to_string(), Value::String("DewResponse".to_string()));
                response.insert("response_type".to_string(), Value::String(response_type.clone()));
                response.insert("body".to_string(), Value::String(body_str));
                response.insert("status".to_string(), Value::Number(status_val.unwrap_or(200) as f64));
                Ok(Value::ReturnSignal(Box::new(Value::Table(response))))
            }
            Expr::DewBefore { server, body } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                dew_module::add_server_before_handler(server_id, body.clone())?;
                Ok(Value::Empty)
            }
            Expr::DewAfter { server, body } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                dew_module::add_server_after_handler(server_id, body.clone())?;
                Ok(Value::Empty)
            }
            Expr::DewUse { server, middleware } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                dew_module::add_server_middleware(server_id, middleware)?;
                Ok(Value::Empty)
            }
            Expr::DewCatch { server, status_code, body } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                dew_module::add_server_error_handler(server_id, *status_code, body.clone())?;
                Ok(Value::Empty)
            }
            Expr::DewGroup { server, prefix, body } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                dew_module::start_route_group(server_id, prefix)?;
                for stmt in body {
                    self.eval(stmt)?;
                }
                dew_module::end_route_group(server_id)?;
                Ok(Value::Empty)
            }
            Expr::DewStatic { server, url_path, dir_path } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                dew_module::add_server_static(server_id, url_path, dir_path)?;
                Ok(Value::Empty)
            }
            Expr::DewRouteValidated { server, method, path, validation_rules, body } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                let rules_val = self.eval(validation_rules)?;
                dew_module::add_server_validated_route(server_id, method, path, rules_val, body.clone())?;
                Ok(Value::Empty)
            }
            Expr::DewConfig { server, config_path } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                dew_module::load_server_config(server_id, config_path)?;
                Ok(Value::Empty)
            }
            Expr::DewDatabase { server, connection_string } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                dew_module::setup_server_database(server_id, connection_string)?;
                Ok(Value::Empty)
            }
            Expr::DewSession { server, config } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                let config_val = if let Some(c) = config {
                    self.eval(c)?
                } else {
                    Value::Table(HashMap::new())
                };
                dew_module::setup_server_session(server_id, config_val)?;
                Ok(Value::Empty)
            }
            Expr::DewRateLimit { server, requests, window_seconds } => {
                let server_val = self.eval(server)?;
                let server_id = match &server_val {
                    Value::Table(map) => {
                        match map.get("__dew_server_id__") {
                            Some(Value::Number(id)) => *id as usize,
                            _ => return Err(MintasError::RuntimeError {
                                message: "Invalid Dew server object".to_string(),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => return Err(MintasError::RuntimeError {
                        message: "Expected Dew server object".to_string(),
                        location: Self::default_location(),
                    }),
                };
                dew_module::setup_server_rate_limit(server_id, *requests, *window_seconds)?;
                Ok(Value::Empty)
            }
        }
    }
    fn eval_method_call(&mut self, object: &Expr, method: &str, args: &[Expr]) -> MintasResult<Value> {
        if let Expr::Variable(var_name) = object {
            if var_name == "math" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return math_module::MathModule::call_function(method, &evaluated_args);
            }
            #[cfg(feature = "datetime")]
            if var_name == "datetime" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return datetime_module::DateTimeModule::call_function(method, &evaluated_args);
            }
            #[cfg(not(feature = "datetime"))]
            if var_name == "datetime" {
                return Err(MintasError::RuntimeError {
                    message: "DateTime module not available. Compile with --features datetime".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "json")]
            if var_name == "json" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return json_module::JsonModule::call_function(method, &evaluated_args);
            }
            #[cfg(not(feature = "json"))]
            if var_name == "json" {
                return Err(MintasError::RuntimeError {
                    message: "JSON module not available. Compile with --features json".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "networking")]
            if var_name == "requests" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return requests_module::RequestsModule::call_function(method, &evaluated_args);
            }
            #[cfg(not(feature = "networking"))]
            if var_name == "requests" {
                return Err(MintasError::RuntimeError {
                    message: "Requests module not available. Compile with --features networking".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "networking")]
            if var_name == "sockets" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return sockets_module::SocketsModule::call_function(method, &evaluated_args);
            }
            #[cfg(not(feature = "networking"))]
            if var_name == "sockets" {
                return Err(MintasError::RuntimeError {
                    message: "Sockets module not available. Compile with --features networking".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "ai")]
            if var_name == "openai" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return openai_module::OpenAIModule::call_function(method, &evaluated_args);
            }
            #[cfg(not(feature = "ai"))]
            if var_name == "openai" {
                return Err(MintasError::RuntimeError {
                    message: "OpenAI module not available. Compile with --features ai".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "database")]
            if var_name == "sqlite3" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return sqlite3_module::SQLite3Module::call_function(method, &evaluated_args);
            }
            #[cfg(not(feature = "database"))]
            if var_name == "sqlite3" {
                return Err(MintasError::RuntimeError {
                    message: "SQLite3 module not available. Compile with --features database".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "database")]
            if var_name == "redis2" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return redis2_module::Redis2Module::call_function(method, &evaluated_args);
            }
            #[cfg(not(feature = "database"))]
            if var_name == "redis2" {
                return Err(MintasError::RuntimeError {
                    message: "Redis2 module not available. Compile with --features database".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "database")]
            if var_name == "postsql" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return postsql_module::PostSqlModule::call_function(method, &evaluated_args);
            }
            #[cfg(not(feature = "database"))]
            if var_name == "postsql" {
                return Err(MintasError::RuntimeError {
                    message: "PostgreSQL module not available. Compile with --features database".to_string(),
                    location: Self::default_location(),
                });
            }
            if var_name == "dew" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return dew_module::DewModule::call_function(method, &evaluated_args);
            }
            if var_name == "dns" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return dns_module::DnsModule::call_function(method, &evaluated_args);
            }
            if var_name == "ping" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return ping_module::PingModule::call_function(method, &evaluated_args);
            }
            #[cfg(feature = "smtp")]
            if var_name == "smtp" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return smtp_module::SmtpModule::call_function(method, &evaluated_args);
            }
            #[cfg(not(feature = "smtp"))]
            if var_name == "smtp" {
                return Err(MintasError::RuntimeError {
                    message: "SMTP module not available. Compile with --features smtp".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "ftp")]
            if var_name == "ftp" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return ftp_module::FtpModule::call_function(method, &evaluated_args);
            }
            #[cfg(not(feature = "ftp"))]
            if var_name == "ftp" {
                return Err(MintasError::RuntimeError {
                    message: "FTP module not available. Compile with --features ftp".to_string(),
                    location: Self::default_location(),
                });
            }
            #[cfg(feature = "ssh")]
            if var_name == "ssh" {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                return ssh_module::SshModule::call_function(method, &evaluated_args);
            }
            #[cfg(not(feature = "ssh"))]
            if var_name == "ssh" {
                return Err(MintasError::RuntimeError {
                    message: "SSH module not available. Compile with --features ssh".to_string(),
                    location: Self::default_location(),
                });
            }
            if var_name == "os" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return os_module::OsModule::call_function(method, &evaluated_args);
            }
            if var_name == "env" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return env_module::EnvModule::call_function(method, &evaluated_args);
            }
            if var_name == "path" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return path_module::PathModule::call_function(method, &evaluated_args);
            }
            if var_name == "sysfiles" || var_name == "fs" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return sysfiles_module::SysfilesModule::call_function(method, &evaluated_args);
            }
            if var_name == "subprocess" || var_name == "proc" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return subprocess_module::SubprocessModule::call_function(method, &evaluated_args);
            }
            if var_name == "base64" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return base64_module::Base64Module::call_function(method, &evaluated_args);
            }
            if var_name == "uuid" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return uuid_module::UuidModule::call_function(method, &evaluated_args);
            }
            if var_name == "hash" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return hash_module::HashModule::call_function(method, &evaluated_args);
            }
            if var_name == "csv" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return csv_module::CsvModule::call_function(method, &evaluated_args);
            }
            if var_name == "colors" || var_name == "color" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return colors_module::ColorsModule::call_function(method, &evaluated_args);
            }
            if var_name == "timer" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return timer_module::TimerModule::call_function(method, &evaluated_args);
            }
            if var_name == "slug" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return slug_module::SlugModule::call_function(method, &evaluated_args);
            }
            if var_name == "validate" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return validate_module::ValidateModule::call_function(method, &evaluated_args);
            }
            if var_name == "cache" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return cache_module::CacheModule::call_function(method, &evaluated_args);
            }
            if var_name == "webhook" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return webhook_module::WebhookModule::call_function(method, &evaluated_args);
            }
            if var_name == "cron" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return cron_module::CronModule::call_function(method, &evaluated_args);
            }
            if var_name == "worker" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return worker_module::WorkerModule::call_function(method, &evaluated_args);
            }
            if var_name == "cluster" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return cluster_module::ClusterModule::call_function(method, &evaluated_args);
            }
            if var_name == "algorithm" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return algorithm_module::AlgorithmModule::call_function(method, &evaluated_args);
            }
            if var_name == "asjokes" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return asjokes_module::AsJokesModule::call_function(method, &evaluated_args);
            }
            if var_name == "canvas" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return canvas_module::CanvasModule::call_function(method, &evaluated_args);
            }
            if var_name == "crypto" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return crypto_module::CryptoModule::call_function(method, &evaluated_args);
            }
            if var_name == "compress" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return compress_module::CompressModule::call_function(method, &evaluated_args);
            }
            if var_name == "queue" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return queue_module::QueueModule::call_function(method, &evaluated_args);
            }
            if var_name == "events" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return events_module::EventsModule::call_function(method, &evaluated_args);
            }
            if var_name == "buffer" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return buffer_module::BufferModule::call_function(method, &evaluated_args);
            }
            if var_name == "myqr" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return myqr_module::MyqrModule::call_function(method, &evaluated_args);
            }
            if var_name == "mypdf" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return mypdf_module::MypdfModule::call_function(method, &evaluated_args);
            }
            if var_name == "myyaml" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return myyaml_module::MyyamlModule::call_function(method, &evaluated_args);
            }
            if var_name == "archive" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return archive_module::ArchiveModule::call_function(method, &evaluated_args);
            }
            if var_name == "cert" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return cert_module::CertModule::call_function(method, &evaluated_args);
            }
            if var_name == "graphql" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return graphql_module::GraphqlModule::call_function(method, &evaluated_args);
            }
            if var_name == "mqtt" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return mqtt_module::MqttModule::call_function(method, &evaluated_args);
            }
            if var_name == "mycli" || var_name == "cli" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return mycli_module::MyCLIModule::call_function(method, &evaluated_args);
            }
            if var_name == "xdbx" || var_name == "debug" {
                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)?); }
                return xdbx_module::XdbxModule::call_function(method, &evaluated_args);
            }
        }
        let obj_val = self.eval(object)?;
        match &obj_val {
            Value::String(s) => self.eval_string_method(s, method, args),
            Value::Array(arr) => self.eval_array_method(arr.clone(), method, args, object),
            Value::Table(map) => self.eval_table_method(map.clone(), method, args, object),
            _ => Err(MintasError::TypeError {
                message: format!("{} has no method '{}'", obj_val.type_name(), method),
                location: Self::default_location(),
            }),
        }
    }
    fn eval_string_method(&mut self, s: &str, method: &str, args: &[Expr]) -> MintasResult<Value> {
        match method {
            "len" => Ok(Value::Number(s.chars().count() as f64)),
            "upper" => Ok(Value::String(s.to_uppercase())),
            "lower" => Ok(Value::String(s.to_lowercase())),
            "trim" => Ok(Value::String(s.trim().to_string())),
            "reverse" => Ok(Value::String(s.chars().rev().collect())),
            "contains" => {
                let sub = self.expect_string_arg(args, 0, "contains")?;
                Ok(Value::Boolean(s.contains(&sub)))
            }
            "startswith" => {
                let prefix = self.expect_string_arg(args, 0, "startswith")?;
                Ok(Value::Boolean(s.starts_with(&prefix)))
            }
            "endswith" => {
                let suffix = self.expect_string_arg(args, 0, "endswith")?;
                Ok(Value::Boolean(s.ends_with(&suffix)))
            }
            "find" => {
                let sub = self.expect_string_arg(args, 0, "find")?;
                match s.find(&sub) {
                    Some(idx) => Ok(Value::Number((idx + 1) as f64)),
                    None => Ok(Value::Number(0.0)),
                }
            }
            "replace" => {
                let old = self.expect_string_arg(args, 0, "replace")?;
                let new = self.expect_string_arg(args, 1, "replace")?;
                Ok(Value::String(s.replace(&old, &new)))
            }
            "split" => {
                let sep = self.expect_string_arg(args, 0, "split")?;
                let parts: Vec<Value> = s.split(&sep).map(|p| Value::String(p.to_string())).collect();
                Ok(Value::Array(parts))
            }
            "slice" => {
                let start = self.expect_number_arg(args, 0, "slice")? as usize;
                let end = self.expect_number_arg(args, 1, "slice")? as usize;
                let chars: Vec<char> = s.chars().collect();
                let start_idx = if start > 0 { start - 1 } else { 0 };
                let slice: String = chars.get(start_idx..end).unwrap_or(&[]).iter().collect();
                Ok(Value::String(slice))
            }
            "insert" => {
                let idx = self.expect_number_arg(args, 0, "insert")? as usize;
                let sub = self.expect_string_arg(args, 1, "insert")?;
                let mut result = s.to_string();
                let insert_idx = if idx > 0 { idx - 1 } else { 0 };
                result.insert_str(insert_idx.min(result.len()), &sub);
                Ok(Value::String(result))
            }
            "remove" => {
                let start = self.expect_number_arg(args, 0, "remove")? as usize;
                let end = self.expect_number_arg(args, 1, "remove")? as usize;
                let chars: Vec<char> = s.chars().collect();
                let start_idx = if start > 0 { start - 1 } else { 0 };
                let mut result = String::new();
                for (i, c) in chars.iter().enumerate() {
                    if i < start_idx || i >= end {
                        result.push(*c);
                    }
                }
                Ok(Value::String(result))
            }
            "removeprefix" => {
                let prefix = self.expect_string_arg(args, 0, "removeprefix")?;
                Ok(Value::String(s.strip_prefix(&prefix).unwrap_or(s).to_string()))
            }
            "removesuffix" => {
                let suffix = self.expect_string_arg(args, 0, "removesuffix")?;
                Ok(Value::String(s.strip_suffix(&suffix).unwrap_or(s).to_string()))
            }
            "addprefix" => {
                let prefix = self.expect_string_arg(args, 0, "addprefix")?;
                Ok(Value::String(format!("{}{}", prefix, s)))
            }
            "addsuffix" => {
                let suffix = self.expect_string_arg(args, 0, "addsuffix")?;
                Ok(Value::String(format!("{}{}", s, suffix)))
            }
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown string method '{}'", method),
                location: Self::default_location(),
            }),
        }
    }
    fn eval_array_method(&mut self, mut arr: Vec<Value>, method: &str, args: &[Expr], object: &Expr) -> MintasResult<Value> {
        match method {
            "len" => Ok(Value::Number(arr.len() as f64)),
            "push" | "append" => {
                let val = self.eval(&args[0])?;
                let element_size = Self::estimate_value_size(&val);
                self.check_memory_limit(element_size)?;
                arr.push(val);
                self.update_array_variable(object, arr.clone())?;
                Ok(Value::Array(arr))
            }
            "pop" => {
                let popped = arr.pop().unwrap_or(Value::Empty);
                self.update_array_variable(object, arr)?;
                Ok(popped)
            }
            "insert" => {
                let idx = self.expect_number_arg(args, 0, "insert")? as usize;
                let val = self.eval(&args[1])?;
                let element_size = Self::estimate_value_size(&val);
                self.check_memory_limit(element_size)?;
                let insert_idx = if idx > 0 { idx - 1 } else { 0 };
                arr.insert(insert_idx.min(arr.len()), val);
                self.update_array_variable(object, arr.clone())?;
                Ok(Value::Array(arr))
            }
            "remove" => {
                let val = self.eval(&args[0])?;
                if let Some(pos) = arr.iter().position(|x| x == &val) {
                    arr.remove(pos);
                }
                self.update_array_variable(object, arr.clone())?;
                Ok(Value::Array(arr))
            }
            "clear" => {
                arr.clear();
                self.update_array_variable(object, arr.clone())?;
                Ok(Value::Array(arr))
            }
            "reverse" => {
                arr.reverse();
                self.update_array_variable(object, arr.clone())?;
                Ok(Value::Array(arr))
            }
            "contains" => {
                let val = self.eval(&args[0])?;
                Ok(Value::Boolean(arr.contains(&val)))
            }
            "index" => {
                let val = self.eval(&args[0])?;
                match arr.iter().position(|x| x == &val) {
                    Some(idx) => Ok(Value::Number((idx + 1) as f64)),
                    None => Ok(Value::Number(0.0)),
                }
            }
            "count" => {
                let val = self.eval(&args[0])?;
                let count = arr.iter().filter(|x| *x == &val).count();
                Ok(Value::Number(count as f64))
            }
            "slice" => {
                let start = self.expect_number_arg(args, 0, "slice")? as usize;
                let end = self.expect_number_arg(args, 1, "slice")? as usize;
                let start_idx = if start > 0 { start - 1 } else { 0 };
                let slice: Vec<Value> = arr.get(start_idx..end).unwrap_or(&[]).to_vec();
                Ok(Value::Array(slice))
            }
            "join" => {
                let sep = self.expect_string_arg(args, 0, "join")?;
                let parts: Vec<String> = arr.iter().map(|v| self.value_to_string(v)).collect();
                Ok(Value::String(parts.join(&sep)))
            }
            "extend" => {
                let other = self.eval(&args[0])?;
                if let Value::Array(other_arr) = other {
                    arr.extend(other_arr);
                    self.update_array_variable(object, arr.clone())?;
                    Ok(Value::Array(arr))
                } else {
                    Err(MintasError::TypeError {
                        message: "extend requires an array argument".to_string(),
                        location: Self::default_location(),
                    })
                }
            }
            "sort" => {
                arr.sort_by(|a, b| {
                    match (a, b) {
                        (Value::Number(n1), Value::Number(n2)) => n1.partial_cmp(n2).unwrap_or(std::cmp::Ordering::Equal),
                        (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
                        _ => std::cmp::Ordering::Equal,
                    }
                });
                self.update_array_variable(object, arr.clone())?;
                Ok(Value::Array(arr))
            }
            "map" => {
                let func_expr = &args[0];
                let func = self.get_function_from_expr(func_expr)?;
                let mut result = Vec::new();
                for (idx, item) in arr.iter().enumerate() {
                    let old_vars = self.variables.clone();
                    if func.params.len() >= 1 {
                        self.variables.insert(func.params[0].clone(), item.clone());
                    }
                    if func.params.len() >= 2 {
                        self.variables.insert(func.params[1].clone(), Value::Number((idx + 1) as f64));
                    }
                    let mapped = self.eval_block(&func.body)?;
                    self.variables = old_vars;
                    result.push(mapped);
                }
                Ok(Value::Array(result))
            }
            "filter" => {
                let func_expr = &args[0];
                let func = self.get_function_from_expr(func_expr)?;
                let mut result = Vec::new();
                for (idx, item) in arr.iter().enumerate() {
                    let old_vars = self.variables.clone();
                    if func.params.len() >= 1 {
                        self.variables.insert(func.params[0].clone(), item.clone());
                    }
                    if func.params.len() >= 2 {
                        self.variables.insert(func.params[1].clone(), Value::Number((idx + 1) as f64));
                    }
                    let filtered = self.eval_block(&func.body)?;
                    self.variables = old_vars;
                    if filtered.is_truthy() {
                        result.push(item.clone());
                    }
                }
                Ok(Value::Array(result))
            }
            "reduce" => {
                let func_expr = &args[0];
                let initial_expr = if args.len() > 1 { Some(&args[1]) } else { None };
                let func = self.get_function_from_expr(func_expr)?;
                let mut accumulator = if let Some(init) = initial_expr {
                    self.eval(init)?
                } else if !arr.is_empty() {
                    arr[0].clone()
                } else {
                    return Err(MintasError::RuntimeError {
                        message: "reduce on empty array requires initial value".to_string(),
                        location: Self::default_location(),
                    });
                };
                let start_idx = if initial_expr.is_some() { 0 } else { 1 };
                for item in arr.iter().skip(start_idx) {
                    let old_vars = self.variables.clone();
                    if func.params.len() >= 1 {
                        self.variables.insert(func.params[0].clone(), accumulator.clone());
                    }
                    if func.params.len() >= 2 {
                        self.variables.insert(func.params[1].clone(), item.clone());
                    }
                    accumulator = self.eval_block(&func.body)?;
                    self.variables = old_vars;
                }
                Ok(accumulator)
            }
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown array method '{}'", method),
                location: Self::default_location(),
            }),
        }
    }
    fn get_function_from_expr(&self, expr: &Expr) -> MintasResult<Function> {
        match expr {
            Expr::Variable(name) => {
                if let Some(func) = self.functions.get(name) {
                    Ok(func.clone())
                } else if let Some(Value::Function(f)) = self.variables.get(name) {
                    Ok(f.as_ref().clone())
                } else {
                    Err(MintasError::TypeError {
                        message: format!("Expected function, got variable '{}'", name),
                        location: Self::default_location(),
                    })
                }
            }
            _ => Err(MintasError::TypeError {
                message: "map/filter/reduce requires a function".to_string(),
                location: Self::default_location(),
            }),
        }
    }
    fn update_array_variable(&mut self, object: &Expr, new_arr: Vec<Value>) -> MintasResult<()> {
        if let Expr::Variable(name) = object {
            if self.constants.contains(name) {
                return Err(MintasError::ConstantReassignment {
                    name: name.clone(),
                    location: Self::default_location(),
                });
            }
            self.variables.insert(name.clone(), Value::Array(new_arr));
        }
        Ok(())
    }
    fn eval_table_method(&mut self, mut map: std::collections::HashMap<String, Value>, method: &str, args: &[Expr], object: &Expr) -> MintasResult<Value> {
        match method {
            "len" => Ok(Value::Number(map.len() as f64)),
            "keys" => {
                let keys: Vec<Value> = map.keys().map(|k| Value::String(k.clone())).collect();
                Ok(Value::Array(keys))
            }
            "values" => {
                let values: Vec<Value> = map.values().cloned().collect();
                Ok(Value::Array(values))
            }
            "has" => {
                let key = self.expect_string_arg(args, 0, "has")?;
                Ok(Value::Boolean(map.contains_key(&key)))
            }
            "remove" => {
                let key = self.expect_string_arg(args, 0, "remove")?;
                let removed = map.remove(&key);
                self.update_table_variable(object, map.clone())?;
                Ok(removed.unwrap_or(Value::Empty))
            }
            "merge" => {
                let other = self.eval(&args[0])?;
                if let Value::Table(other_map) = other {
                    for (k, v) in other_map {
                        map.insert(k, v);
                    }
                    self.update_table_variable(object, map.clone())?;
                    Ok(Value::Table(map))
                } else {
                    Err(MintasError::TypeError {
                        message: "merge requires a table argument".to_string(),
                        location: Self::default_location(),
                    })
                }
            }
            "param" => {
                let key = self.expect_string_arg(args, 0, "param")?;
                if let Some(Value::Table(params)) = map.get("params") {
                    Ok(params.get(&key).cloned().unwrap_or(Value::Empty))
                } else {
                    Ok(Value::Empty)
                }
            }
            "query" => {
                let key = self.expect_string_arg(args, 0, "query")?;
                if let Some(Value::Table(query)) = map.get("query") {
                    Ok(query.get(&key).cloned().unwrap_or(Value::Empty))
                } else {
                    Ok(Value::Empty)
                }
            }
            "header" => {
                let key = self.expect_string_arg(args, 0, "header")?;
                if let Some(Value::Table(headers)) = map.get("headers") {
                    if let Some(v) = headers.get(&key) {
                        Ok(v.clone())
                    } else if let Some(v) = headers.get(&key.to_lowercase()) {
                        Ok(v.clone())
                    } else {
                        Ok(Value::Empty)
                    }
                } else {
                    Ok(Value::Empty)
                }
            }
            "json" => {
                if let Some(Value::String(body)) = map.get("body") {
                    self.parse_json_string(body)
                } else {
                    Ok(Value::Table(HashMap::new()))
                }
            }
            "form" => {
                if let Some(Value::String(body)) = map.get("body") {
                    let mut form_data = HashMap::new();
                    for pair in body.split('&') {
                        let mut parts = pair.splitn(2, '=');
                        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                            form_data.insert(key.to_string(), Value::String(value.to_string()));
                        }
                    }
                    Ok(Value::Table(form_data))
                } else {
                    Ok(Value::Table(HashMap::new()))
                }
            }
            "text" | "body" => {
                Ok(map.get("body").cloned().unwrap_or(Value::String(String::new())))
            }
            "ip" => {
                Ok(map.get("ip").cloned().unwrap_or(Value::String(String::new())))
            }
            "validated" => {
                if let Some(validated_data) = map.get("validated") {
                    Ok(validated_data.clone())
                } else {
                    if let Some(Value::String(body)) = map.get("body") {
                        self.parse_json_string(body)
                    } else {
                        Ok(Value::Table(HashMap::new()))
                    }
                }
            }
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown table method '{}'", method),
                location: Self::default_location(),
            }),
        }
    }
    fn update_table_variable(&mut self, object: &Expr, new_map: std::collections::HashMap<String, Value>) -> MintasResult<()> {
        if let Expr::Variable(name) = object {
            if self.constants.contains(name) {
                return Err(MintasError::ConstantReassignment {
                    name: name.clone(),
                    location: Self::default_location(),
                });
            }
            self.variables.insert(name.clone(), Value::Table(new_map));
        }
        Ok(())
    }
    fn expect_string_arg(&mut self, args: &[Expr], idx: usize, method: &str) -> MintasResult<String> {
        if idx >= args.len() {
            return Err(MintasError::InvalidArgumentCount {
                function: method.to_string(),
                expected: idx + 1,
                got: args.len(),
                location: Self::default_location(),
            });
        }
        match self.eval(&args[idx])? {
            Value::String(s) => Ok(s),
            other => Err(MintasError::TypeError {
                message: format!("{} expects string argument, got {}", method, other.type_name()),
                location: Self::default_location(),
            }),
        }
    }
    fn expect_number_arg(&mut self, args: &[Expr], idx: usize, method: &str) -> MintasResult<f64> {
        if idx >= args.len() {
            return Err(MintasError::InvalidArgumentCount {
                function: method.to_string(),
                expected: idx + 1,
                got: args.len(),
                location: Self::default_location(),
            });
        }
        match self.eval(&args[idx])? {
            Value::Number(n) => Ok(n),
            other => Err(MintasError::TypeError {
                message: format!("{} expects number argument, got {}", method, other.type_name()),
                location: Self::default_location(),
            }),
        }
    }
    fn eval_block(&mut self, statements: &[Expr]) -> MintasResult<Value> {
        let mut last = Value::Empty;
        for stmt in statements {
            let val = self.eval(stmt)?;
            if matches!(val, Value::ReturnSignal(_)) {
                return Ok(val);
            }
            if matches!(val, Value::ExitSignal | Value::ProceedSignal) {
                return Ok(val);
            }
            last = val;
        }
        Ok(last)
    }
    fn eval_binary_op(&mut self, op: &BinaryOp, left: &Expr, right: &Expr) -> MintasResult<Value> {
        let left_val = self.eval(left)?;
        let right_val = self.eval(right)?;
        self.apply_binary_op(op, &left_val, &right_val)
    }
    fn apply_binary_op(&self, op: &BinaryOp, left_val: &Value, right_val: &Value) -> MintasResult<Value> {
        match op {
            BinaryOp::Add => {
                match (left_val, right_val) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                    (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                    (Value::String(a), Value::Number(b)) => Ok(Value::String(format!("{}{}", a, b))),
                    (Value::Number(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                    (Value::String(a), Value::Boolean(b)) => Ok(Value::String(format!("{}{}", a, b))),
                    (Value::Boolean(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                    (Value::String(a), Value::Maybe) => Ok(Value::String(format!("{}maybe", a))),
                    (Value::Maybe, Value::String(b)) => Ok(Value::String(format!("maybe{}", b))),
                    (Value::String(a), Value::Empty) => Ok(Value::String(format!("{}empty", a))),
                    (Value::Empty, Value::String(b)) => Ok(Value::String(format!("empty{}", b))),
                    _ => Err(MintasError::TypeError {
                        message: format!("Cannot add {} and {}", left_val.type_name(), right_val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            BinaryOp::Subtract => {
                match (left_val, right_val) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                    _ => Err(MintasError::TypeError {
                        message: format!("Subtraction only works with numbers, got {} and {}", 
                            left_val.type_name(), right_val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            BinaryOp::Multiply => {
                match (left_val, right_val) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                    _ => Err(MintasError::TypeError {
                        message: format!("Multiplication only works with numbers, got {} and {}", 
                            left_val.type_name(), right_val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            BinaryOp::Divide => {
                match (left_val, right_val) {
                    (Value::Number(a), Value::Number(b)) => {
                        if *b == 0.0 {
                            Err(MintasError::DivisionByZero {
                                location: Self::default_location(),
                            })
                        } else {
                            Ok(Value::Number(a / b))
                        }
                    }
                    _ => Err(MintasError::TypeError {
                        message: format!("Division only works with numbers, got {} and {}", 
                            left_val.type_name(), right_val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            BinaryOp::Modulo => {
                match (left_val, right_val) {
                    (Value::Number(a), Value::Number(b)) => {
                        if *b == 0.0 {
                            Err(MintasError::DivisionByZero {
                                location: Self::default_location(),
                            })
                        } else {
                            Ok(Value::Number(a % b))
                        }
                    }
                    _ => Err(MintasError::TypeError {
                        message: format!("Modulo only works with numbers, got {} and {}", 
                            left_val.type_name(), right_val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            BinaryOp::Exponent => {
                match (left_val, right_val) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.powf(*b))),
                    _ => Err(MintasError::TypeError {
                        message: format!("Exponentiation only works with numbers, got {} and {}", 
                            left_val.type_name(), right_val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            BinaryOp::Equal => {
                Ok(Value::Boolean(self.values_equal(left_val, right_val)))
            }
            BinaryOp::NotEqual => {
                Ok(Value::Boolean(!self.values_equal(left_val, right_val)))
            }
            BinaryOp::Greater => {
                match (left_val, right_val) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
                    (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a > b)),
                    _ => Err(MintasError::TypeError {
                        message: format!("Cannot compare {} and {} with >", 
                            left_val.type_name(), right_val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            BinaryOp::Less => {
                match (left_val, right_val) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
                    (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a < b)),
                    _ => Err(MintasError::TypeError {
                        message: format!("Cannot compare {} and {} with <", 
                            left_val.type_name(), right_val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            BinaryOp::GreaterEqual => {
                match (left_val, right_val) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
                    (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a >= b)),
                    _ => Err(MintasError::TypeError {
                        message: format!("Cannot compare {} and {} with >=", 
                            left_val.type_name(), right_val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            BinaryOp::LessEqual => {
                match (left_val, right_val) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
                    (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a <= b)),
                    _ => Err(MintasError::TypeError {
                        message: format!("Cannot compare {} and {} with <=", 
                            left_val.type_name(), right_val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            BinaryOp::StrictEqual => {
                Ok(Value::Boolean(self.values_strict_equal(left_val, right_val)))
            }
            BinaryOp::StrictNotEqual => {
                Ok(Value::Boolean(!self.values_strict_equal(left_val, right_val)))
            }
            BinaryOp::And => {
                if !left_val.is_truthy() {
                    Ok(left_val.clone())
                } else {
                    Ok(right_val.clone())
                }
            }
            BinaryOp::Or => {
                if left_val.is_truthy() {
                    Ok(left_val.clone())
                } else {
                    Ok(right_val.clone())
                }
            }
        }
    }
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Maybe, Value::Maybe) => true,
            (Value::Empty, Value::Empty) => true,
            (Value::Number(n), Value::String(s)) | (Value::String(s), Value::Number(n)) => {
                s.parse::<f64>().map(|parsed| (parsed - n).abs() < f64::EPSILON).unwrap_or(false)
            }
            _ => false,
        }
    }
    fn values_strict_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Maybe, Value::Maybe) => true,
            (Value::Empty, Value::Empty) => true,
            _ => false,
        }
    }
    fn table_to_json(&self, map: &HashMap<String, Value>) -> String {
        let mut parts = Vec::new();
        for (key, value) in map {
            let val_str = self.value_to_json(value);
            parts.push(format!("\"{}\":{}", key, val_str));
        }
        format!("{{{}}}", parts.join(","))
    }
    fn value_to_json(&self, value: &Value) -> String {
        match value {
            Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Boolean(b) => format!("{}", b),
            Value::Maybe | Value::Empty => "null".to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.value_to_json(v)).collect();
                format!("[{}]", items.join(","))
            }
            Value::Table(map) => self.table_to_json(map),
            _ => "null".to_string(),
        }
    }
    fn parse_json_string(&self, json: &str) -> MintasResult<Value> {
        let json = json.trim();
        if json.is_empty() {
            return Ok(Value::Table(HashMap::new()));
        }
        if json.starts_with('{') {
            self.parse_json_object(json)
        } else if json.starts_with('[') {
            self.parse_json_array(json)
        } else if json.starts_with('"') {
            let s = json.trim_matches('"');
            Ok(Value::String(s.to_string()))
        } else if json == "true" {
            Ok(Value::Boolean(true))
        } else if json == "false" {
            Ok(Value::Boolean(false))
        } else if json == "null" {
            Ok(Value::Empty)
        } else if let Ok(n) = json.parse::<f64>() {
            Ok(Value::Number(n))
        } else {
            Ok(Value::String(json.to_string()))
        }
    }
    fn parse_json_object(&self, json: &str) -> MintasResult<Value> {
        let json = json.trim();
        if !json.starts_with('{') || !json.ends_with('}') {
            return Ok(Value::Table(HashMap::new()));
        }
        let inner = &json[1..json.len()-1].trim();
        if inner.is_empty() {
            return Ok(Value::Table(HashMap::new()));
        }
        let mut map = HashMap::new();
        let mut depth = 0;
        let mut in_string = false;
        let mut escape = false;
        let mut start = 0;
        let chars: Vec<char> = inner.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            if escape {
                escape = false;
                continue;
            }
            if c == '\\' {
                escape = true;
                continue;
            }
            if c == '"' {
                in_string = !in_string;
                continue;
            }
            if in_string {
                continue;
            }
            if c == '{' || c == '[' {
                depth += 1;
            } else if c == '}' || c == ']' {
                depth -= 1;
            } else if c == ',' && depth == 0 {
                let pair: String = chars[start..i].iter().collect();
                self.parse_json_pair(&pair, &mut map);
                start = i + 1;
            }
        }
        if start < chars.len() {
            let pair: String = chars[start..].iter().collect();
            self.parse_json_pair(&pair, &mut map);
        }
        Ok(Value::Table(map))
    }
    fn parse_json_pair(&self, pair: &str, map: &mut HashMap<String, Value>) {
        let pair = pair.trim();
        if let Some(colon_pos) = pair.find(':') {
            let key = pair[..colon_pos].trim().trim_matches('"');
            let value = pair[colon_pos + 1..].trim();
            if let Ok(v) = self.parse_json_string(value) {
                map.insert(key.to_string(), v);
            }
        }
    }
    fn parse_json_array(&self, json: &str) -> MintasResult<Value> {
        let json = json.trim();
        if !json.starts_with('[') || !json.ends_with(']') {
            return Ok(Value::Array(Vec::new()));
        }
        let inner = &json[1..json.len()-1].trim();
        if inner.is_empty() {
            return Ok(Value::Array(Vec::new()));
        }
        let mut arr = Vec::new();
        let mut depth = 0;
        let mut in_string = false;
        let mut escape = false;
        let mut start = 0;
        let chars: Vec<char> = inner.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            if escape {
                escape = false;
                continue;
            }
            if c == '\\' {
                escape = true;
                continue;
            }
            if c == '"' {
                in_string = !in_string;
                continue;
            }
            if in_string {
                continue;
            }
            if c == '{' || c == '[' {
                depth += 1;
            } else if c == '}' || c == ']' {
                depth -= 1;
            } else if c == ',' && depth == 0 {
                let item: String = chars[start..i].iter().collect();
                if let Ok(v) = self.parse_json_string(item.trim()) {
                    arr.push(v);
                }
                start = i + 1;
            }
        }
        if start < chars.len() {
            let item: String = chars[start..].iter().collect();
            if let Ok(v) = self.parse_json_string(item.trim()) {
                arr.push(v);
            }
        }
        Ok(Value::Array(arr))
    }
    fn eval_unary_op(&mut self, op: &UnaryOp, expr: &Expr) -> MintasResult<Value> {
        match op {
            UnaryOp::Negate => {
                match self.eval(expr)? {
                    Value::Number(n) => Ok(Value::Number(-n)),
                    other => Err(MintasError::TypeError {
                        message: format!("Cannot negate {}", other.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            UnaryOp::Not => {
                let val = self.eval(expr)?;
                Ok(Value::Boolean(!val.is_truthy()))
            }
            UnaryOp::Increment => {
                match expr {
                    Expr::Variable(name) => {
                        if self.constants.contains(name) {
                            return Err(MintasError::ConstantReassignment {
                                name: name.clone(),
                                location: Self::default_location(),
                            });
                        }
                        let current = self.variables.get(name).cloned().ok_or_else(|| {
                            MintasError::UndefinedVariable {
                                name: name.clone(),
                                location: Self::default_location(),
                            }
                        })?;
                        match current {
                            Value::Number(n) => {
                                let new_val = Value::Number(n + 1.0);
                                self.variables.insert(name.clone(), new_val.clone());
                                Ok(new_val)
                            }
                            other => Err(MintasError::TypeError {
                                message: format!("Cannot increment {}", other.type_name()),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => Err(MintasError::RuntimeError {
                        message: "Increment can only be applied to variables".to_string(),
                        location: Self::default_location(),
                    }),
                }
            }
            UnaryOp::Decrement => {
                match expr {
                    Expr::Variable(name) => {
                        if self.constants.contains(name) {
                            return Err(MintasError::ConstantReassignment {
                                name: name.clone(),
                                location: Self::default_location(),
                            });
                        }
                        let current = self.variables.get(name).cloned().ok_or_else(|| {
                            MintasError::UndefinedVariable {
                                name: name.clone(),
                                location: Self::default_location(),
                            }
                        })?;
                        match current {
                            Value::Number(n) => {
                                let new_val = Value::Number(n - 1.0);
                                self.variables.insert(name.clone(), new_val.clone());
                                Ok(new_val)
                            }
                            other => Err(MintasError::TypeError {
                                message: format!("Cannot decrement {}", other.type_name()),
                                location: Self::default_location(),
                            }),
                        }
                    }
                    _ => Err(MintasError::RuntimeError {
                        message: "Decrement can only be applied to variables".to_string(),
                        location: Self::default_location(),
                    }),
                }
            }
        }
    }
    fn eval_call(&mut self, name: &str, args: &[Expr]) -> MintasResult<Value> {
        match name {
            "say" => {
                if args.len() != 1 {
                    return Err(MintasError::InvalidArgumentCount {
                        function: "say".to_string(),
                        expected: 1,
                        got: args.len(),
                        location: Self::default_location(),
                    });
                }
                let val = self.eval(&args[0])?;
                {
                    let mut stdout = self.stdout_buffer.borrow_mut();
                    self.write_value_to_buffer(&val, &mut *stdout)?;
                    writeln!(stdout).map_err(|e| MintasError::RuntimeError {
                        message: format!("Output error: {}", e),
                        location: Self::default_location(),
                    })?;
                    if stdout.buffer().len() > 7000 {
                        stdout.flush().map_err(|e| MintasError::RuntimeError {
                            message: format!("Output flush error: {}", e),
                            location: Self::default_location(),
                        })?;
                    }
                }
                Ok(val)
            }
            "ask" => {
                if args.len() != 1 {
                    return Err(MintasError::InvalidArgumentCount {
                        function: "ask".to_string(),
                        expected: 1,
                        got: args.len(),
                        location: Self::default_location(),
                    });
                }
                let prompt_val = self.eval(&args[0])?;
                let mut prompt = self.value_to_string(&prompt_val);
                if prompt.ends_with(':') {
                    prompt.push(' ');
                }
                {
                    let mut stdout = self.stdout_buffer.borrow_mut();
                    write!(stdout, "{}", prompt).map_err(|e| MintasError::RuntimeError {
                        message: format!("Prompt output error: {}", e),
                        location: Self::default_location(),
                    })?;
                    stdout.flush().map_err(|e| MintasError::RuntimeError {
                        message: format!("Prompt flush error: {}", e),
                        location: Self::default_location(),
                    })?;
                }
                let mut input = String::with_capacity(256); 
                {
                    let mut stdin = self.stdin_buffer.borrow_mut();
                    stdin.read_line(&mut input).map_err(|e| {
                        MintasError::RuntimeError {
                            message: format!("Failed to read input: {}", e),
                            location: Self::default_location(),
                        }
                    })?;
                }
                input.truncate(input.trim_end().len()); 
                Ok(Value::String(input))
            }
            "read" => {
                if args.len() != 1 {
                    return Err(MintasError::InvalidArgumentCount {
                        function: "read".to_string(),
                        expected: 1,
                        got: args.len(),
                        location: Self::default_location(),
                    });
                }
                let file_path_val = self.eval(&args[0])?;
                let file_path = match file_path_val {
                    Value::String(s) => s,
                    _ => return Err(MintasError::TypeError {
                        message: "read() expects a string file path".to_string(),
                        location: Self::default_location(),
                    }),
                };
                use std::fs::File;
                use std::io::Read;
                match File::open(&file_path) {
                    Ok(mut file) => {
                        let metadata = file.metadata().map_err(|e| MintasError::RuntimeError {
                            message: format!("Failed to get file metadata '{}': {}", file_path, e),
                            location: Self::default_location(),
                        })?;
                        let file_size = metadata.len() as usize;
                        let mut content = String::with_capacity(file_size); 
                        file.read_to_string(&mut content).map_err(|e| MintasError::RuntimeError {
                            message: format!("Failed to read file '{}': {}", file_path, e),
                            location: Self::default_location(),
                        })?;
                        Ok(Value::String(content))
                    }
                    Err(e) => Err(MintasError::RuntimeError {
                        message: format!("Failed to open file '{}': {}", file_path, e),
                        location: Self::default_location(),
                    }),
                }
            }
            "write" => {
                if args.len() != 2 {
                    return Err(MintasError::InvalidArgumentCount {
                        function: "write".to_string(),
                        expected: 2,
                        got: args.len(),
                        location: Self::default_location(),
                    });
                }
                let file_path_val = self.eval(&args[0])?;
                let content_val = self.eval(&args[1])?;
                let file_path = match file_path_val {
                    Value::String(s) => s,
                    _ => return Err(MintasError::TypeError {
                        message: "write() expects a string file path".to_string(),
                        location: Self::default_location(),
                    }),
                };
                let content = self.value_to_string(&content_val);
                use std::fs::File;
                use std::io::Write;
                match File::create(&file_path) {
                    Ok(file) => {
                        let mut writer = BufWriter::with_capacity(8192, file);
                        writer.write_all(content.as_bytes()).map_err(|e| MintasError::RuntimeError {
                            message: format!("Failed to write to file '{}': {}", file_path, e),
                            location: Self::default_location(),
                        })?;
                        writer.flush().map_err(|e| MintasError::RuntimeError {
                            message: format!("Failed to flush file '{}': {}", file_path, e),
                            location: Self::default_location(),
                        })?;
                        Ok(Value::Empty)
                    }
                    Err(e) => Err(MintasError::RuntimeError {
                        message: format!("Failed to create file '{}': {}", file_path, e),
                        location: Self::default_location(),
                    }),
                }
            }
            "append" => {
                if args.len() != 2 {
                    return Err(MintasError::InvalidArgumentCount {
                        function: "append".to_string(),
                        expected: 2,
                        got: args.len(),
                        location: Self::default_location(),
                    });
                }
                let file_path_val = self.eval(&args[0])?;
                let content_val = self.eval(&args[1])?;
                let file_path = match file_path_val {
                    Value::String(s) => s,
                    _ => return Err(MintasError::TypeError {
                        message: "append() expects a string file path".to_string(),
                        location: Self::default_location(),
                    }),
                };
                let content = self.value_to_string(&content_val);
                use std::fs::OpenOptions;
                use std::io::Write;
                match OpenOptions::new().create(true).append(true).open(&file_path) {
                    Ok(file) => {
                        let mut writer = BufWriter::with_capacity(8192, file);
                        writer.write_all(content.as_bytes()).map_err(|e| MintasError::RuntimeError {
                            message: format!("Failed to append to file '{}': {}", file_path, e),
                            location: Self::default_location(),
                        })?;
                        writer.flush().map_err(|e| MintasError::RuntimeError {
                            message: format!("Failed to flush append to file '{}': {}", file_path, e),
                            location: Self::default_location(),
                        })?;
                        Ok(Value::Empty)
                    }
                    Err(e) => Err(MintasError::RuntimeError {
                        message: format!("Failed to open file '{}' for appending: {}", file_path, e),
                        location: Self::default_location(),
                    }),
                }
            }
            "exists" => {
                if args.len() != 1 {
                    return Err(MintasError::InvalidArgumentCount {
                        function: "exists".to_string(),
                        expected: 1,
                        got: args.len(),
                        location: Self::default_location(),
                    });
                }
                let file_path_val = self.eval(&args[0])?;
                let file_path = match file_path_val {
                    Value::String(s) => s,
                    _ => return Err(MintasError::TypeError {
                        message: "exists() expects a string file path".to_string(),
                        location: Self::default_location(),
                    }),
                };
                Ok(Value::Boolean(std::path::Path::new(&file_path).exists()))
            }
            "typeof" => {
                if args.len() != 1 {
                    return Err(MintasError::InvalidArgumentCount {
                        function: "typeof".to_string(),
                        expected: 1,
                        got: args.len(),
                        location: Self::default_location(),
                    });
                }
                let val = self.eval(&args[0])?;
                Ok(Value::String(val.type_name().to_string()))
            }
            "toString" => {
                if args.len() < 1 || args.len() > 2 {
                    return Err(MintasError::InvalidArgumentCount {
                        function: "toString".to_string(),
                        expected: 1,
                        got: args.len(),
                        location: Self::default_location(),
                    });
                }
                let val = self.eval(&args[0])?;
                let base = if args.len() == 2 {
                    match self.eval(&args[1])? {
                        Value::Number(n) => n as i32,
                        _ => 10,
                    }
                } else {
                    10
                };
                let result = match val {
                    Value::Number(n) => {
                        if base == 10 {
                            n.to_string()
                        } else if base >= 2 && base <= 36 {
                            format!("{:.*}", 0, n as i64)
                        } else {
                            n.to_string()
                        }
                    }
                    _ => self.value_to_string(&val),
                };
                Ok(Value::String(result))
            }
            "toNumber" => {
                if args.len() < 1 || args.len() > 2 {
                    return Err(MintasError::InvalidArgumentCount {
                        function: "toNumber".to_string(),
                        expected: 1,
                        got: args.len(),
                        location: Self::default_location(),
                    });
                }
                let val = self.eval(&args[0])?;
                let base = if args.len() == 2 {
                    match self.eval(&args[1])? {
                        Value::Number(n) => n as i32,
                        _ => 10,
                    }
                } else {
                    10
                };
                match val {
                    Value::Number(n) => Ok(Value::Number(n)),
                    Value::String(s) => {
                        if base == 10 {
                            s.parse::<f64>().map(Value::Number).map_err(|_| {
                                MintasError::TypeError {
                                    message: format!("Cannot convert '{}' to number", s),
                                    location: Self::default_location(),
                                }
                            })
                        } else if base >= 2 && base <= 36 {
                            i64::from_str_radix(&s, base as u32).map(|n| Value::Number(n as f64)).map_err(|_| {
                                MintasError::TypeError {
                                    message: format!("Cannot convert '{}' to number with base {}", s, base),
                                    location: Self::default_location(),
                                }
                            })
                        } else {
                            Err(MintasError::TypeError {
                                message: format!("Invalid base: {}", base),
                                location: Self::default_location(),
                            })
                        }
                    }
                    Value::Boolean(b) => Ok(Value::Number(if b { 1.0 } else { 0.0 })),
                    _ => Err(MintasError::TypeError {
                        message: format!("Cannot convert {} to number", val.type_name()),
                        location: Self::default_location(),
                    }),
                }
            }
            "assert" => {
                if args.len() < 1 || args.len() > 2 {
                    return Err(MintasError::InvalidArgumentCount {
                        function: "assert".to_string(),
                        expected: 1,
                        got: args.len(),
                        location: Self::default_location(),
                    });
                }
                let condition = self.eval(&args[0])?;
                let is_true = condition.is_truthy();
                if !is_true {
                    let message = if args.len() == 2 {
                        let msg_val = self.eval(&args[1])?;
                        self.value_to_string(&msg_val)
                    } else {
                        "Assertion failed".to_string()
                    };
                    return Err(MintasError::RuntimeError {
                        message: format!("Assert failed: {}", message),
                        location: Self::default_location(),
                    });
                }
                Ok(Value::Boolean(true))
            }
            "test" => {
                if args.len() < 2 {
                    return Err(MintasError::InvalidArgumentCount {
                        function: "test".to_string(),
                        expected: 2,
                        got: args.len(),
                        location: Self::default_location(),
                    });
                }
                let test_name_val = self.eval(&args[0])?;
                let test_name = self.value_to_string(&test_name_val);
                let result = self.eval(&args[1]);
                match result {
                    Ok(_) => {
                        println!("✓ Test '{}' passed", test_name);
                        Ok(Value::Boolean(true))
                    }
                    Err(e) => {
                        println!("✗ Test '{}' failed: {}", test_name, e);
                        Ok(Value::Boolean(false))
                    }
                }
            }
            _ => {
                if let Some(dot_pos) = name.find('.') {
                    let module_name = &name[..dot_pos];
                    let func_name = &name[dot_pos + 1..];
                    let mut evaluated_args = Vec::new();
                    for arg in args {
                        evaluated_args.push(self.eval(arg)?);
                    }
                    match module_name {
                        "math" => {
                            return math_module::MathModule::call_function(func_name, &evaluated_args);
                        }
                        #[cfg(feature = "datetime")]
                        "datetime" => {
                            return datetime_module::DateTimeModule::call_function(func_name, &evaluated_args);
                        }
                        #[cfg(not(feature = "datetime"))]
                        "datetime" => {
                            return Err(MintasError::RuntimeError {
                                message: "DateTime module not available. Compile with --features datetime".to_string(),
                                location: Self::default_location(),
                            });
                        }
                        #[cfg(feature = "json")]
                        "json" => {
                            return json_module::JsonModule::call_function(func_name, &evaluated_args);
                        }
                        #[cfg(not(feature = "json"))]
                        "json" => {
                            return Err(MintasError::RuntimeError {
                                message: "JSON module not available. Compile with --features json".to_string(),
                                location: Self::default_location(),
                            });
                        }
                        _ => {
                            return Err(MintasError::UnknownFunction {
                                name: format!("Unknown module '{}'", module_name),
                                location: Self::default_location(),
                            });
                        }
                    }
                }
                let func = if let Some(f) = self.functions.get(name) {
                    f.clone()
                } else if let Some(Value::Function(f)) = self.variables.get(name) {
                    f.as_ref().clone()
                } else {
                    return Err(MintasError::UnknownFunction {
                        name: name.to_string(),
                        location: Self::default_location(),
                    });
                };
                self.check_recursion_limit()?;
                if func.params.len() != args.len() {
                    return Err(MintasError::InvalidArgumentCount {
                        function: name.to_string(),
                        expected: func.params.len(),
                        got: args.len(),
                        location: Self::default_location(),
                    });
                }
                let mut arg_values = Vec::new();
                for arg_expr in args {
                    arg_values.push(self.eval(arg_expr)?);
                }
                let old_vars = self.variables.clone();
                for (param, arg_val) in func.params.iter().zip(arg_values.iter()) {
                    self.variables.insert(param.clone(), arg_val.clone());
                }
                let result = self.eval_block(&func.body);
                self.security_monitor.exit_recursion();
                self.variables = old_vars;
                match result {
                    Ok(Value::ReturnSignal(ret_val)) => Ok(*ret_val),
                    other => other,
                }
            }
        }
    }
    fn interpolate_string(&mut self, s: &str) -> MintasResult<String> {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '$' {
                if let Some(&next) = chars.peek() {
                    if next == '{' {
                        chars.next();
                        let mut expr_str = String::new();
                        let mut depth = 1;
                        while let Some(ch) = chars.next() {
                            if ch == '{' {
                                depth += 1;
                                expr_str.push(ch);
                            } else if ch == '}' {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                } else {
                                    expr_str.push(ch);
                                }
                            } else {
                                expr_str.push(ch);
                            }
                        }
                        if depth > 0 {
                            return Err(MintasError::ParseError {
                                message: "Unclosed ${} in string interpolation".to_string(),
                                location: Self::default_location(),
                            });
                        }
                        let expr = self.parse_interpolation_expr(&expr_str)?;
                        let val = self.eval(&expr)?;
                        result.push_str(&self.value_to_string(&val));
                    } else if next.is_alphabetic() || next == '_' {
                        let mut var_name = String::new();
                        while let Some(&ch) = chars.peek() {
                            if ch.is_alphanumeric() || ch == '_' {
                                var_name.push(ch);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if let Some(val) = self.variables.get(&var_name) {
                            result.push_str(&self.value_to_string(val));
                        } else {
                            return Err(MintasError::UndefinedVariable {
                                name: var_name,
                                location: Self::default_location(),
                            });
                        }
                    } else {
                        result.push('$');
                        result.push(next);
                        chars.next();
                    }
                } else {
                    result.push('$');
                }
            } else {
                result.push(ch);
            }
        }
        Ok(result)
    }
    fn parse_interpolation_expr(&self, s: &str) -> MintasResult<Expr> {
        use crate::lexer::Lexer;
        let mut lexer = Lexer::new(s);
        let tokens = lexer.tokenize()?;
        let mut parser = crate::parser::Parser::new(tokens);
        let exprs = parser.parse()?;
        if exprs.len() == 1 {
            Ok(exprs[0].clone())
        } else if exprs.is_empty() {
            Ok(Expr::Empty)
        } else {
            Err(MintasError::ParseError {
                message: "Interpolation expression must be a single expression".to_string(),
                location: Self::default_location(),
            })
        }
    }
    fn value_to_string(&self, val: &Value) -> String {
        match val {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Maybe => "maybe".to_string(),
            Value::Empty => "empty".to_string(),
            Value::Array(_) => "[array]".to_string(),
            Value::Table(_) => "{table}".to_string(),
            Value::SuperSet(inner) => format!("spr{{{}}}", self.value_to_string(inner)),
            Value::Function(_) => "<function>".to_string(),
            Value::Class(c) => format!("<class:{}>", c.name),
            Value::Instance(i) => format!("<instance:{}>", i.class_name),
            Value::ExitSignal => "exit".to_string(),
            Value::ProceedSignal => "proceed".to_string(),
            Value::ReturnSignal(_) => "return".to_string(),
        }
    }
    pub fn print_value(&self, val: &Value) {
        match val {
            Value::Number(n) => print!("{}", n),
            Value::String(s) => print!("{}", s),
            Value::Boolean(b) => print!("{}", b),
            Value::Maybe => print!("maybe"),
            Value::Empty => print!("empty"),
            Value::Array(arr) => {
                print!("[");
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    match v {
                        Value::Number(n) => print!("{}", n),
                        Value::String(s) => print!("\"{}\"", s),
                        Value::Boolean(b) => print!("{}", b),
                        Value::Maybe => print!("maybe"),
                        Value::Empty => print!("empty"),
                        _ => print!("..."),
                    }
                }
                print!("]");
            }
            Value::Table(map) => {
                print!("{{");
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    print!("\"{}\" = ", k);
                    match v {
                        Value::Number(n) => print!("{}", n),
                        Value::String(s) => print!("\"{}\"", s),
                        Value::Boolean(b) => print!("{}", b),
                        Value::Maybe => print!("maybe"),
                        Value::Empty => print!("empty"),
                        Value::Function(_) => print!("<function>"),
                        _ => print!("..."),
                    }
                }
                print!("}}");
            }
            Value::SuperSet(inner) => {
                print!("spr{{");
                self.print_value(inner);
                print!("}}");
            }
            Value::Function(_) => print!("<function>"),
            Value::Class(c) => print!("<class:{}>", c.name),
            Value::Instance(i) => print!("<instance:{}>", i.class_name),
            Value::ExitSignal | Value::ProceedSignal | Value::ReturnSignal(_) => {}
        }
    }
    pub fn write_value_to_buffer<W: Write>(&self, val: &Value, writer: &mut W) -> MintasResult<()> {
        let result = match val {
            Value::Number(n) => write!(writer, "{}", n),
            Value::String(s) => write!(writer, "{}", s),
            Value::Boolean(b) => write!(writer, "{}", b),
            Value::Maybe => write!(writer, "maybe"),
            Value::Empty => write!(writer, "empty"),
            Value::Array(arr) => {
                write!(writer, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(writer, ", ")?;
                    }
                    match v {
                        Value::Number(n) => write!(writer, "{}", n)?,
                        Value::String(s) => write!(writer, "\"{}\"", s)?,
                        Value::Boolean(b) => write!(writer, "{}", b)?,
                        Value::Maybe => write!(writer, "maybe")?,
                        Value::Empty => write!(writer, "empty")?,
                        _ => write!(writer, "...")?,
                    }
                }
                write!(writer, "]")
            }
            Value::Table(map) => {
                write!(writer, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(writer, ", ")?;
                    }
                    write!(writer, "\"{}\" = ", k)?;
                    match v {
                        Value::Number(n) => write!(writer, "{}", n)?,
                        Value::String(s) => write!(writer, "\"{}\"", s)?,
                        Value::Boolean(b) => write!(writer, "{}", b)?,
                        Value::Maybe => write!(writer, "maybe")?,
                        Value::Empty => write!(writer, "empty")?,
                        Value::Function(_) => write!(writer, "<function>")?,
                        _ => write!(writer, "...")?,
                    }
                }
                write!(writer, "}}")
            }
            Value::SuperSet(inner) => {
                write!(writer, "spr{{")?;
                self.write_value_to_buffer(inner, writer)?;
                write!(writer, "}}")
            }
            Value::Function(_) => write!(writer, "<function>"),
            Value::Class(c) => write!(writer, "<class:{}>", c.name),
            Value::Instance(i) => write!(writer, "<instance:{}>", i.class_name),
            Value::ExitSignal | Value::ProceedSignal | Value::ReturnSignal(_) => Ok(()),
        };
        result.map_err(|e| MintasError::RuntimeError {
            message: format!("Output error: {}", e),
            location: Self::default_location(),
        })
    }
    #[allow(dead_code)]
    pub fn flush_all_buffers(&mut self) -> MintasResult<()> {
        self.stdout_buffer.borrow_mut().flush().map_err(|e| MintasError::RuntimeError {
            message: format!("Failed to flush output buffer: {}", e),
            location: Self::default_location(),
        })?;
        Ok(())
    }
}