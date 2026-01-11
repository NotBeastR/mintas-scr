use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
pub struct CronModule;
static JOB_COUNTER: AtomicU64 = AtomicU64::new(1);
impl CronModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "schedule" => Self::schedule(args),
            "parse" => Self::parse(args),
            "next" => Self::next(args),
            "validate" => Self::validate(args),
            "list" => Self::list(args),
            "cancel" => Self::cancel(args),
            "pause" => Self::pause(args),
            "resume" => Self::resume(args),
            "every" => Self::every(args),
            "at" => Self::at(args),
            "daily" => Self::daily(args),
            "hourly" => Self::hourly(args),
            "weekly" => Self::weekly(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown cron function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn schedule(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "cron.schedule".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let expression = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Cron expression must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let name = if args.len() > 2 {
            match &args[2] {
                Value::String(s) => s.clone(),
                _ => format!("job_{}", JOB_COUNTER.fetch_add(1, Ordering::SeqCst)),
            }
        } else {
            format!("job_{}", JOB_COUNTER.fetch_add(1, Ordering::SeqCst))
        };
        if !Self::is_valid_cron(&expression) {
            return Err(MintasError::RuntimeError {
                message: format!("Invalid cron expression: {}", expression),
                location: SourceLocation::new(0, 0),
            });
        }
        let mut job = HashMap::new();
        job.insert("id".to_string(), Value::String(name.clone()));
        job.insert("expression".to_string(), Value::String(expression.clone()));
        job.insert("active".to_string(), Value::Boolean(true));
        job.insert("next_run".to_string(), Value::String(Self::calculate_next_run(&expression)));
        job.insert("created_at".to_string(), Value::Number(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as f64
        ));
        Ok(Value::Table(job))
    }
    fn parse(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "cron.parse".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let expression = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Cron expression must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let parts: Vec<&str> = expression.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(MintasError::RuntimeError {
                message: "Cron expression must have 5 fields".to_string(),
                location: SourceLocation::new(0, 0),
            });
        }
        let mut result = HashMap::new();
        result.insert("minute".to_string(), Value::String(parts[0].to_string()));
        result.insert("hour".to_string(), Value::String(parts[1].to_string()));
        result.insert("day_of_month".to_string(), Value::String(parts[2].to_string()));
        result.insert("month".to_string(), Value::String(parts[3].to_string()));
        result.insert("day_of_week".to_string(), Value::String(parts[4].to_string()));
        result.insert("valid".to_string(), Value::Boolean(Self::is_valid_cron(&expression)));
        Ok(Value::Table(result))
    }
    fn next(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "cron.next".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let expression = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Cron expression must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let count = if args.len() > 1 {
            match &args[1] {
                Value::Number(n) => *n as usize,
                _ => 1,
            }
        } else {
            1
        };
        let mut runs = Vec::new();
        for i in 0..count {
            runs.push(Value::String(format!("Next run #{}: {}", i + 1, Self::calculate_next_run(&expression))));
        }
        if count == 1 {
            Ok(runs.into_iter().next().unwrap_or(Value::Empty))
        } else {
            Ok(Value::Array(runs))
        }
    }
    fn validate(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "cron.validate".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let expression = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Ok(Value::Boolean(false)),
        };
        Ok(Value::Boolean(Self::is_valid_cron(&expression)))
    }
    fn list(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Array(Vec::new()))
    }
    fn cancel(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "cron.cancel".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Boolean(true))
    }
    fn pause(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "cron.pause".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Boolean(true))
    }
    fn resume(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "cron.resume".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Boolean(true))
    }
    fn every(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "cron.every".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let interval = match &args[0] {
            Value::Number(n) => *n as u32,
            _ => return Err(MintasError::TypeError {
                message: "Interval must be a number".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let unit = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Unit must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let expression = match unit.to_lowercase().as_str() {
            "minutes" | "minute" | "min" => format!("*/{} * * * *", interval),
            "hours" | "hour" | "hr" => format!("0 */{} * * *", interval),
            "days" | "day" => format!("0 0 */{} * *", interval),
            _ => "* * * * *".to_string(),
        };
        let mut job = HashMap::new();
        job.insert("expression".to_string(), Value::String(expression));
        job.insert("interval".to_string(), Value::Number(interval as f64));
        job.insert("unit".to_string(), Value::String(unit));
        Ok(Value::Table(job))
    }
    fn at(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "cron.at".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let time = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Time must be a string (HH:MM)".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let parts: Vec<&str> = time.split(':').collect();
        let (hour, minute) = if parts.len() >= 2 {
            (parts[0].parse::<u32>().unwrap_or(0), parts[1].parse::<u32>().unwrap_or(0))
        } else {
            (0, 0)
        };
        let expression = format!("{} {} * * *", minute, hour);
        let mut job = HashMap::new();
        job.insert("expression".to_string(), Value::String(expression));
        job.insert("time".to_string(), Value::String(time));
        Ok(Value::Table(job))
    }
    fn daily(args: &[Value]) -> MintasResult<Value> {
        let time = if !args.is_empty() {
            match &args[0] {
                Value::String(s) => s.clone(),
                _ => "00:00".to_string(),
            }
        } else {
            "00:00".to_string()
        };
        let parts: Vec<&str> = time.split(':').collect();
        let (hour, minute) = if parts.len() >= 2 {
            (parts[0].parse::<u32>().unwrap_or(0), parts[1].parse::<u32>().unwrap_or(0))
        } else {
            (0, 0)
        };
        Ok(Value::String(format!("{} {} * * *", minute, hour)))
    }
    fn hourly(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::String("0 * * * *".to_string()))
    }
    fn weekly(args: &[Value]) -> MintasResult<Value> {
        let day = if !args.is_empty() {
            match &args[0] {
                Value::Number(n) => *n as u32,
                Value::String(s) => match s.to_lowercase().as_str() {
                    "sunday" | "sun" => 0,
                    "monday" | "mon" => 1,
                    "tuesday" | "tue" => 2,
                    "wednesday" | "wed" => 3,
                    "thursday" | "thu" => 4,
                    "friday" | "fri" => 5,
                    "saturday" | "sat" => 6,
                    _ => 0,
                },
                _ => 0,
            }
        } else {
            0
        };
        Ok(Value::String(format!("0 0 * * {}", day)))
    }
    fn is_valid_cron(expression: &str) -> bool {
        let parts: Vec<&str> = expression.split_whitespace().collect();
        parts.len() == 5
    }
    fn calculate_next_run(_expression: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH, Duration};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let next = now + Duration::from_secs(60);
        format!("{}", next.as_secs())
    }
}