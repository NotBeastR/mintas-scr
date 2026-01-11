use crate::errors::{MintasError, MintasResult};
use crate::evaluator::Value;
use chrono::{DateTime, Utc, Local, NaiveDate, NaiveTime, NaiveDateTime, Duration, Datelike};
pub struct DateTimeModule;
impl DateTimeModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "now" => Self::now(args),
            "today" => Self::today(args),
            "utcnow" => Self::utcnow(args),
            "timestamp" => Self::timestamp(args),
            "fromtimestamp" => Self::fromtimestamp(args),
            "strptime" => Self::strptime(args),
            "strftime" => Self::strftime(args),
            "add_days" => Self::add_days(args),
            "add_hours" => Self::add_hours(args),
            "add_minutes" => Self::add_minutes(args),
            "add_seconds" => Self::add_seconds(args),
            "diff_days" => Self::diff_days(args),
            "diff_hours" => Self::diff_hours(args),
            "diff_minutes" => Self::diff_minutes(args),
            "diff_seconds" => Self::diff_seconds(args),
            "is_leap_year" => Self::is_leap_year(args),
            "days_in_month" => Self::days_in_month(args),
            "days_in_year" => Self::days_in_year(args),
            "weekday" => Self::weekday(args),
            "isoformat" => Self::isoformat(args),
            "parse" => Self::parse(args),
            _ => Err(MintasError::UnknownFunction {
                name: format!("datetime.{}", name),
                location: crate::errors::SourceLocation::new(0, 0),
            }),
        }
    }
    fn expect_number_arg(args: &[Value], index: usize, func_name: &str) -> MintasResult<f64> {
        if index >= args.len() {
            return Err(MintasError::InvalidArgumentCount {
                function: format!("datetime.{}", func_name),
                expected: index + 1,
                got: args.len(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        match &args[index] {
            Value::Number(n) => Ok(*n),
            _ => Err(MintasError::TypeError {
                message: format!("datetime.{} expects a number for argument {}", func_name, index + 1),
                location: crate::errors::SourceLocation::new(0, 0),
            }),
        }
    }
    fn expect_string_arg(args: &[Value], index: usize, func_name: &str) -> MintasResult<String> {
        if index >= args.len() {
            return Err(MintasError::InvalidArgumentCount {
                function: format!("datetime.{}", func_name),
                expected: index + 1,
                got: args.len(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        match &args[index] {
            Value::String(s) => Ok(s.clone()),
            _ => Err(MintasError::TypeError {
                message: format!("datetime.{} expects a string for argument {}", func_name, index + 1),
                location: crate::errors::SourceLocation::new(0, 0),
            }),
        }
    }
    fn now(_args: &[Value]) -> MintasResult<Value> {
        let now = Local::now();
        let timestamp = now.timestamp() as f64 + now.timestamp_subsec_micros() as f64 / 1_000_000.0;
        Ok(Value::Number(timestamp))
    }
    fn today(_args: &[Value]) -> MintasResult<Value> {
        let today = Local::now().date_naive();
        let timestamp = today.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as f64;
        Ok(Value::Number(timestamp))
    }
    fn utcnow(_args: &[Value]) -> MintasResult<Value> {
        let now = Utc::now();
        let timestamp = now.timestamp() as f64 + now.timestamp_subsec_micros() as f64 / 1_000_000.0;
        Ok(Value::Number(timestamp))
    }
    fn timestamp(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Self::now(args);
        }
        let year = Self::expect_number_arg(args, 0, "timestamp")? as i32;
        let month = Self::expect_number_arg(args, 1, "timestamp")? as u32;
        let day = Self::expect_number_arg(args, 2, "timestamp")? as u32;
        let hour = if args.len() > 3 { Self::expect_number_arg(args, 3, "timestamp")? as u32 } else { 0 };
        let minute = if args.len() > 4 { Self::expect_number_arg(args, 4, "timestamp")? as u32 } else { 0 };
        let second = if args.len() > 5 { Self::expect_number_arg(args, 5, "timestamp")? as u32 } else { 0 };
        let naive_date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid date".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let naive_time = NaiveTime::from_hms_opt(hour, minute, second)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid time".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let datetime = NaiveDateTime::new(naive_date, naive_time);
        Ok(Value::Number(datetime.and_utc().timestamp() as f64))
    }
    fn fromtimestamp(args: &[Value]) -> MintasResult<Value> {
        let timestamp = Self::expect_number_arg(args, 0, "fromtimestamp")?;
        let datetime = DateTime::from_timestamp(timestamp as i64, ((timestamp % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let local_datetime = datetime.with_timezone(&Local);
        Ok(Value::String(local_datetime.format("%Y-%m-%d %H:%M:%S").to_string()))
    }
    fn strptime(args: &[Value]) -> MintasResult<Value> {
        let date_string = Self::expect_string_arg(args, 0, "strptime")?;
        let format = Self::expect_string_arg(args, 1, "strptime")?;
        let naive_datetime = NaiveDateTime::parse_from_str(&date_string, &format)
            .map_err(|_| MintasError::RuntimeError {
                message: format!("Failed to parse date '{}' with format '{}'", date_string, format),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        Ok(Value::Number(naive_datetime.and_utc().timestamp() as f64))
    }
    fn strftime(args: &[Value]) -> MintasResult<Value> {
        let timestamp = Self::expect_number_arg(args, 0, "strftime")?;
        let format = Self::expect_string_arg(args, 1, "strftime")?;
        let datetime = DateTime::from_timestamp(timestamp as i64, ((timestamp % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let local_datetime = datetime.with_timezone(&Local);
        Ok(Value::String(local_datetime.format(&format).to_string()))
    }
    fn add_days(args: &[Value]) -> MintasResult<Value> {
        let timestamp = Self::expect_number_arg(args, 0, "add_days")?;
        let days = Self::expect_number_arg(args, 1, "add_days")?;
        let datetime = DateTime::from_timestamp(timestamp as i64, ((timestamp % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let new_datetime = datetime + Duration::days(days as i64);
        Ok(Value::Number(new_datetime.timestamp() as f64))
    }
    fn add_hours(args: &[Value]) -> MintasResult<Value> {
        let timestamp = Self::expect_number_arg(args, 0, "add_hours")?;
        let hours = Self::expect_number_arg(args, 1, "add_hours")?;
        let datetime = DateTime::from_timestamp(timestamp as i64, ((timestamp % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let new_datetime = datetime + Duration::hours(hours as i64);
        Ok(Value::Number(new_datetime.timestamp() as f64))
    }
    fn add_minutes(args: &[Value]) -> MintasResult<Value> {
        let timestamp = Self::expect_number_arg(args, 0, "add_minutes")?;
        let minutes = Self::expect_number_arg(args, 1, "add_minutes")?;
        let datetime = DateTime::from_timestamp(timestamp as i64, ((timestamp % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let new_datetime = datetime + Duration::minutes(minutes as i64);
        Ok(Value::Number(new_datetime.timestamp() as f64))
    }
    fn add_seconds(args: &[Value]) -> MintasResult<Value> {
        let timestamp = Self::expect_number_arg(args, 0, "add_seconds")?;
        let seconds = Self::expect_number_arg(args, 1, "add_seconds")?;
        let datetime = DateTime::from_timestamp(timestamp as i64, ((timestamp % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let new_datetime = datetime + Duration::seconds(seconds as i64);
        Ok(Value::Number(new_datetime.timestamp() as f64))
    }
    fn diff_days(args: &[Value]) -> MintasResult<Value> {
        let timestamp1 = Self::expect_number_arg(args, 0, "diff_days")?;
        let timestamp2 = Self::expect_number_arg(args, 1, "diff_days")?;
        let datetime1 = DateTime::from_timestamp(timestamp1 as i64, ((timestamp1 % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let datetime2 = DateTime::from_timestamp(timestamp2 as i64, ((timestamp2 % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let duration = datetime2 - datetime1;
        Ok(Value::Number(duration.num_days() as f64))
    }
    fn diff_hours(args: &[Value]) -> MintasResult<Value> {
        let timestamp1 = Self::expect_number_arg(args, 0, "diff_hours")?;
        let timestamp2 = Self::expect_number_arg(args, 1, "diff_hours")?;
        let datetime1 = DateTime::from_timestamp(timestamp1 as i64, ((timestamp1 % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let datetime2 = DateTime::from_timestamp(timestamp2 as i64, ((timestamp2 % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let duration = datetime2 - datetime1;
        Ok(Value::Number(duration.num_hours() as f64))
    }
    fn diff_minutes(args: &[Value]) -> MintasResult<Value> {
        let timestamp1 = Self::expect_number_arg(args, 0, "diff_minutes")?;
        let timestamp2 = Self::expect_number_arg(args, 1, "diff_minutes")?;
        let datetime1 = DateTime::from_timestamp(timestamp1 as i64, ((timestamp1 % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let datetime2 = DateTime::from_timestamp(timestamp2 as i64, ((timestamp2 % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let duration = datetime2 - datetime1;
        Ok(Value::Number(duration.num_minutes() as f64))
    }
    fn diff_seconds(args: &[Value]) -> MintasResult<Value> {
        let timestamp1 = Self::expect_number_arg(args, 0, "diff_seconds")?;
        let timestamp2 = Self::expect_number_arg(args, 1, "diff_seconds")?;
        let duration = (timestamp2 - timestamp1).abs();
        Ok(Value::Number(duration))
    }
    fn is_leap_year(args: &[Value]) -> MintasResult<Value> {
        let year = Self::expect_number_arg(args, 0, "is_leap_year")? as i32;
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        Ok(Value::Boolean(is_leap))
    }
    fn days_in_month(args: &[Value]) -> MintasResult<Value> {
        let year = Self::expect_number_arg(args, 0, "days_in_month")? as i32;
        let month = Self::expect_number_arg(args, 1, "days_in_month")? as u32;
        let date = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid year/month".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let days = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - date
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - date
        };
        Ok(Value::Number(days.num_days() as f64))
    }
    fn days_in_year(args: &[Value]) -> MintasResult<Value> {
        let year = Self::expect_number_arg(args, 0, "days_in_year")? as i32;
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        Ok(Value::Number(if is_leap { 366.0 } else { 365.0 }))
    }
    fn weekday(args: &[Value]) -> MintasResult<Value> {
        let timestamp = Self::expect_number_arg(args, 0, "weekday")?;
        let datetime = DateTime::from_timestamp(timestamp as i64, ((timestamp % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        let local_datetime = datetime.with_timezone(&Local);
        let weekday_num = local_datetime.weekday().num_days_from_monday() as f64;
        Ok(Value::Number(weekday_num))
    }
    fn isoformat(args: &[Value]) -> MintasResult<Value> {
        let timestamp = Self::expect_number_arg(args, 0, "isoformat")?;
        let datetime = DateTime::from_timestamp(timestamp as i64, ((timestamp % 1.0) * 1_000_000_000.0) as u32)
            .ok_or_else(|| MintasError::RuntimeError {
                message: "Invalid timestamp".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            })?;
        Ok(Value::String(datetime.to_rfc3339()))
    }
    fn parse(args: &[Value]) -> MintasResult<Value> {
        let date_string = Self::expect_string_arg(args, 0, "parse")?;
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S%.fZ",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%d",
            "%H:%M:%S",
            "%Y/%m/%d %H:%M:%S",
            "%m/%d/%Y %H:%M:%S",
            "%d/%m/%Y %H:%M:%S",
        ];
        for format in &formats {
            if let Ok(datetime) = NaiveDateTime::parse_from_str(&date_string, format) {
                return Ok(Value::Number(datetime.and_utc().timestamp() as f64));
            }
        }
        Err(MintasError::RuntimeError {
            message: format!("Failed to parse date string: '{}'", date_string),
            location: crate::errors::SourceLocation::new(0, 0),
        })
    }
}