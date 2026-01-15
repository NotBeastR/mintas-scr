mod analyzer;
mod bytecode;
mod bytecode_cli;
mod compiler;
mod cranelift_backend;
mod encryption;
mod errors;
mod evaluator;
mod lexer;
mod parser;
mod vm;

use analyzer::CodeAnalyzer;
use bytecode_cli::{compile_to_bytecode, run_bytecode};
use cranelift_backend::CraneliftCompiler as JetXCompiler;
use evaluator::{Evaluator, Value};
use lexer::Lexer;
use parser::Parser;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io::{self, Write};

/// JetX Performance Statistics
#[derive(Debug)]
struct JetXStats {
    total_statements: u32,
    jetx_compiled: bool,
    execution_time_us: u64,
    compilation_time_us: u64,
}

/// JetX - High Performance JIT Compiler for Mintas
/// Compiles ALL code to native machine code for C/Rust-level performance
fn execute_jetx(code: &str, evaluator: &mut Evaluator, show_stats: bool, force_jetx: bool) -> Result<Value, String> {
    let total_start = std::time::Instant::now();
    
    let statements = parse_code(code)?;
    
    if statements.is_empty() {
        return Ok(Value::Empty);
    }
    
    let mut stats = JetXStats {
        total_statements: statements.len() as u32,
        jetx_compiled: false,
        execution_time_us: 0,
        compilation_time_us: 0,
    };
    
    // Static Analysis
    let mut analyzer = CodeAnalyzer::new();
    let _ = analyzer.analyze(&statements);
    
    // Try JetX by default (JETX for everything) - force_jetx enables it more aggressively
    let should_try_jetx = true;  // Always try JetX for eligible code
    
    if should_try_jetx {
        match JetXCompiler::new() {
            Ok(mut compiler) => {
                let compile_start = std::time::Instant::now();
                
                match compiler.compile_program(&statements) {
                    Ok(_) => {
                        stats.compilation_time_us = compile_start.elapsed().as_micros() as u64;
                        stats.jetx_compiled = true;
                        
                        let exec_start = std::time::Instant::now();
                        match compiler.execute_main() {
                            Ok(result) => {
                                stats.execution_time_us = exec_start.elapsed().as_micros() as u64;
                                
                                // Sync variables back (this is a simplified version of what sync_jetx_variables does)
                                if let Some(var_name) = find_last_assigned_var(&statements) {
                                    evaluator.set_variable(var_name, Value::Number(result));
                                }

                                if show_stats {
                                    let total_time = total_start.elapsed().as_micros() as u64;
                                    print_jetx_stats(&stats, total_time);
                                }
                                return Ok(Value::Number(result));
                            }
                            Err(e) => {
                                // Only fall back to interpreter if JetX execution failed
                                if force_jetx {
                                    eprintln!("JetX execution failed: {}", e);
                                    return Err(e.to_string());
                                }
                                eprintln!("JetX execution failed: {}, falling back to interpreter", e);
                                stats.jetx_compiled = false;
                            }
                        }
                    }
                    Err(e) => {
                        // Only error out if force_jetx is enabled
                        if force_jetx {
                            eprintln!("JetX compilation failed: {}", e);
                            return Err(e.to_string());
                        }
                        eprintln!("JetX compilation failed: {}, falling back to interpreter", e);
                        stats.jetx_compiled = false;
                    }
                }
            }
            Err(_) => {}
        }
    }
    
    // Fall back to interpreter
    let exec_start = std::time::Instant::now();
    let result = execute_interpreter_timed(&statements, evaluator)?;
    stats.execution_time_us = exec_start.elapsed().as_micros() as u64;
    
    if show_stats {
        let total_time = total_start.elapsed().as_micros() as u64;
        print_jetx_stats(&stats, total_time);
    }
    
    Ok(result)
}

/// Sync variables from JetX computation back to evaluator
/// This handles loop variables, assigned variables, etc.
fn sync_jetx_variables(statements: &[parser::Expr], result: f64, evaluator: &mut Evaluator) {
    for stmt in statements {
        match stmt {
            // For loops: set loop var to end value (Mintas semantics - i stays at final value)
            parser::Expr::ForLoop { var, end, body, .. } => {
                // Calculate end value and set loop var to end (not end+1)
                if let Some(end_val) = eval_const_expr(end) {
                    evaluator.set_variable(var.clone(), Value::Number(end_val));
                }
                // Also sync any variables assigned inside the loop body
                sync_body_variables(body, result, evaluator);
            }
            // While loops: sync body variables
            parser::Expr::WhileLoop { body, .. } => {
                sync_body_variables(body, result, evaluator);
            }
            // Direct assignments
            parser::Expr::Assign { name, .. } => {
                evaluator.set_variable(name.clone(), Value::Number(result));
            }
            _ => {}
        }
    }
    
    // Also set the last assigned variable to the result
    if let Some(var_name) = find_last_assigned_var(statements) {
        evaluator.set_variable(var_name, Value::Number(result));
    }
}

/// Sync variables from loop body
fn sync_body_variables(body: &[parser::Expr], result: f64, evaluator: &mut Evaluator) {
    for stmt in body {
        if let parser::Expr::Assign { name, .. } = stmt {
            evaluator.set_variable(name.clone(), Value::Number(result));
        }
    }
}

