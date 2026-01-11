use crate::errors::{MintasError, MintasResult, SourceLocation};
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    String(String),
    Boolean(bool),
    Maybe,
    Empty,
    Plus,
    PlusPlus,
    Minus,
    MinusMinus,
    Multiply,
    Exponent,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    StrictEqual,
    StrictNotEqual,
    And,
    Or,
    Not,
    Assign,
    PlusAssign,
    MinusAssign,
    MultiplyAssign,
    DivideAssign,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Identifier(String),
    Say,
    Ask,
    Let,
    So,
    Const,
    Consta,
    If,
    When,
    Else,
    #[allow(dead_code)]
    ElseIf,
    Elif,
    Otherwise,
    End,
    Colon,
    While,
    For,
    From,
    To,
    In,
    Exit,
    Proceed,
    Dot,
    Question,
    Dollar,
    At,
    Func,
    Lamda,
    Def,
    Make,
    Return,
    Class,
    Public,
    Private,
    New,
    This,
    Try,
    Catch,
    Throw,
    Extends,
    Super,
    Include,
    As,
    Cond,
    Follow,
    Task,
    Switch,
    Case,
    Default,
    Either,
    Goto,
    Times,
    DoubleColon,
    Spr,
    Getback,
    Bring,
    Arrow,  
    #[allow(dead_code)]
    LeftSuperSet,
    #[allow(dead_code)]
    RightSuperSet,
    EOF,
}
#[derive(Debug, Clone)]
pub struct TokenWithLocation {
    pub token: Token,
    pub location: SourceLocation,
}
impl TokenWithLocation {
    pub fn new(token: Token, line: usize, column: usize) -> Self {
        Self {
            token,
            location: SourceLocation::new(line, column),
        }
    }
}
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}
impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }
    #[allow(dead_code)]
    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }
    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.position += 1;
    }
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    fn skip_comments(&mut self) {
        if self.current_char() == Some('#') {
            self.advance();
            if self.current_char() == Some('#') {
                self.advance();
                while let Some(ch) = self.current_char() {
                    if ch == '#' {
                        self.advance();
                        if self.current_char() == Some('#') {
                            self.advance();
                            break;
                        }
                    } else {
                        self.advance();
                    }
                }
            } else {
                while let Some(ch) = self.current_char() {
                    if ch == '\n' || ch == '\r' {
                        break;
                    }
                    self.advance();
                }
            }
        }
    }
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            let start_pos = self.position;
            self.skip_whitespace();
            self.skip_comments();
            if self.position == start_pos {
                break;
            }
        }
    }
    fn read_number(&mut self) -> f64 {
        let mut num_str = String::new();
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() || ch == '.' {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        num_str.parse().unwrap_or(0.0)
    }
    fn read_identifier(&mut self) -> String {
        let mut ident = String::new();
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }
    fn read_string(&mut self) -> MintasResult<String> {
        let start_line = self.line;
        let start_column = self.column;
        self.advance();
        let mut s = String::new();
        loop {
            match self.current_char() {
                Some('"') => {
                    self.advance();
                    return Ok(s);
                }
                Some('/') => {
                    self.advance();
                    match self.current_char() {
                        Some('n') => {
                            s.push('\n');
                            self.advance();
                        }
                        Some('s') => {
                            self.advance();
                            if self.current_char() == Some('.') {
                                self.advance();
                                let mut num_str = String::new();
                                while let Some(ch) = self.current_char() {
                                    if ch.is_ascii_digit() {
                                        num_str.push(ch);
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                }
                                if num_str.is_empty() {
                                    return Err(MintasError::InvalidEscapeSequence {
                                        sequence: "/s.".to_string(),
                                        location: SourceLocation::new(self.line, self.column),
                                    });
                                }
                                let count: usize = num_str.parse().map_err(|_| {
                                    MintasError::InvalidEscapeSequence {
                                        sequence: format!("/s.{}", num_str),
                                        location: SourceLocation::new(self.line, self.column),
                                    }
                                })?;
                                if count > 64 {
                                    return Err(MintasError::InvalidEscapeSequence {
                                        sequence: format!("/s.{} (max is 64)", count),
                                        location: SourceLocation::new(self.line, self.column),
                                    });
                                }
                                for _ in 0..count {
                                    s.push(' ');
                                }
                            } else {
                                s.push('/');
                                s.push('s');
                            }
                        }
                        Some('\\') => {
                            self.advance();
                            match self.current_char() {
                                Some('n') => {
                                    s.push('\n');
                                    self.advance();
                                }
                                Some('t') => {
                                    s.push('\t');
                                    self.advance();
                                }
                                Some('"') => {
                                    s.push('"');
                                    self.advance();
                                }
                                Some('\\') => {
                                    s.push('\\');
                                    self.advance();
                                }
                                Some(ch) => {
                                    s.push(ch);
                                    self.advance();
                                }
                                None => {
                                    return Err(MintasError::UnterminatedString {
                                        location: SourceLocation::new(start_line, start_column),
                                    });
                                }
                            }
                        }
                        Some(ch) => {
                            if ch == '"' {
                                s.push('/');
                            } else {
                                s.push('/');
                                s.push(ch);
                                self.advance();
                            }
                        }
                        None => {
                            return Err(MintasError::UnterminatedString {
                                location: SourceLocation::new(start_line, start_column),
                            });
                        }
                    }
                }
                Some(ch) => {
                    if ch == '\\' {
                        self.advance();
                        match self.current_char() {
                            Some('n') => {
                                s.push('\n');
                                self.advance();
                            }
                            Some('t') => {
                                s.push('\t');
                                self.advance();
                            }
                            Some('r') => {
                                s.push('\r');
                                self.advance();
                            }
                            Some('"') => {
                                s.push('"');
                                self.advance();
                            }
                            Some('\\') => {
                                s.push('\\');
                                self.advance();
                            }
                            Some(c) => {
                                s.push('\\');
                                s.push(c);
                                self.advance();
                            }
                            None => {
                                return Err(MintasError::UnterminatedString {
                                    location: SourceLocation::new(start_line, start_column),
                                });
                            }
                        }
                    } else {
                        s.push(ch);
                        self.advance();
                    }
                }
                None => {
                    return Err(MintasError::UnterminatedString {
                        location: SourceLocation::new(start_line, start_column),
                    });
                }
            }
        }
    }
    #[allow(dead_code)]
    pub fn current_location(&self) -> SourceLocation {
        SourceLocation::new(self.line, self.column)
    }
    pub fn next_token(&mut self) -> MintasResult<TokenWithLocation> {
        self.skip_whitespace_and_comments();
        let start_line = self.line;
        let start_column = self.column;
        let token = match self.current_char() {
            Some('"') => {
                let s = self.read_string()?;
                Token::String(s)
            }
            Some('+') => {
                self.advance();
                match self.current_char() {
                    Some('+') => {
                        self.advance();
                        Token::PlusPlus
                    }
                    Some('=') => {
                        self.advance();
                        Token::PlusAssign
                    }
                    _ => Token::Plus,
                }
            }
            Some('-') => {
                self.advance();
                match self.current_char() {
                    Some('-') => {
                        self.advance();
                        Token::MinusMinus
                    }
                    Some('=') => {
                        self.advance();
                        Token::MinusAssign
                    }
                    _ => Token::Minus,
                }
            }
            Some('*') => {
                self.advance();
                match self.current_char() {
                    Some('*') => {
                        self.advance();
                        Token::Exponent
                    }
                    Some('=') => {
                        self.advance();
                        Token::MultiplyAssign
                    }
                    _ => Token::Multiply,
                }
            }
            Some('/') => {
                self.advance();
                match self.current_char() {
                    Some('=') => {
                        self.advance();
                        Token::DivideAssign
                    }
                    _ => Token::Divide,
                }
            }
            Some('%') => {
                self.advance();
                Token::Modulo
            }
            Some('=') => {
                self.advance();
                match self.current_char() {
                    Some('=') => {
                        self.advance();
                        if self.current_char() == Some('=') {
                            self.advance();
                            Token::StrictEqual
                        } else if self.current_char() == Some('>') {
                            self.advance();
                            Token::Arrow  
                        } else {
                            Token::Equal
                        }
                    }
                    _ => Token::Assign,
                }
            }
            Some('!') => {
                self.advance();
                match self.current_char() {
                    Some('=') => {
                        self.advance();
                        if self.current_char() == Some('=') {
                            self.advance();
                            Token::StrictNotEqual
                        } else {
                            Token::NotEqual
                        }
                    }
                    _ => Token::Not,
                }
            }
            Some('>') => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            Some('<') => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }
            Some('&') => {
                self.advance();
                if self.current_char() == Some('&') {
                    self.advance();
                    Token::And
                } else {
                    return Err(MintasError::InvalidCharacter {
                        character: '&',
                        location: SourceLocation::new(start_line, start_column),
                    });
                }
            }
            Some('|') => {
                self.advance();
                if self.current_char() == Some('|') {
                    self.advance();
                    Token::Or
                } else {
                    return Err(MintasError::InvalidCharacter {
                        character: '|',
                        location: SourceLocation::new(start_line, start_column),
                    });
                }
            }
            Some('(') => {
                self.advance();
                Token::LeftParen
            }
            Some(')') => {
                self.advance();
                Token::RightParen
            }
            Some('{') => {
                self.advance();
                Token::LeftBrace
            }
            Some('}') => {
                self.advance();
                Token::RightBrace
            }
            Some('[') => {
                self.advance();
                Token::LeftBracket
            }
            Some(']') => {
                self.advance();
                Token::RightBracket
            }
            Some(',') => {
                self.advance();
                Token::Comma
            }
            Some(':') => {
                self.advance();
                if matches!(self.current_char(), Some(':')) {
                    self.advance();
                    Token::DoubleColon
                } else {
                    Token::Colon
                }
            }
            Some('.') => {
                self.advance();
                Token::Dot
            }
            Some('?') => {
                self.advance();
                Token::Question
            }
            Some('$') => {
                self.advance();
                Token::Dollar
            }
            Some('@') => {
                self.advance();
                Token::At
            }
            Some(ch) if ch.is_ascii_digit() => Token::Number(self.read_number()),
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                match ident.as_str() {
                    "say" => Token::Say,
                    "ask" => Token::Ask,
                    "let" => Token::Let,
                    "so" => Token::So,
                    "const" => Token::Const,
                    "consta" => Token::Consta,
                    "true" => Token::Boolean(true),
                    "false" => Token::Boolean(false),
                    "maybe" => Token::Maybe,
                    "empty" => Token::Empty,
                    "is" => Token::Assign,
                    "and" => Token::And,
                    "or" => Token::Or,
                    "not" => Token::Not,
                    "equal" => Token::Equal,
                    "no" => Token::NotEqual,
                    "great" => Token::Greater,
                    "less" => Token::Less,
                    "gore" => Token::GreaterEqual,  
                    "lore" => Token::LessEqual,     
                    "sure" => Token::StrictEqual,
                    "nah" => Token::StrictNotEqual,
                    "if" => Token::If,
                    "when" => Token::When,
                    "else" => Token::Else,
                    "elif" => Token::Elif,
                    "elf" => Token::Elif,
                    "otherwise" => Token::Otherwise,
                    "end" => Token::End,
                    "while" => Token::While,
                    "for" => Token::For,
                    "from" => Token::From,
                    "to" => Token::To,
                    "in" => Token::In,
                    "exit" => Token::Exit,
                    "break" => Token::Exit,         
                    "proceed" => Token::Proceed,
                    "continue" => Token::Proceed,   
                    "func" => Token::Func,
                    "lamda" => Token::Lamda,
                    "def" => Token::Def,
                    "make" => Token::Make,
                    "return" => Token::Return,
                    "class" => Token::Class,
                    "public" => Token::Public,
                    "private" => Token::Private,
                    "new" => Token::New,
                    "this" => Token::This,
                    "try" => Token::Try,
                    "catch" => Token::Catch,
                    "throw" => Token::Throw,
                    "cond" => Token::Cond,
                    "follow" => Token::Follow,
                    "extends" => Token::Extends,
                    "super" => Token::Super,
                    "include" => Token::Include,
                    "as" => Token::As,
                    "task" => Token::Task,
                    "switch" => Token::Switch,
                    "case" => Token::Case,
                    "default" => Token::Default,
                    "either" => Token::Either,
                    "goto" => Token::Goto,
                    "times" => Token::Times,
                    "spr" => Token::Spr,
                    "getback" => Token::Getback,
                    "bring" => Token::Bring,
                    _ => Token::Identifier(ident),
                }
            }
            None => Token::EOF,
            Some(ch) => {
                return Err(MintasError::InvalidCharacter {
                    character: ch,
                    location: SourceLocation::new(start_line, start_column),
                });
            }
        };
        Ok(TokenWithLocation::new(token, start_line, start_column))
    }
    pub fn tokenize(&mut self) -> MintasResult<Vec<TokenWithLocation>> {
        let mut tokens = Vec::new();
        loop {
            let token_with_loc = self.next_token()?;
            let is_eof = matches!(token_with_loc.token, Token::EOF);
            tokens.push(token_with_loc);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}