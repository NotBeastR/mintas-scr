use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
pub struct AlgorithmModule;
impl AlgorithmModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "sort" => Self::sort(args),
            "quicksort" => Self::quicksort(args),
            "mergesort" => Self::mergesort(args),
            "bubblesort" => Self::bubblesort(args),
            "binary_search" => Self::binary_search(args),
            "linear_search" => Self::linear_search(args),
            "find" => Self::find(args),
            "reverse" => Self::reverse(args),
            "shuffle" => Self::shuffle(args),
            "unique" => Self::unique(args),
            "flatten" => Self::flatten(args),
            "chunk" => Self::chunk(args),
            "zip" => Self::zip(args),
            "gcd" => Self::gcd(args),
            "lcm" => Self::lcm(args),
            "factorial" => Self::factorial(args),
            "fibonacci" => Self::fibonacci(args),
            "prime" => Self::is_prime(args),
            "primes" => Self::primes(args),
            "levenshtein" => Self::levenshtein(args),
            "palindrome" => Self::is_palindrome(args),
            "anagram" => Self::is_anagram(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown algorithm function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn sort(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.sort".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        Self::quicksort(args)
    }
    fn quicksort(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.quicksort".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let arr = match &args[0] {
            Value::Array(a) => a.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Expected array".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let mut nums: Vec<f64> = arr.iter().filter_map(|v| {
            if let Value::Number(n) = v { Some(*n) } else { None }
        }).collect();
        nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Ok(Value::Array(nums.into_iter().map(Value::Number).collect()))
    }
    fn mergesort(args: &[Value]) -> MintasResult<Value> {
        Self::quicksort(args)
    }
    fn bubblesort(args: &[Value]) -> MintasResult<Value> {
        Self::quicksort(args)
    }
    fn binary_search(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.binary_search".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let arr = match &args[0] {
            Value::Array(a) => a.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Expected array".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let target = match &args[1] {
            Value::Number(n) => *n,
            _ => return Err(MintasError::TypeError {
                message: "Expected number".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let nums: Vec<f64> = arr.iter().filter_map(|v| {
            if let Value::Number(n) = v { Some(*n) } else { None }
        }).collect();
        match nums.binary_search_by(|x| x.partial_cmp(&target).unwrap_or(std::cmp::Ordering::Equal)) {
            Ok(idx) => Ok(Value::Number((idx + 1) as f64)), 
            Err(_) => Ok(Value::Number(-1.0)),
        }
    }
    fn linear_search(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.linear_search".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let arr = match &args[0] {
            Value::Array(a) => a.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Expected array".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        for (i, item) in arr.iter().enumerate() {
            if Self::values_equal(item, &args[1]) {
                return Ok(Value::Number((i + 1) as f64)); 
            }
        }
        Ok(Value::Number(-1.0))
    }
    fn find(args: &[Value]) -> MintasResult<Value> {
        Self::linear_search(args)
    }
    fn reverse(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.reverse".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        match &args[0] {
            Value::Array(a) => {
                let mut reversed = a.clone();
                reversed.reverse();
                Ok(Value::Array(reversed))
            }
            Value::String(s) => Ok(Value::String(s.chars().rev().collect())),
            _ => Err(MintasError::TypeError {
                message: "Expected array or string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn shuffle(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.shuffle".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let arr = match &args[0] {
            Value::Array(a) => a.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Expected array".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as usize;
        let mut shuffled = arr;
        for i in (1..shuffled.len()).rev() {
            let j = (seed + i) % (i + 1);
            shuffled.swap(i, j);
        }
        Ok(Value::Array(shuffled))
    }
    fn unique(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.unique".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let arr = match &args[0] {
            Value::Array(a) => a.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Expected array".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let mut seen = Vec::new();
        for item in arr {
            if !seen.iter().any(|x| Self::values_equal(x, &item)) {
                seen.push(item);
            }
        }
        Ok(Value::Array(seen))
    }
    fn flatten(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.flatten".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        fn flatten_recursive(val: &Value, result: &mut Vec<Value>) {
            match val {
                Value::Array(arr) => {
                    for item in arr {
                        flatten_recursive(item, result);
                    }
                }
                other => result.push(other.clone()),
            }
        }
        let mut result = Vec::new();
        flatten_recursive(&args[0], &mut result);
        Ok(Value::Array(result))
    }
    fn chunk(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.chunk".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let arr = match &args[0] {
            Value::Array(a) => a.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Expected array".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let size = match &args[1] {
            Value::Number(n) => *n as usize,
            _ => return Err(MintasError::TypeError {
                message: "Expected number".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        if size == 0 {
            return Ok(Value::Array(Vec::new()));
        }
        let chunks: Vec<Value> = arr.chunks(size)
            .map(|c| Value::Array(c.to_vec()))
            .collect();
        Ok(Value::Array(chunks))
    }
    fn zip(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.zip".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let arr1 = match &args[0] {
            Value::Array(a) => a.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Expected array".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let arr2 = match &args[1] {
            Value::Array(a) => a.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Expected array".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let zipped: Vec<Value> = arr1.into_iter().zip(arr2.into_iter())
            .map(|(a, b)| Value::Array(vec![a, b]))
            .collect();
        Ok(Value::Array(zipped))
    }
    fn gcd(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.gcd".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let a = match &args[0] {
            Value::Number(n) => *n as i64,
            _ => return Err(MintasError::TypeError {
                message: "Expected number".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let b = match &args[1] {
            Value::Number(n) => *n as i64,
            _ => return Err(MintasError::TypeError {
                message: "Expected number".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        fn gcd_impl(mut a: i64, mut b: i64) -> i64 {
            while b != 0 {
                let t = b;
                b = a % b;
                a = t;
            }
            a.abs()
        }
        Ok(Value::Number(gcd_impl(a, b) as f64))
    }
    fn lcm(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.lcm".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let a = match &args[0] {
            Value::Number(n) => *n as i64,
            _ => return Err(MintasError::TypeError {
                message: "Expected number".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let b = match &args[1] {
            Value::Number(n) => *n as i64,
            _ => return Err(MintasError::TypeError {
                message: "Expected number".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        fn gcd_impl(mut a: i64, mut b: i64) -> i64 {
            while b != 0 {
                let t = b;
                b = a % b;
                a = t;
            }
            a.abs()
        }
        let result = (a * b).abs() / gcd_impl(a, b);
        Ok(Value::Number(result as f64))
    }
    fn factorial(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.factorial".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let n = match &args[0] {
            Value::Number(num) => *num as u64,
            _ => return Err(MintasError::TypeError {
                message: "Expected number".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let result: u64 = (1..=n).product();
        Ok(Value::Number(result as f64))
    }
    fn fibonacci(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.fibonacci".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let n = match &args[0] {
            Value::Number(num) => *num as usize,
            _ => return Err(MintasError::TypeError {
                message: "Expected number".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        if n == 0 { return Ok(Value::Number(0.0)); }
        if n == 1 { return Ok(Value::Number(1.0)); }
        let mut a = 0u64;
        let mut b = 1u64;
        for _ in 2..=n {
            let c = a + b;
            a = b;
            b = c;
        }
        Ok(Value::Number(b as f64))
    }
    fn is_prime(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.prime".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let n = match &args[0] {
            Value::Number(num) => *num as u64,
            _ => return Err(MintasError::TypeError {
                message: "Expected number".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        if n < 2 { return Ok(Value::Boolean(false)); }
        if n == 2 { return Ok(Value::Boolean(true)); }
        if n % 2 == 0 { return Ok(Value::Boolean(false)); }
        let sqrt = (n as f64).sqrt() as u64;
        for i in (3..=sqrt).step_by(2) {
            if n % i == 0 { return Ok(Value::Boolean(false)); }
        }
        Ok(Value::Boolean(true))
    }
    fn primes(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.primes".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let n = match &args[0] {
            Value::Number(num) => *num as usize,
            _ => return Err(MintasError::TypeError {
                message: "Expected number".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        if n < 2 { return Ok(Value::Array(Vec::new())); }
        let mut sieve = vec![true; n + 1];
        sieve[0] = false;
        sieve[1] = false;
        for i in 2..=((n as f64).sqrt() as usize) {
            if sieve[i] {
                for j in (i * i..=n).step_by(i) {
                    sieve[j] = false;
                }
            }
        }
        let primes: Vec<Value> = sieve.iter().enumerate()
            .filter(|(_, &is_prime)| is_prime)
            .map(|(i, _)| Value::Number(i as f64))
            .collect();
        Ok(Value::Array(primes))
    }
    fn levenshtein(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.levenshtein".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let s1 = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Expected string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let s2 = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err(MintasError::TypeError {
                message: "Expected string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let v1: Vec<char> = s1.chars().collect();
        let v2: Vec<char> = s2.chars().collect();
        let m = v1.len();
        let n = v2.len();
        let mut dp = vec![vec![0usize; n + 1]; m + 1];
        for i in 0..=m { dp[i][0] = i; }
        for j in 0..=n { dp[0][j] = j; }
        for i in 1..=m {
            for j in 1..=n {
                let cost = if v1[i - 1] == v2[j - 1] { 0 } else { 1 };
                dp[i][j] = (dp[i - 1][j] + 1)
                    .min(dp[i][j - 1] + 1)
                    .min(dp[i - 1][j - 1] + cost);
            }
        }
        Ok(Value::Number(dp[m][n] as f64))
    }
    fn is_palindrome(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.palindrome".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        let s = match &args[0] {
            Value::String(s) => s.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect::<String>(),
            _ => return Err(MintasError::TypeError {
                message: "Expected string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let reversed: String = s.chars().rev().collect();
        Ok(Value::Boolean(s == reversed))
    }
    fn is_anagram(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "algorithm.anagram".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let s1 = match &args[0] {
            Value::String(s) => s.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect::<Vec<_>>(),
            _ => return Err(MintasError::TypeError {
                message: "Expected string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let s2 = match &args[1] {
            Value::String(s) => s.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect::<Vec<_>>(),
            _ => return Err(MintasError::TypeError {
                message: "Expected string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let mut sorted1 = s1;
        let mut sorted2 = s2;
        sorted1.sort();
        sorted2.sort();
        Ok(Value::Boolean(sorted1 == sorted2))
    }
    fn values_equal(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            _ => false,
        }
    }
}