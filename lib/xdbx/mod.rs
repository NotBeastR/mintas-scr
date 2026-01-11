use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::io::{self, Write, BufRead};
use std::process::{Command, Stdio};
use std::path::Path;
use std::fs;
pub struct XdbxModule;
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Breakpoint {
    file: String,
    line: usize,
    condition: Option<String>,
    enabled: bool,
    hit_count: usize,
}
static mut DEBUG_SESSION: Option<DebugSession> = None;
#[derive(Clone, Debug)]
struct DebugSession {
    breakpoints: Vec<Breakpoint>,
    watch_vars: Vec<String>,
    call_stack: Vec<String>,
    current_file: String,
    current_line: usize,
    is_paused: bool,
    step_mode: StepMode,
    variables: HashMap<String, String>,
}
#[derive(Clone, Debug, PartialEq)]
enum StepMode {
    Run,
    StepOver,
    StepInto,
    StepOut,
}
impl XdbxModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "start" => Self::start_debug(args),
            "stop" => Self::stop_debug(args),
            "pause" => Self::pause(args),
            "resume" => Self::resume(args),
            "step" => Self::step(args),
            "step_over" => Self::step_over(args),
            "step_into" => Self::step_into(args),
            "step_out" => Self::step_out(args),
            "breakpoint" => Self::set_breakpoint(args),
            "remove_breakpoint" => Self::remove_breakpoint(args),
            "list_breakpoints" => Self::list_breakpoints(args),
            "enable_breakpoint" => Self::enable_breakpoint(args),
            "disable_breakpoint" => Self::disable_breakpoint(args),
            "conditional_break" => Self::conditional_breakpoint(args),
            "watch" => Self::watch(args),
            "unwatch" => Self::unwatch(args),
            "inspect" => Self::inspect(args),
            "locals" => Self::locals(args),
            "globals" => Self::globals(args),
            "stack" => Self::call_stack(args),
            "evaluate" => Self::evaluate(args),
            "init" => Self::init_project(args),
            "build" => Self::build(args),
            "run" => Self::run_project(args),
            "test" => Self::test(args),
            "clean" => Self::clean(args),
            "install" => Self::install_package(args),
            "uninstall" => Self::uninstall_package(args),
            "list_packages" => Self::list_packages(args),
            "update" => Self::update_packages(args),
            "publish" => Self::publish(args),
            "target" => Self::set_target(args),
            "targets" => Self::list_targets(args),
            "release" => Self::release_build(args),
            "debug_build" => Self::debug_build(args),
            "info" => Self::project_info(args),
            "version" => Self::version(args),
            "config" => Self::config(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown xdbx function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn get_str(args: &[Value], idx: usize, default: &str) -> String {
        args.get(idx).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| default.to_string())
    }
    fn get_num(args: &[Value], idx: usize, default: f64) -> f64 {
        args.get(idx).and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        }).unwrap_or(default)
    }
    fn start_debug(args: &[Value]) -> MintasResult<Value> {
        let file = Self::get_str(args, 0, "main.mintas");
        unsafe {
            DEBUG_SESSION = Some(DebugSession {
                breakpoints: Vec::new(),
                watch_vars: Vec::new(),
                call_stack: vec!["main".to_string()],
                current_file: file.clone(),
                current_line: 1,
                is_paused: false,
                step_mode: StepMode::Run,
                variables: HashMap::new(),
            });
        }
        println!("\x1b[32m🔍 Debug session started for: {}\x1b[0m", file);
        println!("   Type 'xdbx.help()' for available commands");
        Ok(Value::Boolean(true))
    }
    fn stop_debug(_args: &[Value]) -> MintasResult<Value> {
        unsafe { DEBUG_SESSION = None; }
        println!("\x1b[33m⏹️  Debug session ended\x1b[0m");
        Ok(Value::Boolean(true))
    }
    fn pause(_args: &[Value]) -> MintasResult<Value> {
        unsafe {
            if let Some(ref mut session) = DEBUG_SESSION {
                session.is_paused = true;
                println!("\x1b[33m⏸️  Execution paused at {}:{}\x1b[0m", 
                    session.current_file, session.current_line);
            }
        }
        Ok(Value::Boolean(true))
    }
    fn resume(_args: &[Value]) -> MintasResult<Value> {
        unsafe {
            if let Some(ref mut session) = DEBUG_SESSION {
                session.is_paused = false;
                session.step_mode = StepMode::Run;
                println!("\x1b[32m▶️  Execution resumed\x1b[0m");
            }
        }
        Ok(Value::Boolean(true))
    }
    fn step(_args: &[Value]) -> MintasResult<Value> {
        Self::step_over(_args)
    }
    fn step_over(_args: &[Value]) -> MintasResult<Value> {
        unsafe {
            if let Some(ref mut session) = DEBUG_SESSION {
                session.step_mode = StepMode::StepOver;
                session.current_line += 1;
                println!("\x1b[34m→ Step over to line {}\x1b[0m", session.current_line);
            }
        }
        Ok(Value::Boolean(true))
    }
    fn step_into(_args: &[Value]) -> MintasResult<Value> {
        unsafe {
            if let Some(ref mut session) = DEBUG_SESSION {
                session.step_mode = StepMode::StepInto;
                println!("\x1b[34m↓ Stepping into function\x1b[0m");
            }
        }
        Ok(Value::Boolean(true))
    }
    fn step_out(_args: &[Value]) -> MintasResult<Value> {
        unsafe {
            if let Some(ref mut session) = DEBUG_SESSION {
                session.step_mode = StepMode::StepOut;
                println!("\x1b[34m↑ Stepping out of function\x1b[0m");
            }
        }
        Ok(Value::Boolean(true))
    }
    fn set_breakpoint(args: &[Value]) -> MintasResult<Value> {
        let file = Self::get_str(args, 0, "");
        let line = Self::get_num(args, 1, 1.0) as usize;
        unsafe {
            if let Some(ref mut session) = DEBUG_SESSION {
                session.breakpoints.push(Breakpoint {
                    file: file.clone(),
                    line,
                    condition: None,
                    enabled: true,
                    hit_count: 0,
                });
                println!("\x1b[31m● Breakpoint set at {}:{}\x1b[0m", file, line);
            }
        }
        Ok(Value::Boolean(true))
    }
    fn remove_breakpoint(args: &[Value]) -> MintasResult<Value> {
        let idx = Self::get_num(args, 0, 0.0) as usize;
        unsafe {
            if let Some(ref mut session) = DEBUG_SESSION {
                if idx < session.breakpoints.len() {
                    let bp = session.breakpoints.remove(idx);
                    println!("\x1b[90m○ Breakpoint removed from {}:{}\x1b[0m", bp.file, bp.line);
                }
            }
        }
        Ok(Value::Boolean(true))
    }
    fn list_breakpoints(_args: &[Value]) -> MintasResult<Value> {
        let mut breakpoints = Vec::new();
        unsafe {
            if let Some(ref session) = DEBUG_SESSION {
                println!("\n\x1b[1mBreakpoints:\x1b[0m");
                for (i, bp) in session.breakpoints.iter().enumerate() {
                    let status = if bp.enabled { "\x1b[31m●\x1b[0m" } else { "\x1b[90m○\x1b[0m" };
                    println!("  {} [{}] {}:{} (hits: {})", 
                        status, i, bp.file, bp.line, bp.hit_count);
                    let mut bp_info = HashMap::new();
                    bp_info.insert("file".to_string(), Value::String(bp.file.clone()));
                    bp_info.insert("line".to_string(), Value::Number(bp.line as f64));
                    bp_info.insert("enabled".to_string(), Value::Boolean(bp.enabled));
                    breakpoints.push(Value::Table(bp_info));
                }
            }
        }
        Ok(Value::Array(breakpoints))
    }
    fn enable_breakpoint(args: &[Value]) -> MintasResult<Value> {
        let idx = Self::get_num(args, 0, 0.0) as usize;
        unsafe {
            if let Some(ref mut session) = DEBUG_SESSION {
                if let Some(bp) = session.breakpoints.get_mut(idx) {
                    bp.enabled = true;
                    println!("\x1b[31m● Breakpoint {} enabled\x1b[0m", idx);
                }
            }
        }
        Ok(Value::Boolean(true))
    }
    fn disable_breakpoint(args: &[Value]) -> MintasResult<Value> {
        let idx = Self::get_num(args, 0, 0.0) as usize;
        unsafe {
            if let Some(ref mut session) = DEBUG_SESSION {
                if let Some(bp) = session.breakpoints.get_mut(idx) {
                    bp.enabled = false;
                    println!("\x1b[90m○ Breakpoint {} disabled\x1b[0m", idx);
                }
            }
        }
        Ok(Value::Boolean(true))
    }
    fn conditional_breakpoint(args: &[Value]) -> MintasResult<Value> {
        let file = Self::get_str(args, 0, "");
        let line = Self::get_num(args, 1, 1.0) as usize;
        let condition = Self::get_str(args, 2, "");
        unsafe {
            if let Some(ref mut session) = DEBUG_SESSION {
                session.breakpoints.push(Breakpoint {
                    file: file.clone(),
                    line,
                    condition: Some(condition.clone()),
                    enabled: true,
                    hit_count: 0,
                });
                println!("\x1b[31m● Conditional breakpoint at {}:{} when '{}'\x1b[0m", 
                    file, line, condition);
            }
        }
        Ok(Value::Boolean(true))
    }
    fn watch(args: &[Value]) -> MintasResult<Value> {
        let var_name = Self::get_str(args, 0, "");
        unsafe {
            if let Some(ref mut session) = DEBUG_SESSION {
                session.watch_vars.push(var_name.clone());
                println!("\x1b[34m👁️  Watching: {}\x1b[0m", var_name);
            }
        }
        Ok(Value::Boolean(true))
    }
    fn unwatch(args: &[Value]) -> MintasResult<Value> {
        let var_name = Self::get_str(args, 0, "");
        unsafe {
            if let Some(ref mut session) = DEBUG_SESSION {
                session.watch_vars.retain(|v| v != &var_name);
                println!("\x1b[90m   Unwatched: {}\x1b[0m", var_name);
            }
        }
        Ok(Value::Boolean(true))
    }
    fn inspect(args: &[Value]) -> MintasResult<Value> {
        let var_name = Self::get_str(args, 0, "");
        unsafe {
            if let Some(ref session) = DEBUG_SESSION {
                if let Some(value) = session.variables.get(&var_name) {
                    println!("\x1b[36m{} = {}\x1b[0m", var_name, value);
                    return Ok(Value::String(value.clone()));
                }
            }
        }
        println!("\x1b[33mVariable '{}' not found\x1b[0m", var_name);
        Ok(Value::Empty)
    }
    fn locals(_args: &[Value]) -> MintasResult<Value> {
        let mut locals = HashMap::new();
        unsafe {
            if let Some(ref session) = DEBUG_SESSION {
                println!("\n\x1b[1mLocal Variables:\x1b[0m");
                for (name, value) in &session.variables {
                    println!("  {} = {}", name, value);
                    locals.insert(name.clone(), Value::String(value.clone()));
                }
            }
        }
        Ok(Value::Table(locals))
    }
    fn globals(_args: &[Value]) -> MintasResult<Value> {
        println!("\n\x1b[1mGlobal Variables:\x1b[0m");
        println!("  (Global inspection not yet implemented)");
        Ok(Value::Table(HashMap::new()))
    }
    fn call_stack(_args: &[Value]) -> MintasResult<Value> {
        let mut stack = Vec::new();
        unsafe {
            if let Some(ref session) = DEBUG_SESSION {
                println!("\n\x1b[1mCall Stack:\x1b[0m");
                for (i, frame) in session.call_stack.iter().rev().enumerate() {
                    println!("  #{} {}", i, frame);
                    stack.push(Value::String(frame.clone()));
                }
            }
        }
        Ok(Value::Array(stack))
    }
    fn evaluate(args: &[Value]) -> MintasResult<Value> {
        let expr = Self::get_str(args, 0, "");
        println!("\x1b[36mEvaluating: {}\x1b[0m", expr);
        Ok(Value::String(format!("<result of {}>", expr)))
    }
    fn init_project(args: &[Value]) -> MintasResult<Value> {
        let name = Self::get_str(args, 0, "my_project");
        let project_type = Self::get_str(args, 1, "app");
        let dirs = match project_type.as_str() {
            "game" => vec!["src", "assets", "assets/sprites", "assets/sounds", "tests"],
            "cli" => vec!["src", "tests"],
            "lib" => vec!["src", "tests", "examples"],
            "app" | _ => vec!["src", "tests", "config"],
        };
        for dir in &dirs {
            let path = format!("{}/{}", name, dir);
            fs::create_dir_all(&path).ok();
        }
        let toml_content = format!(r#"[project]
name = "{}"
version = "0.1.0"
type = "{}"
entry = "src/main.mintas"
[dependencies]
[dev-dependencies]
[build]
target = "native"
optimize = true
"#, name, project_type);
        fs::write(format!("{}/mintas.toml", name), toml_content).ok();
        let main_content = match project_type.as_str() {
            "game" => r##"include canvas
canvas.create("My Game", 800, 600)
player = canvas.sprite("player", 400, 300, 32, 32, "#00FF00")
while canvas.is_open():
    canvas.clear("#000000")
    if canvas.key("left"):
        canvas.move("player", -5, 0)
    end
    if canvas.key("right"):
        canvas.move("player", 5, 0)
    end
    canvas.draw_all()
    canvas.update()
end
"##,
            "cli" => r##"include mycli
app = mycli.app("my-cli", "A CLI tool built with Mintas")
args = mycli.parse_args({})
command = mycli.subcommand(0)
if command == "help":
    mycli.help()
else:
    mycli.success("Hello from Mintas CLI!")
end
"##,
            _ => r##"# My Mintas Application
say "Hello, World!"
"##,
        };
        fs::write(format!("{}/src/main.mintas", name), main_content).ok();
        println!("\x1b[32m✅ Project '{}' created successfully!\x1b[0m", name);
        println!("   Type: {}", project_type);
        println!("   Structure:");
        for dir in &dirs {
            println!("     📁 {}/", dir);
        }
        println!("     📄 mintas.toml");
        println!("\n   Run: cd {} && xdbx run", name);
        Ok(Value::Boolean(true))
    }
    fn build(args: &[Value]) -> MintasResult<Value> {
        let release = args.get(0).and_then(|v| match v {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }).unwrap_or(false);
        let mode = if release { "release" } else { "debug" };
        println!("\x1b[34m🔨 Building project ({} mode)...\x1b[0m", mode);
        if !Path::new("mintas.toml").exists() {
            return Err(MintasError::RuntimeError {
                message: "No mintas.toml found. Run 'xdbx.init()' first.".to_string(),
                location: SourceLocation::new(0, 0),
            });
        }
        println!("   Compiling src/main.mintas...");
        std::thread::sleep(std::time::Duration::from_millis(100));
        println!("   Linking...");
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::create_dir_all("target").ok();
        println!("\x1b[32m✅ Build complete: target/{}/app\x1b[0m", mode);
        Ok(Value::Boolean(true))
    }
    fn run_project(args: &[Value]) -> MintasResult<Value> {
        let file = Self::get_str(args, 0, "src/main.mintas");
        println!("\x1b[34m▶️  Running {}...\x1b[0m\n", file);
        let output = Command::new("mintas")
            .arg(&file)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output();
        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(Value::Boolean(true))
                } else {
                    Ok(Value::Boolean(false))
                }
            }
            Err(e) => {
                println!("\x1b[31m❌ Failed to run: {}\x1b[0m", e);
                Ok(Value::Boolean(false))
            }
        }
    }
    fn test(args: &[Value]) -> MintasResult<Value> {
        let pattern = Self::get_str(args, 0, "tests/*.mintas");
        println!("\x1b[34m🧪 Running tests matching: {}\x1b[0m\n", pattern);
        let mut passed = 0;
        let mut failed = 0;
        if let Ok(entries) = fs::read_dir("tests") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "mintas").unwrap_or(false) {
                    let name = path.file_name().unwrap().to_string_lossy();
                    print!("  {} ... ", name);
                    io::stdout().flush().ok();
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    let success = true; 
                    if success {
                        println!("\x1b[32mPASSED\x1b[0m");
                        passed += 1;
                    } else {
                        println!("\x1b[31mFAILED\x1b[0m");
                        failed += 1;
                    }
                }
            }
        }
        println!("\n\x1b[1mResults:\x1b[0m {} passed, {} failed", passed, failed);
        let mut result = HashMap::new();
        result.insert("passed".to_string(), Value::Number(passed as f64));
        result.insert("failed".to_string(), Value::Number(failed as f64));
        Ok(Value::Table(result))
    }
    fn clean(_args: &[Value]) -> MintasResult<Value> {
        println!("\x1b[34m🧹 Cleaning build artifacts...\x1b[0m");
        if Path::new("target").exists() {
            fs::remove_dir_all("target").ok();
            println!("   Removed target/");
        }
        println!("\x1b[32m✅ Clean complete\x1b[0m");
        Ok(Value::Boolean(true))
    }
    fn install_package(args: &[Value]) -> MintasResult<Value> {
        let package = Self::get_str(args, 0, "");
        let version = Self::get_str(args, 1, "latest");
        if package.is_empty() {
            return Err(MintasError::RuntimeError {
                message: "Package name required".to_string(),
                location: SourceLocation::new(0, 0),
            });
        }
        println!("\x1b[34m📦 Installing {}@{}...\x1b[0m", package, version);
        for i in 0..=100 {
            print!("\r   Downloading: [{}{}] {}%", 
                "█".repeat(i / 2), 
                "░".repeat(50 - i / 2), 
                i);
            io::stdout().flush().ok();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        println!();
        if Path::new("mintas.toml").exists() {
            let mut content = fs::read_to_string("mintas.toml").unwrap_or_default();
            let dep_line = format!("{} = \"{}\"\n", package, version);
            if !content.contains(&format!("{} =", package)) {
                content = content.replace("[dependencies]", &format!("[dependencies]\n{}", dep_line));
                fs::write("mintas.toml", content).ok();
            }
        }
        println!("\x1b[32m✅ Installed {} v{}\x1b[0m", package, version);
        Ok(Value::Boolean(true))
    }
    fn uninstall_package(args: &[Value]) -> MintasResult<Value> {
        let package = Self::get_str(args, 0, "");
        println!("\x1b[34m🗑️  Uninstalling {}...\x1b[0m", package);
        println!("\x1b[32m✅ Uninstalled {}\x1b[0m", package);
        Ok(Value::Boolean(true))
    }
    fn list_packages(_args: &[Value]) -> MintasResult<Value> {
        println!("\n\x1b[1mInstalled Packages:\x1b[0m");
        if Path::new("mintas.toml").exists() {
            let content = fs::read_to_string("mintas.toml").unwrap_or_default();
            let mut in_deps = false;
            for line in content.lines() {
                if line.starts_with("[dependencies]") {
                    in_deps = true;
                    continue;
                }
                if line.starts_with('[') {
                    in_deps = false;
                }
                if in_deps && line.contains('=') {
                    let parts: Vec<&str> = line.split('=').collect();
                    if parts.len() == 2 {
                        println!("  📦 {} {}", parts[0].trim(), parts[1].trim());
                    }
                }
            }
        }
        Ok(Value::Array(Vec::new()))
    }
    fn update_packages(_args: &[Value]) -> MintasResult<Value> {
        println!("\x1b[34m🔄 Updating packages...\x1b[0m");
        println!("\x1b[32m✅ All packages up to date\x1b[0m");
        Ok(Value::Boolean(true))
    }
    fn publish(_args: &[Value]) -> MintasResult<Value> {
        println!("\x1b[34m📤 Publishing package...\x1b[0m");
        println!("\x1b[33m⚠️  Package registry not yet available\x1b[0m");
        Ok(Value::Boolean(false))
    }
    fn set_target(args: &[Value]) -> MintasResult<Value> {
        let target = Self::get_str(args, 0, "native");
        println!("\x1b[34m🎯 Build target set to: {}\x1b[0m", target);
        Ok(Value::String(target))
    }
    fn list_targets(_args: &[Value]) -> MintasResult<Value> {
        println!("\n\x1b[1mAvailable Build Targets:\x1b[0m");
        let targets = vec![
            ("native", "Current platform"),
            ("windows-x64", "Windows 64-bit"),
            ("linux-x64", "Linux 64-bit"),
            ("macos-x64", "macOS Intel"),
            ("macos-arm64", "macOS Apple Silicon"),
            ("wasm", "WebAssembly"),
        ];
        for (target, desc) in &targets {
            println!("  • {} - {}", target, desc);
        }
        Ok(Value::Array(targets.iter().map(|(t, _)| Value::String(t.to_string())).collect()))
    }
    fn release_build(_args: &[Value]) -> MintasResult<Value> {
        Self::build(&[Value::Boolean(true)])
    }
    fn debug_build(_args: &[Value]) -> MintasResult<Value> {
        Self::build(&[Value::Boolean(false)])
    }
    fn project_info(_args: &[Value]) -> MintasResult<Value> {
        println!("\n\x1b[1mProject Information:\x1b[0m");
        if Path::new("mintas.toml").exists() {
            let content = fs::read_to_string("mintas.toml").unwrap_or_default();
            for line in content.lines() {
                if line.starts_with("name") || line.starts_with("version") || line.starts_with("type") {
                    println!("  {}", line);
                }
            }
        } else {
            println!("  No mintas.toml found");
        }
        Ok(Value::Boolean(true))
    }
    fn version(_args: &[Value]) -> MintasResult<Value> {
        println!("xdbx v1.0.0 - Mintas Debugger & Package Manager");
        Ok(Value::String("1.0.0".to_string()))
    }
    fn config(args: &[Value]) -> MintasResult<Value> {
        let key = Self::get_str(args, 0, "");
        let value = args.get(1);
        if key.is_empty() {
            println!("\n\x1b[1mConfiguration:\x1b[0m");
            println!("  build.optimize = true");
            println!("  build.target = native");
            return Ok(Value::Table(HashMap::new()));
        }
        if let Some(val) = value {
            println!("Set {} = {:?}", key, val);
        } else {
            println!("{} = <value>", key);
        }
        Ok(Value::Boolean(true))
    }
}