use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::io::{self, Write};
use chrono;
pub struct MyCLIModule;
impl MyCLIModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "app" => Self::app(args),
            "command" => Self::command(args),
            "option" => Self::option(args),
            "flag" => Self::flag(args),
            "run" => Self::run(args),
            "version" => Self::version(args),
            "help" => Self::help(args),
            "args" => Self::args(args),
            "parse_args" => Self::parse_args(args),
            "subcommand" => Self::subcommand(args),
            "required" => Self::required(args),
            "default" => Self::default_val(args),
            "env_var" => Self::env_var(args),
            "print" => Self::print(args),
            "println" => Self::println(args),
            "success" => Self::success(args),
            "error" => Self::error(args),
            "warning" => Self::warning(args),
            "info" => Self::info(args),
            "debug" => Self::debug(args),
            "bold" => Self::bold(args),
            "dim" => Self::dim(args),
            "color" => Self::color(args),
            "bg" => Self::bg(args),
            "hex" => Self::hex(args),
            "style" => Self::style(args),
            "progress_bar" => Self::progress_bar(args),
            "progress_update" => Self::progress_update(args),
            "progress_done" => Self::progress_done(args),
            "spinner" => Self::spinner(args),
            "spinner_success" => Self::spinner_success(args),
            "spinner_error" => Self::spinner_error(args),
            "prompt" => Self::prompt(args),
            "password" => Self::password(args),
            "confirm" => Self::confirm(args),
            "select" => Self::select(args),
            "multiselect" => Self::multiselect(args),
            "number" => Self::number(args),
            "autocomplete" => Self::autocomplete(args),
            "edit" => Self::edit(args),
            "table" => Self::table(args),
            "box" => Self::box_display(args),
            "panel" => Self::panel(args),
            "divider" => Self::divider(args),
            "banner" => Self::banner(args),
            "tree" => Self::tree(args),
            "list" => Self::list(args),
            "task" => Self::task(args),
            "task_done" => Self::task_done(args),
            "task_fail" => Self::task_fail(args),
            "task_skip" => Self::task_skip(args),
            "log" => Self::log(args),
            "log_file" => Self::log_file(args),
            "log_level" => Self::log_level(args),
            "clear" => Self::clear(args),
            "newline" => Self::newline(args),
            "separator" => Self::separator(args),
            "history" => Self::history(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown mycli function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn args(_args: &[Value]) -> MintasResult<Value> {
        let args: Vec<Value> = std::env::args()
            .skip(1)
            .map(|s| Value::String(s))
            .collect();
        Ok(Value::Array(args))
    }
    fn parse_args(args: &[Value]) -> MintasResult<Value> {
        let spec = args.get(0).and_then(|v| match v {
            Value::Table(t) => Some(t.clone()),
            _ => None,
        }).unwrap_or_default();
        let cli_args: Vec<String> = std::env::args().skip(1).collect();
        let mut result = HashMap::new();
        let mut positional = Vec::new();
        let mut i = 0;
        while i < cli_args.len() {
            let arg = &cli_args[i];
            if arg.starts_with("--") {
                let key = arg.trim_start_matches("--");
                if let Some(eq_pos) = key.find('=') {
                    let (k, v) = key.split_at(eq_pos);
                    result.insert(k.to_string(), Value::String(v[1..].to_string()));
                } else if i + 1 < cli_args.len() && !cli_args[i + 1].starts_with('-') {
                    result.insert(key.to_string(), Value::String(cli_args[i + 1].clone()));
                    i += 1;
                } else {
                    result.insert(key.to_string(), Value::Boolean(true));
                }
            } else if arg.starts_with('-') {
                let key = arg.trim_start_matches('-');
                if i + 1 < cli_args.len() && !cli_args[i + 1].starts_with('-') {
                    result.insert(key.to_string(), Value::String(cli_args[i + 1].clone()));
                    i += 1;
                } else {
                    result.insert(key.to_string(), Value::Boolean(true));
                }
            } else {
                positional.push(Value::String(arg.clone()));
            }
            i += 1;
        }
        for (key, default_val) in spec {
            if !result.contains_key(&key) {
                result.insert(key, default_val);
            }
        }
        result.insert("_positional".to_string(), Value::Array(positional));
        Ok(Value::Table(result))
    }
    fn subcommand(args: &[Value]) -> MintasResult<Value> {
        let cli_args: Vec<String> = std::env::args().skip(1).collect();
        let idx = args.get(0).and_then(|v| match v {
            Value::Number(n) => Some(*n as usize),
            _ => None,
        }).unwrap_or(0);
        Ok(cli_args.get(idx)
            .map(|s| Value::String(s.clone()))
            .unwrap_or(Value::Empty))
    }
    fn required(args: &[Value]) -> MintasResult<Value> {
        let value = args.get(0);
        let name = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "argument".to_string());
        match value {
            Some(Value::Empty) | None => {
                eprintln!("\x1b[31m❌ Error: {} is required\x1b[0m", name);
                std::process::exit(1);
            }
            Some(v) => Ok(v.clone()),
        }
    }
    fn default_val(args: &[Value]) -> MintasResult<Value> {
        let value = args.get(0);
        let default = args.get(1).cloned().unwrap_or(Value::Empty);
        match value {
            Some(Value::Empty) | None => Ok(default),
            Some(v) => Ok(v.clone()),
        }
    }
    fn env_var(args: &[Value]) -> MintasResult<Value> {
        let name = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let default = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        });
        match std::env::var(&name) {
            Ok(val) => Ok(Value::String(val)),
            Err(_) => Ok(default.map(Value::String).unwrap_or(Value::Empty)),
        }
    }
    fn app(args: &[Value]) -> MintasResult<Value> {
        let name = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "app".to_string());
        let description = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let mut app = HashMap::new();
        app.insert("name".to_string(), Value::String(name));
        app.insert("description".to_string(), Value::String(description));
        app.insert("version".to_string(), Value::String("1.0.3".to_string()));
        app.insert("commands".to_string(), Value::Array(Vec::new()));
        app.insert("options".to_string(), Value::Array(Vec::new()));
        Ok(Value::Table(app))
    }
    fn command(args: &[Value]) -> MintasResult<Value> {
        let name = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let description = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let mut cmd = HashMap::new();
        cmd.insert("name".to_string(), Value::String(name));
        cmd.insert("description".to_string(), Value::String(description));
        Ok(Value::Table(cmd))
    }
    fn option(args: &[Value]) -> MintasResult<Value> {
        let long = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let short = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let description = args.get(2).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let mut opt = HashMap::new();
        opt.insert("long".to_string(), Value::String(long));
        opt.insert("short".to_string(), Value::String(short));
        opt.insert("description".to_string(), Value::String(description));
        opt.insert("required".to_string(), Value::Boolean(false));
        Ok(Value::Table(opt))
    }
    fn flag(args: &[Value]) -> MintasResult<Value> {
        Self::option(args)
    }
    fn run(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Boolean(true))
    }
    fn version(args: &[Value]) -> MintasResult<Value> {
        let ver = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "1.0.3".to_string());
        Ok(Value::String(ver))
    }
    fn help(_args: &[Value]) -> MintasResult<Value> {
        println!("Usage: <command> [options]");
        println!("\nCommands:");
        println!("  help     Show this help message");
        println!("  version  Show version information");
        Ok(Value::Boolean(true))
    }
    fn print(args: &[Value]) -> MintasResult<Value> {
        let text = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => Some(format!("{:?}", v)),
        }).unwrap_or_default();
        print!("{}", text);
        io::stdout().flush().ok();
        Ok(Value::Boolean(true))
    }
    fn println(args: &[Value]) -> MintasResult<Value> {
        let text = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => Some(format!("{:?}", v)),
        }).unwrap_or_default();
        println!("{}", text);
        Ok(Value::Boolean(true))
    }
    fn success(args: &[Value]) -> MintasResult<Value> {
        let text = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        println!("\x1b[32m✅ {}\x1b[0m", text);
        Ok(Value::Boolean(true))
    }
    fn error(args: &[Value]) -> MintasResult<Value> {
        let text = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        eprintln!("\x1b[31m❌ {}\x1b[0m", text);
        Ok(Value::Boolean(true))
    }
    fn warning(args: &[Value]) -> MintasResult<Value> {
        let text = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        println!("\x1b[33m⚠️  {}\x1b[0m", text);
        Ok(Value::Boolean(true))
    }
    fn info(args: &[Value]) -> MintasResult<Value> {
        let text = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        println!("\x1b[34mℹ️  {}\x1b[0m", text);
        Ok(Value::Boolean(true))
    }
    fn debug(args: &[Value]) -> MintasResult<Value> {
        let text = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        println!("\x1b[90m🔍 {}\x1b[0m", text);
        Ok(Value::Boolean(true))
    }
    fn bold(args: &[Value]) -> MintasResult<Value> {
        let text = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        Ok(Value::String(format!("\x1b[1m{}\x1b[0m", text)))
    }
    fn dim(args: &[Value]) -> MintasResult<Value> {
        let text = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        Ok(Value::String(format!("\x1b[2m{}\x1b[0m", text)))
    }
    fn color(args: &[Value]) -> MintasResult<Value> {
        let color_name = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let text = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let code = match color_name.to_lowercase().as_str() {
            "red" => "31",
            "green" => "32",
            "yellow" => "33",
            "blue" => "34",
            "magenta" => "35",
            "cyan" => "36",
            "white" => "37",
            "gray" | "grey" => "90",
            _ => "0",
        };
        println!("\x1b[{}m{}\x1b[0m", code, text);
        Ok(Value::Boolean(true))
    }
    fn bg(args: &[Value]) -> MintasResult<Value> {
        let color_name = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let text = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let code = match color_name.to_lowercase().as_str() {
            "red" => "41",
            "green" => "42",
            "yellow" => "43",
            "blue" => "44",
            "magenta" => "45",
            "cyan" => "46",
            "white" => "47",
            _ => "0",
        };
        println!("\x1b[{}m{}\x1b[0m", code, text);
        Ok(Value::Boolean(true))
    }
    fn hex(args: &[Value]) -> MintasResult<Value> {
        let _hex_color = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let text = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        println!("{}", text);
        Ok(Value::Boolean(true))
    }
    fn style(args: &[Value]) -> MintasResult<Value> {
        let text = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        println!("{}", text);
        Ok(Value::Boolean(true))
    }
    fn progress_bar(args: &[Value]) -> MintasResult<Value> {
        let total = args.get(0).and_then(|v| match v {
            Value::Number(n) => Some(*n as u64),
            _ => None,
        }).unwrap_or(100);
        let label = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Progress".to_string());
        let mut bar = HashMap::new();
        bar.insert("total".to_string(), Value::Number(total as f64));
        bar.insert("current".to_string(), Value::Number(0.0));
        bar.insert("label".to_string(), Value::String(label));
        Ok(Value::Table(bar))
    }
    fn progress_update(args: &[Value]) -> MintasResult<Value> {
        let current = args.get(1).and_then(|v| match v {
            Value::Number(n) => Some(*n as u64),
            _ => None,
        }).unwrap_or(0);
        let total = 100u64;
        let percent = (current * 100) / total;
        let filled = (percent / 2) as usize;
        let empty = 50 - filled;
        print!("\r[{}{}] {}%", "█".repeat(filled), "░".repeat(empty), percent);
        io::stdout().flush().ok();
        Ok(Value::Boolean(true))
    }
    fn progress_done(_args: &[Value]) -> MintasResult<Value> {
        println!("\r[{}] 100% ✅", "█".repeat(50));
        Ok(Value::Boolean(true))
    }
    fn spinner(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Loading...".to_string());
        print!("⠋ {}", message);
        io::stdout().flush().ok();
        let mut spinner = HashMap::new();
        spinner.insert("message".to_string(), Value::String(message));
        spinner.insert("active".to_string(), Value::Boolean(true));
        Ok(Value::Table(spinner))
    }
    fn spinner_success(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Done!".to_string());
        println!("\r\x1b[32m✅ {}\x1b[0m", message);
        Ok(Value::Boolean(true))
    }
    fn spinner_error(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Failed!".to_string());
        println!("\r\x1b[31m❌ {}\x1b[0m", message);
        Ok(Value::Boolean(true))
    }
    fn prompt(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Enter value:".to_string());
        let default = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        });
        if let Some(def) = &default {
            print!("{} [{}]: ", message, def);
        } else {
            print!("{}: ", message);
        }
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_string();
        if input.is_empty() {
            Ok(Value::String(default.unwrap_or_default()))
        } else {
            Ok(Value::String(input))
        }
    }
    fn password(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Password:".to_string());
        print!("{}: ", message);
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        Ok(Value::String(input.trim().to_string()))
    }
    fn confirm(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Confirm?".to_string());
        print!("{} [y/N]: ", message);
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_lowercase();
        Ok(Value::Boolean(input == "y" || input == "yes"))
    }
    fn select(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Select:".to_string());
        let options = args.get(1).and_then(|v| match v {
            Value::Array(arr) => Some(arr.clone()),
            _ => None,
        }).unwrap_or_default();
        println!("{}", message);
        for (i, opt) in options.iter().enumerate() {
            if let Value::String(s) = opt {
                println!("  {}. {}", i + 1, s);
            }
        }
        print!("Choice: ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let choice: usize = input.trim().parse().unwrap_or(1);
        if choice > 0 && choice <= options.len() {
            Ok(options[choice - 1].clone())
        } else {
            Ok(options.first().cloned().unwrap_or(Value::Empty))
        }
    }
    fn multiselect(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Select (comma-separated):".to_string());
        let options = args.get(1).and_then(|v| match v {
            Value::Array(arr) => Some(arr.clone()),
            _ => None,
        }).unwrap_or_default();
        println!("{}", message);
        for (i, opt) in options.iter().enumerate() {
            if let Value::String(s) = opt {
                println!("  {}. {}", i + 1, s);
            }
        }
        print!("Choices: ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let selected: Vec<Value> = input.trim()
            .split(',')
            .filter_map(|s| s.trim().parse::<usize>().ok())
            .filter(|&i| i > 0 && i <= options.len())
            .map(|i| options[i - 1].clone())
            .collect();
        Ok(Value::Array(selected))
    }
    fn number(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Enter number:".to_string());
        let min = args.get(1).and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        });
        let max = args.get(2).and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        });
        print!("{}: ", message);
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let num: f64 = input.trim().parse().unwrap_or(0.0);
        let num = if let Some(min_val) = min {
            num.max(min_val)
        } else { num };
        let num = if let Some(max_val) = max {
            num.min(max_val)
        } else { num };
        Ok(Value::Number(num))
    }
    fn table(args: &[Value]) -> MintasResult<Value> {
        let data = args.get(0).and_then(|v| match v {
            Value::Array(arr) => Some(arr.clone()),
            _ => None,
        }).unwrap_or_default();
        if data.is_empty() {
            return Ok(Value::Boolean(true));
        }
        let mut widths: Vec<usize> = Vec::new();
        for row in &data {
            if let Value::Array(cols) = row {
                for (i, col) in cols.iter().enumerate() {
                    let len = match col {
                        Value::String(s) => s.len(),
                        Value::Number(n) => format!("{}", n).len(),
                        _ => 4,
                    };
                    if i >= widths.len() {
                        widths.push(len);
                    } else if len > widths[i] {
                        widths[i] = len;
                    }
                }
            }
        }
        let border_top = format!("╭{}╮", widths.iter().map(|w| "─".repeat(w + 2)).collect::<Vec<_>>().join("┬"));
        let border_mid = format!("├{}┤", widths.iter().map(|w| "─".repeat(w + 2)).collect::<Vec<_>>().join("┼"));
        let border_bot = format!("╰{}╯", widths.iter().map(|w| "─".repeat(w + 2)).collect::<Vec<_>>().join("┴"));
        println!("{}", border_top);
        for (row_idx, row) in data.iter().enumerate() {
            if let Value::Array(cols) = row {
                let formatted: Vec<String> = cols.iter().enumerate().map(|(i, col)| {
                    let text = match col {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => format!("{}", n),
                        _ => String::new(),
                    };
                    format!("{:width$}", text, width = widths.get(i).copied().unwrap_or(0))
                }).collect();
                println!("│ {} │", formatted.join(" │ "));
            }
            if row_idx == 0 && data.len() > 1 {
                println!("{}", border_mid);
            }
        }
        println!("{}", border_bot);
        Ok(Value::Boolean(true))
    }
    fn clear(_args: &[Value]) -> MintasResult<Value> {
        print!("\x1b[2J\x1b[H");
        io::stdout().flush().ok();
        Ok(Value::Boolean(true))
    }
    fn newline(_args: &[Value]) -> MintasResult<Value> {
        println!();
        Ok(Value::Boolean(true))
    }
    fn separator(args: &[Value]) -> MintasResult<Value> {
        let char = args.get(0).and_then(|v| match v {
            Value::String(s) => s.chars().next(),
            _ => None,
        }).unwrap_or('─');
        let width = args.get(1).and_then(|v| match v {
            Value::Number(n) => Some(*n as usize),
            _ => None,
        }).unwrap_or(50);
        println!("{}", char.to_string().repeat(width));
        Ok(Value::Boolean(true))
    }
    fn box_display(args: &[Value]) -> MintasResult<Value> {
        let content = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let title = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        });
        let lines: Vec<&str> = content.lines().collect();
        let max_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let box_width = max_width.max(title.as_ref().map(|t| t.len()).unwrap_or(0)) + 4;
        if let Some(t) = &title {
            let padding = (box_width - t.len() - 4) / 2;
            println!("╭{}─ {} ─{}╮", "─".repeat(padding), t, "─".repeat(box_width - padding - t.len() - 4));
        } else {
            println!("╭{}╮", "─".repeat(box_width));
        }
        for line in lines {
            let padding = box_width - line.chars().count() - 2;
            println!("│ {}{} │", line, " ".repeat(padding));
        }
        println!("╰{}╯", "─".repeat(box_width));
        Ok(Value::Boolean(true))
    }
    fn panel(args: &[Value]) -> MintasResult<Value> {
        let content = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let style = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "info".to_string());
        let (color, icon) = match style.as_str() {
            "success" => ("\x1b[32m", "✅"),
            "error" => ("\x1b[31m", "❌"),
            "warning" => ("\x1b[33m", "⚠️"),
            "info" | _ => ("\x1b[34m", "ℹ️"),
        };
        println!("{}┌────────────────────────────────────────┐\x1b[0m", color);
        for line in content.lines() {
            println!("{}│ {} {:<36} │\x1b[0m", color, icon, line);
        }
        println!("{}└────────────────────────────────────────┘\x1b[0m", color);
        Ok(Value::Boolean(true))
    }
    fn divider(args: &[Value]) -> MintasResult<Value> {
        let text = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        });
        let width = args.get(1).and_then(|v| match v {
            Value::Number(n) => Some(*n as usize),
            _ => None,
        }).unwrap_or(60);
        if let Some(t) = text {
            let padding = (width - t.len() - 2) / 2;
            println!("{}─ {} ─{}", "─".repeat(padding), t, "─".repeat(width - padding - t.len() - 2));
        } else {
            println!("{}", "─".repeat(width));
        }
        Ok(Value::Boolean(true))
    }
    fn banner(args: &[Value]) -> MintasResult<Value> {
        let text = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let color = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "cyan".to_string());
        let color_code = match color.as_str() {
            "red" => "\x1b[31m",
            "green" => "\x1b[32m",
            "yellow" => "\x1b[33m",
            "blue" => "\x1b[34m",
            "magenta" => "\x1b[35m",
            "cyan" | _ => "\x1b[36m",
        };
        let width = text.len() + 8;
        println!("{}", color_code);
        println!("╔{}╗", "═".repeat(width));
        println!("║    {}    ║", text);
        println!("╚{}╝", "═".repeat(width));
        println!("\x1b[0m");
        Ok(Value::Boolean(true))
    }
    fn tree(args: &[Value]) -> MintasResult<Value> {
        let data = args.get(0).and_then(|v| match v {
            Value::Table(t) => Some(t.clone()),
            _ => None,
        }).unwrap_or_default();
        fn print_tree(data: &HashMap<String, Value>, prefix: &str, is_last: bool) {
            let connector = if is_last { "└── " } else { "├── " };
            let extension = if is_last { "    " } else { "│   " };
            let keys: Vec<_> = data.keys().collect();
            for (i, key) in keys.iter().enumerate() {
                let is_last_item = i == keys.len() - 1;
                let value = data.get(*key).unwrap();
                match value {
                    Value::Table(nested) => {
                        println!("{}{}{}/", prefix, if i == 0 && prefix.is_empty() { "" } else { connector }, key);
                        print_tree(nested, &format!("{}{}", prefix, extension), is_last_item);
                    }
                    Value::String(s) => {
                        println!("{}{}{}: {}", prefix, connector, key, s);
                    }
                    _ => {
                        println!("{}{}{}", prefix, connector, key);
                    }
                }
            }
        }
        print_tree(&data, "", true);
        Ok(Value::Boolean(true))
    }
    fn list(args: &[Value]) -> MintasResult<Value> {
        let items = args.get(0).and_then(|v| match v {
            Value::Array(arr) => Some(arr.clone()),
            _ => None,
        }).unwrap_or_default();
        let style = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "bullet".to_string());
        for (i, item) in items.iter().enumerate() {
            let marker = match style.as_str() {
                "number" => format!("{}.", i + 1),
                "letter" => format!("{}.", (b'a' + (i % 26) as u8) as char),
                "check" => "☐".to_string(),
                "arrow" => "→".to_string(),
                "bullet" | _ => "•".to_string(),
            };
            if let Value::String(s) = item {
                println!("  {} {}", marker, s);
            }
        }
        Ok(Value::Boolean(true))
    }
    fn task(args: &[Value]) -> MintasResult<Value> {
        let name = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Task".to_string());
        print!("\x1b[34m⏳ {}...\x1b[0m", name);
        io::stdout().flush().ok();
        let mut task = HashMap::new();
        task.insert("name".to_string(), Value::String(name));
        task.insert("status".to_string(), Value::String("running".to_string()));
        Ok(Value::Table(task))
    }
    fn task_done(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Done".to_string());
        println!("\r\x1b[32m✅ {}\x1b[0m", message);
        Ok(Value::Boolean(true))
    }
    fn task_fail(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Failed".to_string());
        println!("\r\x1b[31m❌ {}\x1b[0m", message);
        Ok(Value::Boolean(true))
    }
    fn task_skip(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Skipped".to_string());
        println!("\r\x1b[33m⏭️  {}\x1b[0m", message);
        Ok(Value::Boolean(true))
    }
    fn log(args: &[Value]) -> MintasResult<Value> {
        let level = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "info".to_string());
        let message = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let (color, label) = match level.to_lowercase().as_str() {
            "error" => ("\x1b[31m", "ERROR"),
            "warn" | "warning" => ("\x1b[33m", "WARN"),
            "debug" => ("\x1b[90m", "DEBUG"),
            "trace" => ("\x1b[90m", "TRACE"),
            "info" | _ => ("\x1b[34m", "INFO"),
        };
        println!("{}[{}] [{}] {}\x1b[0m", color, timestamp, label, message);
        Ok(Value::Boolean(true))
    }
    fn log_file(args: &[Value]) -> MintasResult<Value> {
        let path = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "app.log".to_string());
        let message = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_line = format!("[{}] {}\n", timestamp, message);
        use std::fs::OpenOptions;
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = file.write_all(log_line.as_bytes());
        }
        Ok(Value::Boolean(true))
    }
    fn log_level(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::String("info".to_string()))
    }
    fn autocomplete(args: &[Value]) -> MintasResult<Value> {
        let message = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Enter:".to_string());
        let suggestions = args.get(1).and_then(|v| match v {
            Value::Array(arr) => Some(arr.clone()),
            _ => None,
        }).unwrap_or_default();
        println!("{}", message);
        println!("\x1b[90mSuggestions: {}\x1b[0m", 
            suggestions.iter()
                .filter_map(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
                .collect::<Vec<_>>()
                .join(", "));
        print!("> ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        Ok(Value::String(input.trim().to_string()))
    }
    fn edit(args: &[Value]) -> MintasResult<Value> {
        let initial = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_default();
        let message = args.get(1).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }).unwrap_or_else(|| "Edit:".to_string());
        println!("{}", message);
        print!("[{}] > ", initial);
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim();
        if input.is_empty() {
            Ok(Value::String(initial))
        } else {
            Ok(Value::String(input.to_string()))
        }
    }
    fn history(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Array(Vec::new()))
    }
}