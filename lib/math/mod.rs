use crate::errors::{MintasError, MintasResult};
use crate::evaluator::Value;
pub struct MathModule;
impl MathModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "sin" => Self::sin(args),
            "cos" => Self::cos(args),
            "tan" => Self::tan(args),
            "asin" => Self::asin(args),
            "acos" => Self::acos(args),
            "atan" => Self::atan(args),
            "atan2" => Self::atan2(args),
            "sinh" => Self::sinh(args),
            "cosh" => Self::cosh(args),
            "tanh" => Self::tanh(args),
            "sqrt" => Self::sqrt(args),
            "cbrt" => Self::cbrt(args),
            "pow" => Self::pow(args),
            "exp" => Self::exp(args),
            "exp2" => Self::exp2(args),
            "ln" => Self::ln(args),
            "log10" => Self::log10(args),
            "log2" => Self::log2(args),
            "abs" => Self::abs(args),
            "floor" => Self::floor(args),
            "ceil" => Self::ceil(args),
            "round" => Self::round(args),
            "trunc" => Self::trunc(args),
            "min" => Self::min(args),
            "max" => Self::max(args),
            "random" => Self::random(args),
            "pi" => Self::pi(args),
            "e" => Self::e(args),
            _ => Err(MintasError::UnknownFunction {
                name: format!("math.{}", name),
                location: crate::errors::SourceLocation::new(0, 0),
            }),
        }
    }
    fn expect_number_arg(args: &[Value], index: usize, func_name: &str) -> MintasResult<f64> {
        if index >= args.len() {
            return Err(MintasError::InvalidArgumentCount {
                function: format!("math.{}", func_name),
                expected: index + 1,
                got: args.len(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        match &args[index] {
            Value::Number(n) => Ok(*n),
            _ => Err(MintasError::TypeError {
                message: format!("math.{} expects a number for argument {}", func_name, index + 1),
                location: crate::errors::SourceLocation::new(0, 0),
            }),
        }
    }
    fn sin(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "sin")?;
        Ok(Value::Number(x.sin()))
    }
    fn cos(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "cos")?;
        Ok(Value::Number(x.cos()))
    }
    fn tan(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "tan")?;
        Ok(Value::Number(x.tan()))
    }
    fn asin(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "asin")?;
        if x < -1.0 || x > 1.0 {
            return Err(MintasError::RuntimeError {
                message: "asin: input must be between -1 and 1".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Number(x.asin()))
    }
    fn acos(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "acos")?;
        if x < -1.0 || x > 1.0 {
            return Err(MintasError::RuntimeError {
                message: "acos: input must be between -1 and 1".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Number(x.acos()))
    }
    fn atan(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "atan")?;
        Ok(Value::Number(x.atan()))
    }
    fn atan2(args: &[Value]) -> MintasResult<Value> {
        let y = Self::expect_number_arg(args, 0, "atan2")?;
        let x = Self::expect_number_arg(args, 1, "atan2")?;
        Ok(Value::Number(y.atan2(x)))
    }
    fn sinh(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "sinh")?;
        Ok(Value::Number(x.sinh()))
    }
    fn cosh(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "cosh")?;
        Ok(Value::Number(x.cosh()))
    }
    fn tanh(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "tanh")?;
        Ok(Value::Number(x.tanh()))
    }
    fn sqrt(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "sqrt")?;
        if x < 0.0 {
            return Err(MintasError::RuntimeError {
                message: "sqrt: cannot take square root of negative number".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Number(x.sqrt()))
    }
    fn cbrt(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "cbrt")?;
        Ok(Value::Number(x.cbrt()))
    }
    fn pow(args: &[Value]) -> MintasResult<Value> {
        let base = Self::expect_number_arg(args, 0, "pow")?;
        let exp = Self::expect_number_arg(args, 1, "pow")?;
        Ok(Value::Number(base.powf(exp)))
    }
    fn exp(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "exp")?;
        Ok(Value::Number(x.exp()))
    }
    fn exp2(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "exp2")?;
        Ok(Value::Number(x.exp2()))
    }
    fn ln(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "ln")?;
        if x <= 0.0 {
            return Err(MintasError::RuntimeError {
                message: "ln: input must be positive".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Number(x.ln()))
    }
    fn log10(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "log10")?;
        if x <= 0.0 {
            return Err(MintasError::RuntimeError {
                message: "log10: input must be positive".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Number(x.log10()))
    }
    fn log2(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "log2")?;
        if x <= 0.0 {
            return Err(MintasError::RuntimeError {
                message: "log2: input must be positive".to_string(),
                location: crate::errors::SourceLocation::new(0, 0),
            });
        }
        Ok(Value::Number(x.log2()))
    }
    fn abs(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "abs")?;
        Ok(Value::Number(x.abs()))
    }
    fn floor(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "floor")?;
        Ok(Value::Number(x.floor()))
    }
    fn ceil(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "ceil")?;
        Ok(Value::Number(x.ceil()))
    }
    fn round(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "round")?;
        Ok(Value::Number(x.round()))
    }
    fn trunc(args: &[Value]) -> MintasResult<Value> {
        let x = Self::expect_number_arg(args, 0, "trunc")?;
        Ok(Value::Number(x.trunc()))
    }
    fn min(args: &[Value]) -> MintasResult<Value> {
        let a = Self::expect_number_arg(args, 0, "min")?;
        let b = Self::expect_number_arg(args, 1, "min")?;
        Ok(Value::Number(a.min(b)))
    }
    fn max(args: &[Value]) -> MintasResult<Value> {
        let a = Self::expect_number_arg(args, 0, "max")?;
        let b = Self::expect_number_arg(args, 1, "max")?;
        Ok(Value::Number(a.max(b)))
    }
    fn random(args: &[Value]) -> MintasResult<Value> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let result = match args.len() {
            0 => {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                (timestamp % 1000000) as f64 / 1000000.0
            }
            1 => {
                let max = Self::expect_number_arg(args, 0, "random")?;
                if max <= 0.0 {
                    return Err(MintasError::RuntimeError {
                        message: "random: max value must be positive".to_string(),
                        location: crate::errors::SourceLocation::new(0, 0),
                    });
                }
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                ((timestamp % 1000000) as f64 / 1000000.0) * max
            }
            2 => {
                let min = Self::expect_number_arg(args, 0, "random")?;
                let max = Self::expect_number_arg(args, 1, "random")?;
                if min >= max {
                    return Err(MintasError::RuntimeError {
                        message: "random: min must be less than max".to_string(),
                        location: crate::errors::SourceLocation::new(0, 0),
                    });
                }
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let rand = (timestamp % 1000000) as f64 / 1000000.0;
                min + rand * (max - min)
            }
            _ => {
                return Err(MintasError::InvalidArgumentCount {
                    function: "math.random".to_string(),
                    expected: 2,
                    got: args.len(),
                    location: crate::errors::SourceLocation::new(0, 0),
                });
            }
        };
        Ok(Value::Number(result))
    }
    fn pi(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Number(std::f64::consts::PI))
    }
    fn e(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Number(std::f64::consts::E))
    }
}