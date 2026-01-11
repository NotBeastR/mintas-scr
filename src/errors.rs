use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug, Clone)]
pub enum MintasError {
    #[allow(dead_code)]
    LexerError { message: String, location: SourceLocation },
    UnterminatedString { location: SourceLocation },
    InvalidEscapeSequence { sequence: String, location: SourceLocation },
    InvalidCharacter { character: char, location: SourceLocation },
    ParseError { message: String, location: SourceLocation },
    UnexpectedToken { expected: String, found: String, location: SourceLocation },
    UnexpectedEndOfInput { location: SourceLocation },
    InvalidVariableName { name: String, reason: String, location: SourceLocation },
    MissingAssignment { keyword: String, location: SourceLocation },
    RuntimeError { message: String, location: SourceLocation },
    CompileError { message: String, location: SourceLocation },
    TypeError { message: String, location: SourceLocation },
    UndefinedVariable { name: String, location: SourceLocation },
    DivisionByZero { location: SourceLocation },
    #[allow(dead_code)]
    InvalidAssignment { message: String, location: SourceLocation },
    ConstantReassignment { name: String, location: SourceLocation },
    UnknownFunction { name: String, location: SourceLocation },
    InvalidArgumentCount { function: String, expected: usize, got: usize, location: SourceLocation },
    #[allow(dead_code)]
    InvalidOperand { operation: String, operand_type: String, location: SourceLocation },
}

impl MintasError {
    #[allow(dead_code)]
    pub fn lexer_error(message: impl Into<String>, line: usize, column: usize) -> Self {
        MintasError::LexerError {
            message: message.into(),
            location: SourceLocation::new(line, column),
        }
    }

    #[allow(dead_code)]
    pub fn parse_error(message: impl Into<String>, line: usize, column: usize) -> Self {
        MintasError::ParseError {
            message: message.into(),
            location: SourceLocation::new(line, column),
        }
    }

    #[allow(dead_code)]
    pub fn runtime_error(message: impl Into<String>, line: usize, column: usize) -> Self {
        MintasError::RuntimeError {
            message: message.into(),
            location: SourceLocation::new(line, column),
        }
    }

    #[allow(dead_code)]
    pub fn location(&self) -> &SourceLocation {
        match self {
            MintasError::LexerError { location, .. } => location,
            MintasError::UnterminatedString { location } => location,
            MintasError::InvalidEscapeSequence { location, .. } => location,
            MintasError::InvalidCharacter { location, .. } => location,
            MintasError::ParseError { location, .. } => location,
            MintasError::UnexpectedToken { location, .. } => location,
            MintasError::UnexpectedEndOfInput { location } => location,
            MintasError::InvalidVariableName { location, .. } => location,
            MintasError::MissingAssignment { location, .. } => location,
            MintasError::RuntimeError { location, .. } => location,
            MintasError::TypeError { location, .. } => location,
            MintasError::UndefinedVariable { location, .. } => location,
            MintasError::DivisionByZero { location } => location,
            MintasError::InvalidAssignment { location, .. } => location,
            MintasError::ConstantReassignment { location, .. } => location,
            MintasError::UnknownFunction { location, .. } => location,
            MintasError::InvalidArgumentCount { location, .. } => location,
            MintasError::InvalidOperand { location, .. } => location,
            MintasError::CompileError { location, .. } => location,
        }
    }
}

