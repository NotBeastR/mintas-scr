use crate::compiler::BytecodeCompiler;
use crate::encryption::{load_encrypted_bytecode, save_encrypted_bytecode};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::vm::BytecodeVM;
use std::fs;

/// Compile .as file to encrypted .ms bytecode
pub fn compile_to_bytecode(input_path: &str, secret: Option<String>) {
    println!("🔨 Compiling {} to bytecode...", input_path);
    
    // Read source file
    let source = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Error reading file: {}", e);
            std::process::exit(1);
        }
    };
    
    // Lex and parse
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ Lexer error: {}", e);
            std::process::exit(1);
        }
    };
    
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("❌ Parser error: {}", e);
            std::process::exit(1);
        }
    };
    
    // Compile to bytecode
    let mut compiler = BytecodeCompiler::new();
    let program = match compiler.compile(&ast) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ Compilation error: {}", e);
            std::process::exit(1);
        }
    };
    
    // Generate output path
    let output_path = input_path.replace(".as", ".ms");
    
    if secret.is_some() {
        println!("🔒 Using custom secret key for encryption");
    } else {
        println!("⚠️  No secret key provided, using default (insecure) key");
    }

    // Save encrypted bytecode
    match save_encrypted_bytecode(&program, &output_path, secret.as_deref()) {
        Ok(_) => {
            println!("✅ Compiled successfully!");
            println!("📦 Output: {}", output_path);
            println!("🔐 Bytecode is encrypted with AES-256");
        }
        Err(e) => {
            eprintln!("❌ Error saving bytecode: {}", e);
            std::process::exit(1);
        }
    }
}

/// Run encrypted .ms bytecode file
pub fn run_bytecode(input_path: &str, secret: Option<String>) {
    println!("▶️  Running bytecode: {}", input_path);
    
    if secret.is_some() {
        println!("🔓 Using provided secret key for decryption");
    }

    // Load and decrypt bytecode
    let program = match load_encrypted_bytecode(input_path, secret.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ Error loading bytecode: {}", e);
            std::process::exit(1);
        }
    };
    
    // Execute in VM
    let mut vm = BytecodeVM::new(program);
    match vm.execute() {
        Ok(_) => {
            // Success
        }
        Err(e) => {
            eprintln!("❌ Runtime error: {}", e);
            std::process::exit(1);
        }
    }
}