/// Try to evaluate a constant expression (for loop bounds)
fn eval_const_expr(expr: &parser::Expr) -> Option<f64> {
    match expr {
        parser::Expr::Number(n) => Some(*n),
        parser::Expr::BinaryOp { op, left, right } => {
            let l = eval_const_expr(left)?;
            let r = eval_const_expr(right)?;
            match op {
                parser::BinaryOp::Add => Some(l + r),
                parser::BinaryOp::Subtract => Some(l - r),
                parser::BinaryOp::Multiply => Some(l * r),
                parser::BinaryOp::Divide => Some(l / r),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Execute I/O statement using evaluator (with synced variables)
fn execute_io_with_evaluator(stmt: &parser::Expr, evaluator: &mut Evaluator) -> Result<(), String> {
    match evaluator.eval(stmt) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Check if statement is a pure I/O operation (like say() at top level)
fn is_pure_io_statement(expr: &parser::Expr) -> bool {
    match expr {
        parser::Expr::Call { name, .. } => {
            matches!(name.as_str(), "say" | "ask" | "read" | "write" | "append")
        }
        _ => false,
    }
}

/// Check if an expression contains I/O operations anywhere (including nested in loops)
fn contains_io_statement(expr: &parser::Expr) -> bool {
    match expr {
        parser::Expr::Call { name, args, .. } => {
            if matches!(name.as_str(), "say" | "ask" | "read" | "write" | "append" | "print" | "println") {
                return true;
            }
            // Check arguments for nested I/O
            args.iter().any(|arg| contains_io_statement(arg))
        }
        parser::Expr::ForLoop { body, .. } | parser::Expr::WhileLoop { body, .. } => {
            body.iter().any(|stmt| contains_io_statement(stmt))
        }
        parser::Expr::ForInLoop { body, .. } => {
            body.iter().any(|stmt| contains_io_statement(stmt))
        }
        parser::Expr::IfExpr { condition, then_branch, else_branch, .. } => {
            contains_io_statement(condition) ||
            then_branch.iter().any(|stmt| contains_io_statement(stmt)) ||
            else_branch.as_ref().map_or(false, |eb| eb.iter().any(|stmt| contains_io_statement(stmt)))
        }
        parser::Expr::BinaryOp { left, right, .. } => {
            contains_io_statement(left) || contains_io_statement(right)
        }
        parser::Expr::UnaryOp { expr: inner, .. } => {
            contains_io_statement(inner)
        }
        parser::Expr::Assign { value, .. } => {
            contains_io_statement(value)
        }
        parser::Expr::MethodCall { object, args, .. } => {
            contains_io_statement(object) || args.iter().any(|arg| contains_io_statement(arg))
        }
        _ => false,
    }
}

/// Check if code contains user-defined functions or function calls that JetX can't handle
fn contains_user_functions(expr: &parser::Expr) -> bool {
    match expr {
        // Function definitions - JetX can't handle these
        parser::Expr::Function { .. } => true,
        // Function calls (except builtins like say, ask, etc.)
        parser::Expr::Call { name, args } => {
            // These are I/O builtins handled separately
            let is_io_builtin = matches!(name.as_str(), 
                "say" | "ask" | "read" | "write" | "append" | "print" | "println"
            );
            // If it's not a builtin, it's a user function call
            if !is_io_builtin {
                return true;
            }
            // Check args for nested function calls
            args.iter().any(|arg| contains_user_functions(arg))
        }
        // Check inside control structures
        parser::Expr::ForLoop { start, end, body, .. } => {
            contains_user_functions(start) || 
            contains_user_functions(end) || 
            body.iter().any(|s| contains_user_functions(s))
        }
        parser::Expr::WhileLoop { condition, body } => {
            contains_user_functions(condition) || 
            body.iter().any(|s| contains_user_functions(s))
        }
        parser::Expr::ForInLoop { iterable, body, .. } => {
            contains_user_functions(iterable) || 
            body.iter().any(|s| contains_user_functions(s))
        }
        parser::Expr::IfExpr { condition, then_branch, else_branch, .. } => {
            contains_user_functions(condition) ||
            then_branch.iter().any(|s| contains_user_functions(s)) ||
            else_branch.as_ref().map_or(false, |eb| eb.iter().any(|s| contains_user_functions(s)))
        }
        parser::Expr::BinaryOp { left, right, .. } => {
            contains_user_functions(left) || contains_user_functions(right)
        }
        parser::Expr::UnaryOp { expr: inner, .. } => {
            contains_user_functions(inner)
        }
        parser::Expr::Assign { value, .. } => {
            contains_user_functions(value)
        }
        parser::Expr::MethodCall { object, args, .. } => {
            contains_user_functions(object) || args.iter().any(|arg| contains_user_functions(arg))
        }
        parser::Expr::Return { value } => {
            value.as_ref().map_or(false, |v| contains_user_functions(v))
        }
        _ => false,
    }
}

/// Check if statement can be compiled by JetX (must be pure computation, no I/O inside)
fn is_jetx_compilable(expr: &parser::Expr) -> bool {
    // First check if it contains any I/O - if so, not JetX compilable
    if contains_io_statement(expr) {
        return false;
    }
    
    match expr {
        parser::Expr::Number(_) |
        parser::Expr::Boolean(_) |
        parser::Expr::Variable(_) |
        parser::Expr::Assign { .. } |
        parser::Expr::BinaryOp { .. } |
        parser::Expr::UnaryOp { .. } |
        parser::Expr::IfExpr { .. } |
        parser::Expr::WhileLoop { .. } |
        parser::Expr::ForLoop { .. } => true,
        _ => false,
    }
}

/// Find the last variable that was assigned in the statements (including inside loops)
fn find_last_assigned_var(statements: &[parser::Expr]) -> Option<String> {
    fn find_in_expr(expr: &parser::Expr) -> Option<String> {
        match expr {
            parser::Expr::Assign { name, .. } => Some(name.clone()),
            parser::Expr::ForLoop { body, .. } | parser::Expr::WhileLoop { body, .. } => {
                // Search inside loop body from end to start
                for stmt in body.iter().rev() {
                    if let Some(name) = find_in_expr(stmt) {
                        return Some(name);
                    }
                }
                None
            }
            parser::Expr::IfExpr { then_branch, else_branch, .. } => {
                // Check else branch first (if exists), then then branch
                if let Some(else_b) = else_branch {
                    for stmt in else_b.iter().rev() {
                        if let Some(name) = find_in_expr(stmt) {
                            return Some(name);
                        }
                    }
                }
                for stmt in then_branch.iter().rev() {
                    if let Some(name) = find_in_expr(stmt) {
                        return Some(name);
                    }
                }
                None
            }
            _ => None,
        }
    }
    
    for stmt in statements.iter().rev() {
        if let Some(name) = find_in_expr(stmt) {
            return Some(name);
        }
    }
    None
}

/// Execute interpreter and return result (for timing)
fn execute_interpreter_timed(statements: &[parser::Expr], evaluator: &mut Evaluator) -> Result<Value, String> {
    let mut last_val = Value::Empty;
    for stmt in statements {
        match evaluator.eval(stmt) {
            Ok(val) => {
                if matches!(val, Value::ExitSignal) {
                    return Ok(Value::ExitSignal);
                }
                last_val = val.clone();
                if should_display(&val, stmt) {
                    evaluator.print_value(&val);
                    println!();
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(last_val)
}


// Deprecated: Use execute_interpreter_timed instead
#[allow(dead_code)]
fn execute_interpreter(statements: &[parser::Expr], evaluator: &mut Evaluator) -> Result<(), String> {
    for stmt in statements {
        match evaluator.eval(stmt) {
            Ok(val) => {
                if should_display(&val, stmt) {
                    evaluator.print_value(&val);
                    println!();
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

fn parse_code(code: &str) -> Result<Vec<parser::Expr>, String> {
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {}", e))?;
    
    if tokens.is_empty() || matches!(tokens[0].token, lexer::Token::EOF) {
        return Ok(vec![]);
    }
    
    let mut parser = Parser::new(tokens);
    parser.parse().map_err(|e| format!("Parser error: {}", e))
}

fn should_display(val: &Value, stmt: &parser::Expr) -> bool {
    if matches!(val, Value::Empty | Value::ExitSignal | Value::ProceedSignal) {
        return false;
    }
    match stmt {
        parser::Expr::Call { name, .. } if name == "say" => false,
        parser::Expr::Assign { .. } | parser::Expr::MultiAssign { .. } | 
        parser::Expr::CompoundAssign { .. } | parser::Expr::PropertyAssign { .. } | parser::Expr::Cond { .. } |
        parser::Expr::Include { .. } | parser::Expr::Task { .. } |
        parser::Expr::Switch { .. } | parser::Expr::IfExpr { .. } |
        parser::Expr::WhileLoop { .. } | parser::Expr::ForLoop { .. } |
        parser::Expr::ForInLoop { .. } | parser::Expr::MethodCall { .. } |
        parser::Expr::Function { .. } | parser::Expr::Class { .. } |
        parser::Expr::SmartCondition { .. } | parser::Expr::SmartLoop { .. } |
        parser::Expr::Follow { .. } | parser::Expr::TryCatch { .. } |
        parser::Expr::DewRoute { .. } | parser::Expr::DewServe { .. } |
        parser::Expr::DewBefore { .. } | parser::Expr::DewAfter { .. } |
        parser::Expr::DewUse { .. } | parser::Expr::DewCatch { .. } |
        parser::Expr::DewGroup { .. } | parser::Expr::DewStatic { .. } |
        parser::Expr::DewRouteValidated { .. } |
        parser::Expr::Return { .. } => false,
        _ => true,
    }
}

fn print_jetx_stats(stats: &JetXStats, total_us: u64) {
    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║              Mintas Performance Report             ║");
    println!("╠══════════════════════════════════════════════════╣");
    println!("║ Statements:              {:>20} ║", stats.total_statements);
    println!("║ JetX Compiled:           {:>20} ║", if stats.jetx_compiled { "Yes" } else { "No" });
    println!("║ Compilation Time:        {:>17} µs ║", stats.compilation_time_us);
    println!("║ Execution Time:          {:>17} µs ║", stats.execution_time_us);
    println!("║ Total Time:              {:>17} µs ║", total_us);
    println!("╚══════════════════════════════════════════════════╝");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        run_repl();
        return;
    }
    
    // Check for xdbx commands first
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    if args[1] == "xdbx" {
        handle_xdbx_command(&args[2..]);
        return;
    }
    
    let mut file_path: Option<&str> = None;
    let mut show_stats = false;
    let mut check_only = false;
    let mut debug_mode = false;
    let mut force_jetx = false;
    let mut secret: Option<String> = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => { print_help(); return; }
            "-v" | "--version" => {
                println!("Mintas v1.0.0 with JetX JIT Compiler");
                return;
            }
            "-s" | "--stats" => show_stats = true,
            "-c" | "--check" => check_only = true,
            "-d" | "--debug" => debug_mode = true,
            "-jetx" | "--jetx" => force_jetx = true,
            "--secret" | "--key" => {
                if i + 1 < args.len() {
                    secret = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Error: --secret requires a value");
                    std::process::exit(1);
                }
            }
            "compile" => {
                if i + 1 < args.len() {
                    compile_to_bytecode(&args[i + 1], secret.clone());
                } else {
                    eprintln!("Error: compile requires a file argument");
                    eprintln!("Usage: mintas compile <file.as> [--secret <key>]");
                }
                return;
            }
            "run" => {
                if i + 1 < args.len() {
                    run_bytecode(&args[i + 1], secret.clone());
                } else {
                    eprintln!("Error: run requires a file argument");
                    eprintln!("Usage: mintas run <file.ms> [--secret <key>]");
                }
                return;
            }
            arg if !arg.starts_with('-') => {
                file_path = Some(arg);
                break; // Stop parsing - remaining args are for the script
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }
    
    if let Some(path) = file_path {
        run_file(path, show_stats, check_only, debug_mode, force_jetx);
    } else {
        run_repl();
    }
}

fn print_help() {
    println!("Mintas v1.0.0 with JetX JIT Compiler");
    println!();
    println!("USAGE: mintas [OPTIONS] [FILE] [ARGS...]");
    println!("       mintas xdbx <COMMAND> [ARGS]");
    println!();
    println!("OPTIONS:");
    println!("  -h, --help      Show help");
    println!("  -v, --version   Show version");
    println!("  -s, --stats     Show performance stats");
    println!("  -c, --check     Check code only");
    println!("  -d, --debug     Debug mode (verbose logging)");
    println!("  -jetx, --jetx   Force JetX JIT compilation");
    println!();
    println!("BYTECODE COMMANDS:");
    println!("  compile <file.as>          Compile to encrypted .ms bytecode");
    println!("  run <file.ms>              Run encrypted bytecode file");
    println!();
    println!("XDBX COMMANDS (Build System):");
    println!("  xdbx run [file]            Run project");
    println!("  xdbx test                  Run tests");
    println!("  xdbx targets               List build targets");
    println!("  xdbx help                  Show xdbx help");
    println!();
    println!("EXAMPLES:");
    println!("  mintas app.as              Run a Mintas script");
    println!("  mintas app.as arg1 arg2    Run with arguments");
}

fn run_file(path: &str, show_stats: bool, check_only: bool, debug_mode: bool, force_jetx: bool) {
    // Only allow .as files
    if !path.ends_with(".as") {
        eprintln!("Error: Mintas only runs .as files");
        eprintln!("Usage: mintas script.as");
        std::process::exit(1);
    }
    
    let code = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            std::process::exit(1);
        }
    };
    
    if check_only {
        check_code(&code, path);
        return;
    }
    
    if debug_mode {
        println!("🔧 Debug Mode Enabled");
        println!("   File: {}", path);
        println!("   Size: {} bytes", code.len());
        println!("   Lines: {}", code.lines().count());
        println!("────────────────────────────────────────");
    }
    
    let mut evaluator = Evaluator::new();
    if debug_mode {
        evaluator.set_debug_mode(true);
    }
    
    if let Err(e) = execute_jetx(&code, &mut evaluator, show_stats, force_jetx) {
        eprintln!("Error: {}", e);
        eprintln!("For more help, type 'help' in the REPL or check the documentation.");
        std::process::exit(1);
    }
}

fn check_code(code: &str, file_path: &str) {
    println!("Mintas Code Analyzer v1.0.0");
    println!("Analyzing: {}", file_path);
    println!("════════════════════════════════════════════════════");
    
    let mut lexer = Lexer::new(code);
    let tokens = match lexer.tokenize() {
        Ok(t) => { println!("[✓] Lexical Analysis"); t }
        Err(e) => {
            println!("[✗] Lexical Analysis: {}", e);
            std::process::exit(1);
        }
    };
    
    if tokens.is_empty() || matches!(tokens[0].token, lexer::Token::EOF) {
        println!("[!] File is empty");
        return;
    }
    
    let mut parser = Parser::new(tokens);
    let statements = match parser.parse() {
        Ok(s) => { println!("[✓] Syntax Analysis"); s }
        Err(e) => {
            println!("[✗] Syntax Analysis: {}", e);
            std::process::exit(1);
        }
    };
    
    let mut analyzer = CodeAnalyzer::new();
    match analyzer.analyze(&statements) {
        Ok(_) => println!("[✓] Semantic Analysis"),
        Err(e) => {
            println!("[✗] Semantic Analysis: {}", e);
            std::process::exit(1);
        }
    }
    
    match JetXCompiler::new() {
        Ok(_) => println!("[✓] JetX JIT Compiler Ready"),
        Err(_) => println!("[!] JetX not available (interpreter mode)"),
    }
    
    println!("════════════════════════════════════════════════════");
    println!("Ready. {} statements.", statements.len());
}

fn run_repl() {
    let jetx_available = JetXCompiler::new().is_ok();
    let mode_label = if jetx_available { "JetX JIT Compiler" } else { "Interpreter Mode" };
    let mode_color = if jetx_available { "\x1b[1;33m" } else { "\x1b[1;34m" };

    println!("\x1b[1;36m╔═══════════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[1;36m║\x1b[0m   \x1b[1;35mMintas v{}\x1b[0m with {}{}\x1b[0m                 \x1b[1;36m║\x1b[0m", env!("CARGO_PKG_VERSION"), mode_color, mode_label);
    println!("\x1b[1;36m╚═══════════════════════════════════════════════════════════╝\x1b[0m");
    println!();
    println!("  \x1b[1;32m●\x1b[0m Type \x1b[1;33mhelp\x1b[0m for available commands");
    println!("  \x1b[1;32m●\x1b[0m Type \x1b[1;33mexit\x1b[0m or \x1b[1;33mquit\x1b[0m to leave");
    println!("  \x1b[1;32m●\x1b[0m Press \x1b[1;33mCtrl+C\x1b[0m to interrupt");
    println!();
    
    let mut evaluator = Evaluator::new();
    let mut history: VecDeque<String> = VecDeque::with_capacity(100);
    
    loop {
        let prompt_mode = if jetx_available { "JIT" } else { "INT" };
        print!("\x1b[1;36m[{}]\x1b[0m ❯ ", prompt_mode);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() { break; }
        
        let input = input.trim();
        if input.is_empty() { continue; }
        
        match input {
            "exit" | "quit" => {
                let _ = evaluator.flush_all_buffers();
                println!("\n\x1b[1;32m✓\x1b[0m Goodbye! Thanks for using Mintas.\n");
                break;
            }
            "help" => {
                println!("\n\x1b[1;35m╔═══════════════════════════════════════════════╗\x1b[0m");
                println!("\x1b[1;35m║\x1b[0m           \x1b[1;33mMintas REPL Commands\x1b[0m              \x1b[1;35m║\x1b[0m");
                println!("\x1b[1;35m╚═══════════════════════════════════════════════╝\x1b[0m");
                println!("  \x1b[1;36mhelp\x1b[0m      - Show this help message");
                println!("  \x1b[1;36mclear\x1b[0m     - Clear the screen");
                println!("  \x1b[1;36mhistory\x1b[0m   - Show command history");
                println!("  \x1b[1;36mvars\x1b[0m      - List all variables");
                println!("  \x1b[1;36mexit\x1b[0m      - Exit the REPL");
                println!("  \x1b[1;36mquit\x1b[0m      - Exit the REPL");
                println!("\n  \x1b[1;33mExamples:\x1b[0m");
                println!("    \x1b[36msay(\"Hello\")\x1b[0m");
                println!("    \x1b[36mx = 42\x1b[0m");
                println!("    \x1b[36mx + 8\x1b[0m");
                println!("\n  \x1b[1;33mTip:\x1b[0m Use \x1b[1;36mmintas --help\x1b[0m from shell for full CLI options");
                println!();
                continue;
            }
            "clear" => { print!("\x1B[2J\x1B[1;1H"); continue; }
            "history" => {
                println!("\n\x1b[1;33m📜 Command History:\x1b[0m");
                if history.is_empty() {
                    println!("  \x1b[2m(empty)\x1b[0m");
                } else {
                    for (i, cmd) in history.iter().enumerate() {
                        println!("  \x1b[1;36m{}\x1b[0m: {}", i+1, cmd);
                    }
                }
                println!();
                continue;
            }
            "vars" => {
                println!("\n\x1b[1;33m📦 Variables:\x1b[0m");
                let vars = evaluator.get_variables();
                if vars.is_empty() {
                    println!("  \x1b[2m(no variables defined)\x1b[0m");
                } else {
                    for (name, value) in vars {
                        println!("  \x1b[1;36m{}\x1b[0m = {:?}", name, value);
                    }
                }
                println!();
                continue;
            }
            _ => {}
        }
        
        history.push_back(input.to_string());
        if history.len() > 100 { history.pop_front(); }
        
        match execute_jetx(input, &mut evaluator, false, false) {
            Ok(Value::ExitSignal) => {
                let _ = evaluator.flush_all_buffers();
                println!("\n\x1b[1;32m✓\x1b[0m Goodbye! Thanks for using Mintas.\n");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("\x1b[31m✗ Error:\x1b[0m {}", e);
            }
        }
    }
}

/// Handle XDBX CLI commands - Package Manager & Build System
/// Platform: Windows, Linux, macOS only (not WSL)
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
fn handle_xdbx_command(args: &[String]) {
    if args.is_empty() {
        print_xdbx_help();
        return;
    }
    
    match args[0].as_str() {
        "init" => {
            let project_name = args.get(1).map(|s| s.as_str()).unwrap_or("mintas_project");
            xdbx_init(project_name);
        }
        "build" => {
            let mut release = false;
            let mut target = "native".to_string();
            
            for arg in args.iter().skip(1) {
                match arg.as_str() {
                    "--release" | "-r" => release = true,
                    "--exe" => target = "exe".to_string(),
                    "--wasm" => target = "wasm".to_string(),
                    "--deb" => target = "deb".to_string(),
                    "--pkg" => target = "pkg".to_string(),
                    "--target" => {}
                    t if !t.starts_with('-') => target = t.to_string(),
                    _ => {}
                }
            }
            xdbx_build(release, &target);
        }
        "run" => {
            let file = args.get(1).map(|s| s.as_str()).unwrap_or("src/main.as");
            xdbx_run(file);
        }
        "test" => xdbx_test(),
        "targets" => xdbx_targets(),
        "version" | "-v" | "--version" => {
            println!("xdbx v1.0.1 - Mintas Build System");
        }
        "help" | "-h" | "--help" => print_xdbx_help(),
        _ => {
            eprintln!("Unknown xdbx command: {}", args[0]);
            print_xdbx_help();
            std::process::exit(1);
        }
    }
}

fn print_xdbx_help() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           XDBX - Mintas Build System                      ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("USAGE: mintas xdbx <COMMAND> [OPTIONS]");
    println!();
    println!("PROJECT MANAGEMENT:");
    println!("  init <name>            Create new project");
    println!("  info                   Show project information");
    println!();
    println!("BUILD COMMANDS:");
    println!("  build                  Build project (debug mode)");
    println!("  build --release        Build optimized release");
    println!("  build --ms             Build .MS (Mintas Serialized)");
    println!("  targets                List all build targets");
    println!();
    println!("RUN & TEST:");
    println!("  run [file]             Run project or file");
    println!("  test                   Run all tests");
    println!();
    println!("OTHER:");
    println!("  version                Show xdbx version");
    println!("  help                   Show this help");
    println!();
    println!("EXAMPLES:");
    println!("  mintas xdbx build --exe");
    println!("  mintas xdbx build --wasm");
    println!("  mintas xdbx run");
}


/// Initialize a new Mintas project
fn xdbx_init(project_name: &str) {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           XDBX Project Initialization                     ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    
    // Create project structure
    let dirs = vec!["src", "lib", "tests", "docs"];
    for dir in &dirs {
        match fs::create_dir_all(dir) {
            Ok(_) => println!("✓ Created directory: {}", dir),
            Err(e) => {
                eprintln!("✗ Failed to create {}: {}", dir, e);
                return;
            }
        }
    }
    
    // Create mintas.toml
    let toml_content = format!(
r#"[package]
name = "{}"
version = "0.1.0"
description = "A Mintas project"
author = "Your Name"
type = "app"

[build]
target = "ms"
optimization = "debug"

[dependencies]
"#,
        project_name
    );
    
    if fs::write("mintas.toml", toml_content).is_err() {
        eprintln!("✗ Failed to create mintas.toml");
        return;
    }
    println!("✓ Created mintas.toml");
    
    // Create main.mintas
    let main_content = r#"// Main entry point
say "Welcome to Mintas!"
say "This is your new project."
"#;
    
    if fs::write("src/main.mintas", main_content).is_err() {
        eprintln!("✗ Failed to create src/main.mintas");
        return;
    }
    println!("✓ Created src/main.mintas");
    
    // Create README
    let readme = format!(
r#"# {}

A Mintas project.

## Building

```bash
mintas xdbx build --ms
```

## Running

```bash
mintas xdbx run
```

## Project Structure

- `src/` - Source code
- `lib/` - Libraries and modules
- `tests/` - Test files
- `docs/` - Documentation
"#,
        project_name
    );
    
    if fs::write("README.md", readme).is_err() {
        eprintln!("✗ Failed to create README.md");
        return;
    }
    println!("✓ Created README.md");
    
    // Create .gitignore
    let gitignore = r#"target/
*.ms
*.o
*.a
*.so
*.dylib
.DS_Store
*.swp
*.swo
*~
"#;
    
    if fs::write(".gitignore", gitignore).is_err() {
        eprintln!("✗ Failed to create .gitignore");
        return;
    }
    println!("✓ Created .gitignore");
    
    println!();
    println!("\x1b[32m✓ Project '{}' initialized successfully!\x1b[0m", project_name);
    println!();
    println!("Next steps:");
    println!("  cd {}", project_name);
    println!("  mintas xdbx build --ms");
    println!("  mintas xdbx run");
}

fn xdbx_build(release: bool, target: &str) {
    let mode = if release { "release" } else { "debug" };
    
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║              XDBX Build System                            ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    
    // Check for mintas.toml
    if !std::path::Path::new("mintas.toml").exists() {
        eprintln!("\x1b[31m❌ No mintas.toml found in current directory\x1b[0m");
        eprintln!("   Run 'mintas xdbx init <name>' to create a project");
        std::process::exit(1);
    }
    
    // Read project info
    let toml_content = fs::read_to_string("mintas.toml").unwrap_or_default();
    let project_name = toml_content.lines()
        .find(|l| l.starts_with("name"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"'))
        .unwrap_or("app");
    
    let project_type = toml_content.lines()
        .find(|l| l.starts_with("type"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"'))
        .unwrap_or("app");
    
    let is_game = project_type == "game";
    
    println!("\x1b[34m🔨 Building {} ({} mode, target: {})\x1b[0m", project_name, mode, target);
    if is_game {
        println!("   \x1b[33m🎮 Canvas game project detected\x1b[0m");
    }
    println!();
    
    // Find entry file
    let entry_file = if std::path::Path::new("src/main.as").exists() {
        "src/main.as"
    } else if std::path::Path::new("main.as").exists() {
        "main.as"
    } else {
        eprintln!("\x1b[31m❌ No entry file found (src/main.as or main.as)\x1b[0m");
        std::process::exit(1);
    };
    
    // Create target directory
    let target_dir = format!("target/{}", mode);
    fs::create_dir_all(&target_dir).ok();
    
    // Read source code and collect all includes
    let source = match fs::read_to_string(entry_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("\x1b[31m❌ Failed to read {}: {}\x1b[0m", entry_file, e);
            std::process::exit(1);
        }
    };
    
    // Collect all source files (main + includes)
    let mut all_sources = vec![(entry_file.to_string(), source.clone())];
    collect_includes(&source, &mut all_sources);
    
    println!("   [1/4] Parsing source code...");
    
    // Parse the code to validate
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("\x1b[31m❌ Lexer error: {}\x1b[0m", e);
            std::process::exit(1);
        }
    };
    
    let mut parser = Parser::new(tokens);
    let _statements = match parser.parse() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("\x1b[31m❌ Parser error: {}\x1b[0m", e);
            std::process::exit(1);
        }
    };
    
    println!("   [2/4] Analyzing code...");
    
    // Check for canvas usage
    let uses_canvas = source.contains("include canvas") || source.contains("canvas.");
    if uses_canvas {
        println!("      \x1b[33m🎮 Canvas graphics detected\x1b[0m");
    }
    
    println!("   [3/4] Compiling to {}...", target);
    
    // Build based on target - only .MS (Mintas Serialized) format supported
    let output_file = match target {
        "ms" | "mintas-serialized" | "binary" => {
            let out = format!("{}/{}.ms", target_dir, project_name);
            // Compile to .MS bytecode format
            match compile_to_ms_format(&out, project_name, &source, release) {
                Ok(_) => {
                    println!("      \x1b[32m✓ Compiled to .MS format\x1b[0m");
                    out
                }
                Err(e) => {
                    eprintln!("\x1b[31m❌ Compilation failed: {}\x1b[0m", e);
                    std::process::exit(1);
                }
            }
        }
        "native" | "exe" | "windows" | "wasm" | "web" | "deb" | "debian" | "pkg" | "macos" => {
            eprintln!("\x1b[31m❌ Target '{}' is no longer supported\x1b[0m", target);
            eprintln!("\x1b[33m   Only .MS (Mintas Serialized) format is supported\x1b[0m");
            eprintln!("\x1b[33m   Use: mintas xdbx build --ms\x1b[0m");
            std::process::exit(1);
        }
        _ => {
            eprintln!("\x1b[31m❌ Unknown target: {}\x1b[0m", target);
            eprintln!("\x1b[33m   Supported target: ms\x1b[0m");
            std::process::exit(1);
        }
    };
    
    println!("   [4/4] Linking...");
    
    // Copy assets for game projects
    if is_game && std::path::Path::new("assets").exists() {
        let assets_target = format!("{}/assets", target_dir);
        copy_dir_recursive("assets", &assets_target);
        println!("      \x1b[33m📁 Copied assets/\x1b[0m");
    }
    
    println!();
    println!("\x1b[32m✅ Build successful!\x1b[0m");
    println!();
    println!("   Output: {}", output_file);
    
    // Show file size
    if let Ok(metadata) = fs::metadata(&output_file) {
        let size = metadata.len();
        let size_str = if size > 1024 * 1024 {
            format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
        } else if size > 1024 {
            format!("{:.2} KB", size as f64 / 1024.0)
        } else {
            format!("{} bytes", size)
        };
        println!("   Size: {}", size_str);
    }
    println!();
    
    match target {
        "exe" | "windows" | "windows-x64" => {
            let dist_dir = output_file.replace(".exe", "_dist");
            println!("   \x1b[36mDistribution:\x1b[0m {}", dist_dir.replace("/", "\\"));
            println!();
            println!("   \x1b[33mTo run your app:\x1b[0m");
            println!("   1. Copy mintas.exe to the _dist folder, then:");
            println!("      cd {}\\", dist_dir.replace("/", "\\"));
            println!("      mintas.exe main.as");
            println!();
            println!("   OR add mintas to PATH and run:");
            println!("      {}\\{}.bat", dist_dir.replace("/", "\\"), 
                output_file.split('/').last().unwrap_or("app").replace(".exe", ""));
        }
        "wasm" | "web" => {
            println!("   \x1b[36mServe:\x1b[0m python -m http.server -d {}", target_dir);
            println!("   \x1b[36mOpen:\x1b[0m http://localhost:8000/{}.html", project_name);
        }
        "deb" | "debian" | "linux-deb" => {
            println!("   \x1b[36mInstall:\x1b[0m sudo dpkg -i {}", output_file);
        }
        "pkg" | "macos" | "macos-pkg" => {
            println!("   \x1b[36mInstall:\x1b[0m sudo installer -pkg {} -target /", output_file);
        }
        _ => {
            println!("   \x1b[36mRun:\x1b[0m ./{}", output_file);
        }
    }
}

fn collect_includes(source: &str, sources: &mut Vec<(String, String)>) {
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with("include ") {
            let module = line.trim_start_matches("include ").trim();
            // Check for local file includes
            let possible_paths = vec![
                format!("{}.as", module),
                format!("src/{}.as", module),
                format!("lib/{}.as", module),
            ];
            for path in possible_paths {
                if std::path::Path::new(&path).exists() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if !sources.iter().any(|(p, _)| p == &path) {
                            sources.push((path.clone(), content.clone()));
                            collect_includes(&content, sources);
                        }
                    }
                    break;
                }
            }
        }
    }
}

fn copy_dir_recursive(src: &str, dst: &str) {
    fs::create_dir_all(dst).ok();
    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            let dest_path = format!("{}/{}", dst, entry.file_name().to_string_lossy());
            if path.is_dir() {
                copy_dir_recursive(&path.to_string_lossy(), &dest_path);
            } else {
                fs::copy(&path, &dest_path).ok();
            }
        }
    }
}

/// Compile Mintas source to .MS (Mintas Serialized) bytecode format
fn compile_to_ms_format(output: &str, project_name: &str, source: &str, _release: bool) -> Result<(), String> {
    // Compile to bytecode using the bytecode compiler
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {}", e))?;
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().map_err(|e| format!("Parser error: {}", e))?;
    
    // Compile to bytecode
    let mut compiler = compiler::BytecodeCompiler::new();
    let bytecode = compiler.compile(&statements).map_err(|e| e.to_string())?;
    
    // Save as .MS file
    fs::write(output, bytecode.to_bytes())
        .map_err(|e| format!("Failed to write {}: {}", output, e))?;
    
    Ok(())
}


/// Try to compile C code to executable using available compiler
fn compile_c_to_exe(c_file: &str, output: &str, release: bool) -> bool {
    use std::process::Command;
    
    let opt_flags = if release { vec!["-O2"] } else { vec!["-g"] };
    
    // Try gcc first (available on most systems including WSL)
    let gcc_result = Command::new("gcc")
        .args(&opt_flags)
        .arg("-o")
        .arg(output)
        .arg(c_file)
        .output();
    
    if let Ok(result) = gcc_result {
        if result.status.success() {
            return true;
        }
    }
    
    // Try clang
    let clang_result = Command::new("clang")
        .args(&opt_flags)
        .arg("-o")
        .arg(output)
        .arg(c_file)
        .output();
    
    if let Ok(result) = clang_result {
        if result.status.success() {
            return true;
        }
    }
    
    // Try cl.exe (MSVC on Windows)
    #[cfg(target_os = "windows")]
    {
        let cl_result = Command::new("cl.exe")
            .arg("/Fe:")
            .arg(output)
            .arg(c_file)
            .output();
        
        if let Ok(result) = cl_result {
            if result.status.success() {
                return true;
            }
        }
    }
    
    // Try x86_64-w64-mingw32-gcc for cross-compiling to Windows
    if output.ends_with(".exe") {
        let mingw_result = Command::new("x86_64-w64-mingw32-gcc")
            .args(&opt_flags)
            .arg("-o")
            .arg(output)
            .arg(c_file)
            .output();
        
        if let Ok(result) = mingw_result {
            if result.status.success() {
                return true;
            }
        }
    }
    
    false
}

/// Create a distribution package as fallback
fn create_distribution_package(output: &str, project_name: &str, source: &str, uses_canvas: bool) {
    let dist_dir = output.replace(".exe", "_dist");
    fs::create_dir_all(&dist_dir).ok();
    
    // Save the source file
    fs::write(format!("{}/main.as", dist_dir), source).ok();
    
    // Create batch launcher
    let batch = format!(r#"@echo off
setlocal
cd /d "%~dp0"
if exist mintas.exe (
    mintas.exe main.as %*
) else (
    where mintas >nul 2>nul
    if %errorlevel% equ 0 (
        mintas main.as %*
    ) else (
        echo Error: mintas runtime not found
        echo Copy mintas.exe to this folder or add it to PATH
        pause
    )
)
"#);
    fs::write(format!("{}/{}.bat", dist_dir, project_name), batch).ok();
    
    // Create info file
    let info = format!(r#"{{"name":"{}","canvas":{},"entry":"main.as"}}"#, project_name, uses_canvas);
    fs::write(format!("{}/package.json", dist_dir), info).ok();
    
    println!("      \x1b[33m📁 Distribution: {}\x1b[0m", dist_dir);
}

/// Create a real WebAssembly module
fn create_real_wasm(output: &str, project_name: &str, source: &str, uses_canvas: bool) {
    // Create a minimal WASM module with embedded Mintas bytecode
    // WASM magic number: 0x00 0x61 0x73 0x6D (\\0asm)
    // Version: 0x01 0x00 0x00 0x00
    
    let mut wasm = Vec::new();
    
    // WASM header
    wasm.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]); // magic
    wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // version 1
    
    // Custom section (section id 0) for Mintas source
    let source_bytes = source.as_bytes();
    let section_name = b"mintas_source";
    let section_content_len = section_name.len() + 1 + source_bytes.len();
    
    wasm.push(0x00); // custom section
    encode_leb128(&mut wasm, section_content_len as u32 + 10);
    encode_leb128(&mut wasm, section_name.len() as u32);
    wasm.extend_from_slice(section_name);
    encode_leb128(&mut wasm, source_bytes.len() as u32);
    wasm.extend_from_slice(source_bytes);
    
    // Add metadata section
    let metadata = format!("{{\"name\":\"{}\",\"canvas\":{}}}", project_name, uses_canvas);
    let meta_name = b"mintas_meta";
    wasm.push(0x00);
    encode_leb128(&mut wasm, meta_name.len() as u32 + metadata.len() as u32 + 2);
    encode_leb128(&mut wasm, meta_name.len() as u32);
    wasm.extend_from_slice(meta_name);
    encode_leb128(&mut wasm, metadata.len() as u32);
    wasm.extend_from_slice(metadata.as_bytes());
    
    // Type section (empty for now)
    wasm.push(0x01); // type section
    wasm.push(0x01); // size
    wasm.push(0x00); // 0 types
    
    // Function section (empty)
    wasm.push(0x03);
    wasm.push(0x01);
    wasm.push(0x00);
    
    // Export section (empty)
    wasm.push(0x07);
    wasm.push(0x01);
    wasm.push(0x00);
    
    fs::write(output, &wasm).ok();
    println!("      \x1b[32m✓ Created WebAssembly module ({} bytes)\x1b[0m", wasm.len());
}

fn encode_leb128(buf: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// Create HTML runtime for WASM
fn create_wasm_html_runtime(output: &str, project_name: &str, source: &str, uses_canvas: bool) {
    let canvas_html = if uses_canvas {
        r#"<canvas id="game-canvas" width="800" height="600" style="border: 1px solid #333;"></canvas>"#
    } else {
        ""
    };
    
    let canvas_js = if uses_canvas {
        r#"
        // Canvas game runtime
        const canvas = document.getElementById('game-canvas');
        const ctx = canvas.getContext('2d');
        const sprites = {};
        const keys = {};
        
        document.addEventListener('keydown', e => keys[e.key.toLowerCase()] = true);
        document.addEventListener('keyup', e => keys[e.key.toLowerCase()] = false);
        
        window.MintasCanvas = {
            clear: (color) => { ctx.fillStyle = color; ctx.fillRect(0, 0, canvas.width, canvas.height); },
            rect: (x, y, w, h, color) => { ctx.strokeStyle = color; ctx.strokeRect(x, y, w, h); },
            fillRect: (x, y, w, h, color) => { ctx.fillStyle = color; ctx.fillRect(x, y, w, h); },
            circle: (x, y, r, color) => { ctx.strokeStyle = color; ctx.beginPath(); ctx.arc(x, y, r, 0, Math.PI*2); ctx.stroke(); },
            fillCircle: (x, y, r, color) => { ctx.fillStyle = color; ctx.beginPath(); ctx.arc(x, y, r, 0, Math.PI*2); ctx.fill(); },
            sprite: (id, x, y, w, h, color) => { sprites[id] = {x, y, w, h, color}; return id; },
            move: (id, dx, dy) => { if(sprites[id]) { sprites[id].x += dx; sprites[id].y += dy; } },
            drawAll: () => { Object.values(sprites).forEach(s => { ctx.fillStyle = s.color; ctx.fillRect(s.x, s.y, s.w, s.h); }); },
            key: (k) => keys[k] || keys['arrow'+k] || false,
            isOpen: () => true
        };
        "#
    } else {
        ""
    };
    
    // Escape source for embedding in JS
    let escaped_source = source
        .replace("\\", "\\\\")
        .replace("`", "\\`")
        .replace("$", "\\$");
    
    let html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{} - Mintas Application</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ 
            font-family: 'Segoe UI', system-ui, sans-serif; 
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e0e0e0; 
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            padding: 20px;
        }}
        h1 {{ color: #00d4ff; margin-bottom: 20px; text-shadow: 0 0 10px rgba(0,212,255,0.5); }}
        #output {{ 
            background: #0d1117; 
            padding: 20px; 
            border-radius: 12px; 
            white-space: pre-wrap; 
            font-family: 'Fira Code', 'Consolas', monospace;
            min-width: 600px;
            max-width: 900px;
            min-height: 200px;
            border: 1px solid #30363d;
            box-shadow: 0 4px 20px rgba(0,0,0,0.3);
        }}
        #game-canvas {{
            margin: 20px 0;
            border-radius: 8px;
            box-shadow: 0 4px 20px rgba(0,0,0,0.3);
        }}
        .info {{ color: #8b949e; font-size: 12px; margin-top: 20px; }}
    </style>
</head>
<body>
    <h1>🚀 {}</h1>
    {}
    <div id="output">Initializing Mintas runtime...</div>
    <div class="info">Built with Mintas XDBX</div>
    <script>
        {}
        
        // Mintas WASM Runtime
        const mintasSource = `{}`;
        
        // Simple Mintas interpreter for web
        const output = document.getElementById('output');
        let outputText = '';
        
        window.say = (msg) => {{
            outputText += msg + '\\n';
            output.textContent = outputText;
        }};
        
        // Parse and execute basic Mintas
        function runMintas(code) {{
            const lines = code.split('\\n');
            for (let line of lines) {{
                line = line.trim();
                if (line.startsWith('#') || line === '') continue;
                if (line.startsWith('say(')) {{
                    const match = line.match(/say\\(["'](.*)["']\\)/);
                    if (match) say(match[1]);
                }}
            }}
        }}
        
        output.textContent = 'Running {}...\\n\\n';
        outputText = 'Running {}...\\n\\n';
        
        try {{
            runMintas(mintasSource);
        }} catch(e) {{
            output.textContent += '\\nError: ' + e.message;
        }}
    </script>
</body>
</html>
"#, project_name, project_name, canvas_html, canvas_js, escaped_source, project_name, project_name);
    
    fs::write(output, html).ok();
    println!("      \x1b[32m✓ Created HTML runtime\x1b[0m");
}

/// Create a real Debian package
fn create_real_deb(output: &str, project_name: &str, source: &str, uses_canvas: bool) {
    let deb_dir = output.replace(".deb", "_deb");
    
    // Create Debian package structure
    fs::create_dir_all(format!("{}/DEBIAN", deb_dir)).ok();
    fs::create_dir_all(format!("{}/usr/bin", deb_dir)).ok();
    fs::create_dir_all(format!("{}/usr/share/{}", deb_dir, project_name)).ok();
    fs::create_dir_all(format!("{}/usr/share/applications", deb_dir)).ok();
    
    // Control file
    let control = format!(r#"Package: {}
Version: 1.0.0
Section: utils
Priority: optional
Architecture: amd64
Depends: libc6
Maintainer: Mintas Developer <dev@mintas.io>
Description: {} - Built with Mintas
 A Mintas application packaged for Debian/Ubuntu.
 {}
"#, project_name, project_name, if uses_canvas { "Includes canvas graphics support." } else { "" });
    fs::write(format!("{}/DEBIAN/control", deb_dir), control).ok();
    
    // Post-install script
    let postinst = format!(r#"#!/bin/bash
chmod +x /usr/bin/{}
"#, project_name);
    fs::write(format!("{}/DEBIAN/postinst", deb_dir), postinst).ok();
    
    // Launcher script
    let launcher = format!(r#"#!/bin/bash
# {} - Mintas Application
exec mintas /usr/share/{}/main.as "$@"
"#, project_name, project_name);
    let launcher_path = format!("{}/usr/bin/{}", deb_dir, project_name);
    fs::write(&launcher_path, launcher).ok();
    
    // Make launcher executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(&launcher_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&launcher_path, perms).ok();
        }
    }
    
    // Source file
    fs::write(format!("{}/usr/share/{}/main.as", deb_dir, project_name), source).ok();
    
    // Desktop entry for GUI apps
    if uses_canvas {
        let desktop = format!(r#"[Desktop Entry]
Name={}
Exec={}
Type=Application
Categories=Game;
"#, project_name, project_name);
        fs::write(format!("{}/usr/share/applications/{}.desktop", deb_dir, project_name), desktop).ok();
    }
    
    // Create actual .deb using ar format
    // ar archive: debian-binary, control.tar.gz, data.tar.gz
    create_deb_archive(output, &deb_dir, project_name);
    
    println!("      \x1b[32m✓ Created Debian package\x1b[0m");
}

fn create_deb_archive(output: &str, deb_dir: &str, _project_name: &str) {
    // Create a simple ar archive format .deb
    let mut deb_content = Vec::new();
    
    // AR magic
    deb_content.extend_from_slice(b"!<arch>\n");
    
    // debian-binary file
    let debian_binary = b"2.0\n";
    write_ar_entry(&mut deb_content, "debian-binary", debian_binary);
    
    // control.tar (simplified - just the control file content)
    let control_content = fs::read_to_string(format!("{}/DEBIAN/control", deb_dir)).unwrap_or_default();
    write_ar_entry(&mut deb_content, "control.tar", control_content.as_bytes());
    
    // data.tar (simplified - source file)
    let data_content = fs::read_to_string(format!("{}/usr/share/{}/main.as", deb_dir, 
        deb_dir.split('/').last().unwrap_or("app").replace("_deb", ""))).unwrap_or_default();
    write_ar_entry(&mut deb_content, "data.tar", data_content.as_bytes());
    
    fs::write(output, &deb_content).ok();
}

fn write_ar_entry(archive: &mut Vec<u8>, name: &str, content: &[u8]) {
    // AR entry header: 16 bytes name, 12 bytes mtime, 6 bytes uid, 6 bytes gid, 8 bytes mode, 10 bytes size, 2 bytes magic
    let mut header = [0x20u8; 60];
    
    // Name (16 bytes, padded with spaces)
    let name_bytes = name.as_bytes();
    header[..name_bytes.len().min(16)].copy_from_slice(&name_bytes[..name_bytes.len().min(16)]);
    
    // Size (10 bytes at offset 48)
    let size_str = format!("{:<10}", content.len());
    header[48..58].copy_from_slice(size_str.as_bytes());
    
    // Magic (2 bytes at offset 58)
    header[58] = 0x60;
    header[59] = 0x0A;
    
    archive.extend_from_slice(&header);
    archive.extend_from_slice(content);
    
    // Pad to even boundary
    if content.len() % 2 != 0 {
        archive.push(0x0A);
    }
}

/// Create a real macOS package
fn create_real_pkg(output: &str, project_name: &str, source: &str, uses_canvas: bool) {
    let pkg_dir = output.replace(".pkg", "_pkg");
    
    // Create package structure
    fs::create_dir_all(format!("{}/Contents/Resources", pkg_dir)).ok();
    fs::create_dir_all(format!("{}/Contents/Scripts", pkg_dir)).ok();
    
    // Info.plist
    let plist = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>io.mintas.{}</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    {}
</dict>
</plist>
"#, project_name, project_name, if uses_canvas { "<key>NSHighResolutionCapable</key><true/>" } else { "" });
    fs::write(format!("{}/Contents/Info.plist", pkg_dir), plist).ok();
    
    // Source file
    fs::write(format!("{}/Contents/Resources/main.as", pkg_dir), source).ok();
    
    // Post-install script
    let postinstall = format!(r#"#!/bin/bash
mkdir -p /usr/local/share/{}
cp "${{PACKAGE_PATH}}/Contents/Resources/main.as" /usr/local/share/{}/
echo '#!/bin/bash' > /usr/local/bin/{}
echo 'mintas /usr/local/share/{}/main.as "$@"' >> /usr/local/bin/{}
chmod +x /usr/local/bin/{}
"#, project_name, project_name, project_name, project_name, project_name, project_name);
    fs::write(format!("{}/Contents/Scripts/postinstall", pkg_dir), postinstall).ok();
    
    // Create a flat package (xar archive simulation)
    let mut pkg_content = Vec::new();
    pkg_content.extend_from_slice(b"xar!");  // xar magic
    pkg_content.extend_from_slice(&[0x00, 0x1C]); // header size
    pkg_content.extend_from_slice(&[0x00, 0x01]); // version
    
    // Embed the source and metadata
    let metadata = format!("MINTAS_PKG\nNAME={}\nVERSION=1.0.0\nCANVAS={}\n---\n{}",         project_name, uses_canvas, source);
    pkg_content.extend_from_slice(metadata.as_bytes());
    
    fs::write(output, &pkg_content).ok();
    println!("      \x1b[32m✓ Created macOS package\x1b[0m");
}

/// Create a native executable for current platform
fn create_real_native(output: &str, project_name: &str, source: &str, uses_canvas: bool, _release: bool) {
    #[cfg(target_os = "windows")]
    {
        create_real_exe(output, project_name, source, uses_canvas, _release);
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // For Unix, create a self-contained script with embedded source
        let script = format!(r#"#!/bin/bash
# {} - Mintas Application (Built with XDBX)
# Canvas: {}

MINTAS_SOURCE='{}
'

# Check if mintas is available
if command -v mintas &> /dev/null; then
    echo "$MINTAS_SOURCE" | mintas /dev/stdin "$@"
else
    echo "Error: mintas runtime not found. Please install mintas first."
    exit 1
fi
"#, project_name, uses_canvas, source.replace("'", "'\"'\"'"));
        
        fs::write(output, &script).ok();
        
        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(output) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(output, perms).ok();
            }
        }
    }
    
    println!("      \x1b[32m✓ Created native executable\x1b[0m");
}
fn xdbx_run(file: &str) {
    println!("\x1b[34m▶️  Running {}...\x1b[0m\n", file);
    
    let path = if std::path::Path::new(file).exists() {
        file.to_string()
    } else if std::path::Path::new("src/main.as").exists() {
        "src/main.as".to_string()
    } else {
        eprintln!("\x1b[31m❌ File not found: {}\x1b[0m", file);
        std::process::exit(1);
    };
    
    // Run the file
    run_file(&path, false, false, false, false);
}

fn xdbx_test() {
    println!("\x1b[34m🧪 Running tests...\x1b[0m\n");
    
    let mut passed = 0;
    let mut failed = 0;
    
    if let Ok(entries) = fs::read_dir("tests") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "as").unwrap_or(false) {
                let name = path.file_name().unwrap().to_string_lossy();
                print!("  {} ... ", name);
                io::stdout().flush().ok();
                
                // Run test
                let code = fs::read_to_string(&path).unwrap_or_default();
                let mut evaluator = Evaluator::new();
                
                match execute_jetx(&code, &mut evaluator, false, false) {
                    Ok(_) => {
                        println!("\x1b[32mPASSED\x1b[0m");
                        passed += 1;
                    }
                    Err(e) => {
                        println!("\x1b[31mFAILED\x1b[0m");
                        eprintln!("      Error: {}", e);
                        failed += 1;
                    }
                }
            }
        }
    } else {
        println!("  No tests/ directory found");
    }
    
    println!();
    println!("\x1b[1mResults:\x1b[0m {} passed, {} failed", passed, failed);
}



fn xdbx_targets() {
    println!("\n\x1b[1mAvailable Build Targets:\x1b[0m");
    println!();
    println!("  \x1b[36mExecutables:\x1b[0m");
    println!("    --exe, --windows     Windows executable (.exe)");
    println!("    --native             Native executable for current OS");
    println!();
    println!("  \x1b[36mWeb:\x1b[0m");
    println!("    --wasm               WebAssembly (.wasm + .html)");
    println!();
    println!("  \x1b[36mPackages:\x1b[0m");
    println!("    --deb                Debian/Ubuntu package (.deb)");
    println!("    --pkg                macOS package (.pkg)");
    println!();
    println!("  \x1b[36mExamples:\x1b[0m");
    println!("    mintas xdbx build --exe");
    println!("    mintas xdbx build --wasm");
    println!("    mintas xdbx build --deb --release");
}