impl fmt::Display for MintasError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MintasError::LexerError { message, location } => {
                write!(f, "Lexer Error at {}: {}", location, message)?;
                write_suggestions(f, self)
            }
            MintasError::UnterminatedString { location } => {
                write!(f, "Syntax Error at {}: Unterminated string literal", location)?;
                writeln!(f, "\nSuggestion: Make sure to close string literals with matching quotes.")?;
                write_suggestions(f, self)
            }
            MintasError::InvalidEscapeSequence { sequence, location } => {
                write!(f, "Syntax Error at {}: Invalid escape sequence '{}'", location, sequence)?;
                writeln!(f, "\nSuggestion: Valid escape sequences are \\n (newline), \\t (tab), \\\" (quote), and \\\\ (backslash).")?;
                write_suggestions(f, self)
            }
            MintasError::InvalidCharacter { character, location } => {
                write!(f, "Syntax Error at {}: Invalid character '{}'", location, character)?;
                write_suggestions(f, self)
            }
            MintasError::ParseError { message, location } => {
                write!(f, "Parse Error at {}: {}", location, message)?;
                write_suggestions(f, self)
            }
            MintasError::UnexpectedToken { expected, found, location } => {
                write!(f, "Syntax Error at {}: Expected {}, but found {}", location, expected, found)?;
                write_suggestions(f, self)
            }
            MintasError::UnexpectedEndOfInput { location } => {
                write!(f, "Syntax Error at {}: Unexpected end of input", location)?;
                writeln!(f, "\nSuggestion: Check for missing closing brackets, braces, or 'end' keywords.")?;
                write_suggestions(f, self)
            }
            MintasError::InvalidVariableName { name, reason, location } => {
                write!(f, "Syntax Error at {}: Invalid variable name '{}': {}", location, name, reason)?;
                writeln!(f, "\nSuggestion: Variable names must start with a letter and contain only letters, numbers, and underscores.")?;
                write_suggestions(f, self)
            }
            MintasError::MissingAssignment { keyword, location } => {
                write!(f, "Syntax Error at {}: '{}' must be followed by a variable assignment", location, keyword)?;
                match keyword.as_str() {
                    "let" => writeln!(f, "\nSuggestion: Use 'let x = value' to declare a variable.")?,
                    "const" | "consta" => writeln!(f, "\nSuggestion: Use 'const PI = 3.14' to declare a constant.")?,
                    "so" => writeln!(f, "\nSuggestion: Use 'so x = value' as an alias for 'let x = value'.")?,
                    _ => {}
                }
                write_suggestions(f, self)
            }
            MintasError::RuntimeError { message, location } => {
                write!(f, "Runtime Error at {}: {}", location, message)?;
                write_suggestions(f, self)
            }
            MintasError::TypeError { message, location } => {
                write!(f, "Type Error at {}: {}", location, message)?;
                write_suggestions(f, self)
            }
            MintasError::UndefinedVariable { name, location } => {
                write!(f, "Runtime Error at {}: Undefined variable '{}'", location, name)?;
                writeln!(f, "\nSuggestion: Make sure the variable is declared before use. Check for typos in the variable name.")?;
                write_suggestions(f, self)
            }
            MintasError::DivisionByZero { location } => {
                write!(f, "Runtime Error at {}: Division by zero", location)?;
                writeln!(f, "\nSuggestion: Check the divisor value to ensure it's not zero before division.")?;
                write_suggestions(f, self)
            }
            MintasError::InvalidAssignment { message, location } => {
                write!(f, "Runtime Error at {}: Invalid assignment: {}", location, message)?;
                write_suggestions(f, self)
            }
            MintasError::ConstantReassignment { name, location } => {
                write!(f, "Runtime Error at {}: Cannot reassign constant '{}'", location, name)?;
                writeln!(f, "\nSuggestion: Use 'let' instead of 'const' if you need to reassign this variable.")?;
                write_suggestions(f, self)
            }
            MintasError::UnknownFunction { name, location } => {
                write!(f, "Runtime Error at {}: Unknown function '{}'", location, name)?;
                let suggestions = get_function_suggestions(name);
                if !suggestions.is_empty() {
                    writeln!(f, "\nDid you mean one of these functions: {}?", suggestions.join(", "))?;
                }
                write_suggestions(f, self)
            }
            MintasError::InvalidArgumentCount { function, expected, got, location } => {
                write!(f, "Runtime Error at {}: Function '{}' expects {} argument(s), but got {}",
                    location, function, expected, got)?;
                write_suggestions(f, self)
            }
            MintasError::InvalidOperand { operation, operand_type, location } => {
                write!(f, "Type Error at {}: {} does not support operand of type '{}'",
                    location, operation, operand_type)?;
                write_suggestions(f, self)
            }
            MintasError::CompileError { message, location } => {
                write!(f, "Compile Error at {}: {}", location, message)?;
                write_suggestions(f, self)
            }
        }
    }
}

impl std::error::Error for MintasError {}

// Add From trait implementation for io::Error
impl From<std::io::Error> for MintasError {
    fn from(error: std::io::Error) -> Self {
        MintasError::RuntimeError {
            message: format!("I/O error: {}", error),
            location: SourceLocation::new(0, 0),
        }
    }
}

pub type MintasResult<T> = Result<T, MintasError>;

fn write_suggestions(f: &mut fmt::Formatter, _error: &MintasError) -> fmt::Result {
    writeln!(f, "\nFor more help, type 'help' in the REPL or check the documentation.")
}

fn get_function_suggestions(name: &str) -> Vec<&'static str> {
    let built_in_functions = [
        "say", "ask", "len", "upper", "lower", "trim", "push", "pop", "insert", "remove",
        "sort", "reverse", "contains", "find", "replace", "split", "join", "slice",
        "keys", "values", "has", "merge", "read", "write", "append", "exists",
        "typeof", "tostring", "tonumber"
    ];

    let mut suggestions = Vec::new();
    for func in &built_in_functions {
        if levenshtein_distance(name, func) <= 2 {
            suggestions.push(*func);
        }
    }

    suggestions
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1.chars().nth(i - 1) == s2.chars().nth(j - 1) { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost
            );
        }
    }

    matrix[len1][len2]
}
