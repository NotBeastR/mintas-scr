use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::ClassInheritance;
use crate::lexer::{Token, TokenWithLocation};
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    String(String),
    Boolean(bool),
    Maybe,
    Empty,
    Array(Vec<Expr>),
    Table(Vec<(String, Expr)>),
    SuperSet(Box<Expr>),
    Variable(String),
    BinaryOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Assign {
        name: String,
        value: Box<Expr>,
        is_const: bool,
    },
    MultiAssign {
        names: Vec<String>,
        values: Vec<Expr>,
        is_const: bool,
    },
    CompoundAssign {
        name: String,
        op: BinaryOp,
        value: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    IfExpr {
        condition: Box<Expr>,
        then_branch: Vec<Expr>,
        else_if_branches: Vec<(Expr, Vec<Expr>)>,
        else_branch: Option<Vec<Expr>>,
    },
    WhileLoop {
        condition: Box<Expr>,
        body: Vec<Expr>,
    },
    ForLoop {
        var: String,
        start: Box<Expr>,
        end: Box<Expr>,
        body: Vec<Expr>,
    },
    ForInLoop {
        var: String,
        iterable: Box<Expr>,
        body: Vec<Expr>,
    },
    Exit,
    Proceed,
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Expr>,
        is_lambda: bool,
    },
    Return {
        value: Option<Box<Expr>>,
    },
    Class {
        name: String,
        members: Vec<ClassMember>,
        inheritance: ClassInheritance,
    },
    New {
        class_name: String,
        args: Vec<Expr>,
    },
    This,
    Property {
        object: Box<Expr>,
        property: String,
    },
    PropertyAssign {
        object: Box<Expr>,
        property: String,
        value: Box<Expr>,
    },
    TryCatch {
        try_block: Vec<Expr>,
        catch_block: Vec<Expr>,
        error_var: Option<String>,
    },
    Cond {
        condition: Box<Expr>,
    },
    Follow {
        condition: Box<Expr>,
        negate: bool,
    },
    Include {
        module_name: String,
        alias: Option<String>,
    },
    Task {
        name: String,
        params: Vec<String>,
        body: Vec<Expr>,
    },
    Switch {
        expression: Box<Expr>,
        cases: Vec<(Expr, Vec<Expr>)>,
        default_case: Option<Vec<Expr>>,
    },
    SmartCondition {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    SmartLoop {
        var: String,
        count: Box<Expr>,
        body: Vec<Expr>,
    },
    DewRoute {
        server: Box<Expr>,
        method: String,  
        path: String,
        body: Vec<Expr>,
    },
    DewServe {
        server: Box<Expr>,
        port: Box<Expr>,
        host: Option<Box<Expr>>,
    },
    DewReturn {
        response_type: String,  
        body: Box<Expr>,
        status: Option<Box<Expr>>,
        data: Option<Box<Expr>>,  
    },
    Getback,  
    DewBefore {
        server: Box<Expr>,
        body: Vec<Expr>,
    },
    DewAfter {
        server: Box<Expr>,
        body: Vec<Expr>,
    },
    DewUse {
        server: Box<Expr>,
        middleware: String,
    },
    DewCatch {
        server: Box<Expr>,
        status_code: u16,
        body: Vec<Expr>,
    },
    DewGroup {
        server: Box<Expr>,
        prefix: String,
        body: Vec<Expr>,
    },
    DewStatic {
        server: Box<Expr>,
        url_path: String,
        dir_path: String,
    },
    DewRouteValidated {
        server: Box<Expr>,
        method: String,
        path: String,
        validation_rules: Box<Expr>,
        body: Vec<Expr>,
    },
    DewConfig {
        server: Box<Expr>,
        config_path: String,
    },
    DewDatabase {
        server: Box<Expr>,
        connection_string: String,
    },
    DewSession {
        server: Box<Expr>,
        config: Option<Box<Expr>>,
    },
    DewRateLimit {
        server: Box<Expr>,
        requests: u32,
        window_seconds: u32,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum ClassMember {
    Property {
        name: String,
        #[allow(dead_code)]
        is_public: bool,
        initial_value: Option<Expr>,
    },
    Method {
        name: String,
        #[allow(dead_code)]
        is_public: bool,
        params: Vec<String>,
        body: Vec<Expr>,
    },
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
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
}
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    Not,
    Increment,
    Decrement,
}
#[allow(dead_code)]
fn get_precedence(token: &Token) -> u8 {
    match token {
        Token::Assign | Token::PlusAssign | Token::MinusAssign | 
        Token::MultiplyAssign | Token::DivideAssign => 1,
        Token::And | Token::Or => 2,
        Token::Greater | Token::Less => 3,
        Token::Equal | Token::NotEqual | Token::GreaterEqual | Token::LessEqual => 4,
        Token::StrictEqual | Token::StrictNotEqual | Token::Modulo => 5,
        Token::Plus | Token::Minus => 5,
        Token::Multiply | Token::Divide | Token::Exponent => 6,
        Token::Not => 7,
        _ => 0,
    }
}
#[allow(dead_code)]
fn is_right_associative(token: &Token) -> bool {
    matches!(token, Token::Exponent | Token::Assign | 
        Token::PlusAssign | Token::MinusAssign | 
        Token::MultiplyAssign | Token::DivideAssign)
}
pub struct Parser {
    tokens: Vec<TokenWithLocation>,
    position: usize,
}
impl Parser {
    pub fn new(tokens: Vec<TokenWithLocation>) -> Self {
        Self { tokens, position: 0 }
    }
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position).map(|t| &t.token)
    }
    fn current_location(&self) -> SourceLocation {
        self.tokens
            .get(self.position)
            .map(|t| t.location.clone())
            .unwrap_or_default()
    }
    #[allow(dead_code)]
    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1).map(|t| &t.token)
    }
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }
    fn expect(&mut self, expected: &Token) -> MintasResult<()> {
        let loc = self.current_location();
        if let Some(token) = self.current_token() {
            if std::mem::discriminant(token) == std::mem::discriminant(expected) {
                self.advance();
                Ok(())
            } else {
                Err(MintasError::UnexpectedToken {
                    expected: format!("{:?}", expected),
                    found: format!("{:?}", token),
                    location: loc,
                })
            }
        } else {
            Err(MintasError::UnexpectedEndOfInput { location: loc })
        }
    }
    pub fn parse(&mut self) -> MintasResult<Vec<Expr>> {
        let mut statements = Vec::new();
        loop {
            if matches!(self.current_token(), Some(Token::EOF) | None) {
                break;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }
    fn parse_statement(&mut self) -> MintasResult<Expr> {
        match self.current_token() {
            Some(Token::At) => self.parse_dew_decorator(),
            Some(Token::Bring) => self.parse_bring(),
            Some(Token::If) | Some(Token::When) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            Some(Token::For) => self.parse_for(),
            Some(Token::Task) => self.parse_task(),
            Some(Token::Switch) => self.parse_switch(),
            Some(Token::Either) => self.parse_smart_condition(),
            Some(Token::Goto) => self.parse_smart_loop(),
            Some(Token::Cond) => {
                self.advance();
                self.expect(&Token::LeftParen)?;
                let condition = self.parse_logical_or()?;
                self.expect(&Token::RightParen)?;
                Ok(Expr::Cond { condition: Box::new(condition) })
            }
            Some(Token::Follow) => {
                self.advance();
                let negate = if matches!(self.current_token(), Some(Token::Not)) {
                    self.advance();
                    true
                } else {
                    false
                };
                self.expect(&Token::LeftParen)?;
                let condition = self.parse_logical_or()?;
                self.expect(&Token::RightParen)?;
                Ok(Expr::Follow { condition: Box::new(condition), negate })
            }
            Some(Token::Include) => {
                self.advance();
                let module_name = match self.current_token() {
                    Some(Token::Identifier(name)) => {
                        let name = name.clone();
                        self.advance();
                        name
                    }
                    _ => return Err(MintasError::ParseError {
                        message: "Expected module name after 'include'".to_string(),
                        location: self.current_location(),
                    }),
                };
                let alias = if matches!(self.current_token(), Some(Token::As)) {
                    self.advance();
                    match self.current_token() {
                        Some(Token::Identifier(alias_name)) => {
                            let alias = alias_name.clone();
                            self.advance();
                            Some(alias)
                        }
                        _ => return Err(MintasError::ParseError {
                            message: "Expected alias name after 'as'".to_string(),
                            location: self.current_location(),
                        }),
                    }
                } else {
                    None
                };
                Ok(Expr::Include { module_name, alias })
            }
            Some(Token::Try) => self.parse_try_catch(),
            Some(Token::Def) | Some(Token::Make) => {
                let keyword = if matches!(self.current_token(), Some(Token::Def)) { "def" } else { "make" };
                self.advance();
                if matches!(self.current_token(), Some(Token::Class)) {
                    self.parse_class()
                } else if matches!(self.current_token(), Some(Token::Func)) {
                    self.parse_function(false)
                } else {
                    return Err(MintasError::ParseError {
                        message: format!("'{}' must be followed by 'func' or 'class'", keyword),
                        location: self.current_location(),
                    });
                }
            }
            Some(Token::Func) => self.parse_function(false),
            Some(Token::Lamda) => self.parse_function(true),
            Some(Token::Exit) => {
                self.advance();
                Ok(Expr::Exit)
            }
            Some(Token::Proceed) => {
                self.advance();
                Ok(Expr::Proceed)
            }
            _ => self.parse_expression(),
        }
    }
    fn parse_expression(&mut self) -> MintasResult<Expr> {
        self.parse_assignment()
    }
    fn parse_assignment(&mut self) -> MintasResult<Expr> {
        let loc = self.current_location();
        let has_let = matches!(self.current_token(), Some(Token::Let));
        let has_so = matches!(self.current_token(), Some(Token::So));
        let has_const = matches!(self.current_token(), Some(Token::Const));
        let has_consta = matches!(self.current_token(), Some(Token::Consta));
        let is_const = has_const || has_consta;
        if has_let || has_so || has_const || has_consta {
            self.advance();
        }
        let expr = self.parse_logical_or()?;
        if let Expr::Property { object, property } = expr {
            if matches!(self.current_token(), Some(Token::Assign)) {
                self.advance(); 
                let value = self.parse_logical_or()?;
                return Ok(Expr::PropertyAssign {
                    object,
                    property,
                    value: Box::new(value),
                });
            } else {
                return Ok(Expr::Property { object, property });
            }
        }
        if let Expr::Variable(name) = expr {
            if matches!(self.current_token(), Some(Token::Assign)) {
                self.advance(); 
                let value = self.parse_logical_or()?;
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                    is_const,
                });
            } else if matches!(self.current_token(), Some(Token::PlusAssign)) {
                if is_const {
                    return Err(MintasError::ConstantReassignment {
                        name,
                        location: loc,
                    });
                }
                self.advance(); 
                let value = self.parse_logical_or()?;
                return Ok(Expr::CompoundAssign {
                    name,
                    op: BinaryOp::Add,
                    value: Box::new(value),
                });
            } else if matches!(self.current_token(), Some(Token::MinusAssign)) {
                if is_const {
                    return Err(MintasError::ConstantReassignment {
                        name,
                        location: loc,
                    });
                }
                self.advance(); 
                let value = self.parse_logical_or()?;
                return Ok(Expr::CompoundAssign {
                    name,
                    op: BinaryOp::Subtract,
                    value: Box::new(value),
                });
            } else if matches!(self.current_token(), Some(Token::MultiplyAssign)) {
                if is_const {
                    return Err(MintasError::ConstantReassignment {
                        name,
                        location: loc,
                    });
                }
                self.advance(); 
                let value = self.parse_logical_or()?;
                return Ok(Expr::CompoundAssign {
                    name,
                    op: BinaryOp::Multiply,
                    value: Box::new(value),
                });
            } else if matches!(self.current_token(), Some(Token::DivideAssign)) {
                if is_const {
                    return Err(MintasError::ConstantReassignment {
                        name,
                        location: loc,
                    });
                }
                self.advance(); 
                let value = self.parse_logical_or()?;
                return Ok(Expr::CompoundAssign {
                    name,
                    op: BinaryOp::Divide,
                    value: Box::new(value),
                });
            } else {
                if has_let {
                    return Err(MintasError::MissingAssignment {
                        keyword: "let".to_string(),
                        location: loc,
                    });
                }
                if has_so {
                    return Err(MintasError::MissingAssignment {
                        keyword: "so".to_string(),
                        location: loc,
                    });
                }
                if has_const {
                    return Err(MintasError::MissingAssignment {
                        keyword: "const".to_string(),
                        location: loc,
                    });
                }
                if has_consta {
                    return Err(MintasError::MissingAssignment {
                        keyword: "consta".to_string(),
                        location: loc,
                    });
                }
                return Ok(Expr::Variable(name));
            }
        }
        if has_let || has_so || has_const || has_consta {
            return Err(MintasError::ParseError {
                message: format!("'{}' must be followed by a variable name", 
                    if has_let { "let" } else if has_so { "so" } else if has_const { "const" } else { "consta" }),
                location: loc,
            });
        }
        Ok(expr)
    }
    fn validate_variable_name(&self, name: &str) -> MintasResult<()> {
        let loc = self.current_location();
        if name.is_empty() {
            return Err(MintasError::InvalidVariableName {
                name: name.to_string(),
                reason: "Variable name cannot be empty".to_string(),
                location: loc,
            });
        }
        if name.len() > 32 {
            return Err(MintasError::InvalidVariableName {
                name: name.to_string(),
                reason: "Variable name cannot exceed 32 characters".to_string(),
                location: loc,
            });
        }
        let first_char = name.chars().next().unwrap();
        if first_char.is_ascii_digit() {
            return Err(MintasError::InvalidVariableName {
                name: name.to_string(),
                reason: "Variable name cannot start with a number".to_string(),
                location: loc,
            });
        }
        for ch in name.chars() {
            if !(ch.is_ascii_alphanumeric() || ch == '_') {
                return Err(MintasError::InvalidVariableName {
                    name: name.to_string(),
                    reason: format!("Invalid character '{}' in variable name", ch),
                    location: loc,
                });
            }
        }
        match name {
            "say" | "ask" | "let" | "so" | "is" | "const" | "consta" |
            "true" | "false" | "maybe" | "empty" |
            "and" | "or" | "not" | "equal" | "no" | "great" | "less" | "sure" | "nah" |
            "if" | "when" | "else" | "elif" | "otherwise" | "end" |
            "while" | "for" | "from" | "to" | "in" | "exit" | "proceed" |
            "func" | "def" | "make" | "lamda" | "return" |
            "class" | "public" | "private" | "new" | "this" |
            "try" | "catch" | "throw" | "extends" | "super" => {
                Err(MintasError::InvalidVariableName {
                    name: name.to_string(),
                    reason: format!("'{}' is a reserved keyword", name),
                    location: loc,
                })
            }
            _ => Ok(()),
        }
    }
    fn parse_logical_or(&mut self) -> MintasResult<Expr> {
        let mut expr = self.parse_logical_and()?;
        while let Some(Token::Or) = self.current_token() {
            self.advance();
            let right = self.parse_logical_and()?;
            expr = Expr::BinaryOp {
                op: BinaryOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn parse_logical_and(&mut self) -> MintasResult<Expr> {
        let mut expr = self.parse_comparison()?;
        while let Some(Token::And) = self.current_token() {
            self.advance();
            let right = self.parse_comparison()?;
            expr = Expr::BinaryOp {
                op: BinaryOp::And,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn parse_comparison(&mut self) -> MintasResult<Expr> {
        let mut expr = self.parse_additive()?;
        loop {
            let op = match self.current_token() {
                Some(Token::Equal) => BinaryOp::Equal,
                Some(Token::NotEqual) => BinaryOp::NotEqual,
                Some(Token::Greater) => BinaryOp::Greater,
                Some(Token::Less) => BinaryOp::Less,
                Some(Token::GreaterEqual) => BinaryOp::GreaterEqual,
                Some(Token::LessEqual) => BinaryOp::LessEqual,
                Some(Token::StrictEqual) => BinaryOp::StrictEqual,
                Some(Token::StrictNotEqual) => BinaryOp::StrictNotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            expr = Expr::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn parse_additive(&mut self) -> MintasResult<Expr> {
        let mut expr = self.parse_multiplicative()?;
        loop {
            let op = match self.current_token() {
                Some(Token::Plus) => BinaryOp::Add,
                Some(Token::Minus) => BinaryOp::Subtract,
                Some(Token::Modulo) => BinaryOp::Modulo,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            expr = Expr::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn parse_multiplicative(&mut self) -> MintasResult<Expr> {
        let mut expr = self.parse_exponent()?;
        loop {
            let op = match self.current_token() {
                Some(Token::Multiply) => BinaryOp::Multiply,
                Some(Token::Divide) => BinaryOp::Divide,
                _ => break,
            };
            self.advance();
            let right = self.parse_exponent()?;
            expr = Expr::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn parse_exponent(&mut self) -> MintasResult<Expr> {
        let mut expr = self.parse_unary()?;
        while let Some(Token::Exponent) = self.current_token() {
            self.advance();
            let right = self.parse_exponent()?;
            expr = Expr::BinaryOp {
                op: BinaryOp::Exponent,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    fn parse_unary(&mut self) -> MintasResult<Expr> {
        match self.current_token() {
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Negate,
                    expr: Box::new(expr),
                })
            }
            Some(Token::Not) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_postfix(),
        }
    }
    fn parse_postfix(&mut self) -> MintasResult<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.current_token() {
                Some(Token::Dot) => {
                    self.advance();
                    let property = match self.current_token() {
                        Some(Token::Identifier(name)) => name.clone(),
                        _ => return Err(MintasError::ParseError {
                            message: "Expected property or method name after '.'".to_string(),
                            location: self.current_location(),
                        }),
                    };
                    self.advance();
                    if matches!(self.current_token(), Some(Token::LeftParen)) {
                        self.advance();
                        let mut args = Vec::new();
                        if !matches!(self.current_token(), Some(Token::RightParen)) {
                            loop {
                                args.push(self.parse_logical_or()?);
                                if matches!(self.current_token(), Some(Token::Comma)) {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                        }
                        self.expect(&Token::RightParen)?;
                        expr = Expr::MethodCall {
                            object: Box::new(expr),
                            method: property,
                            args,
                        };
                    } else {
                        expr = Expr::Property {
                            object: Box::new(expr),
                            property,
                        };
                    }
                }
                Some(Token::LeftBracket) => {
                    self.advance();
                    let index = self.parse_logical_or()?;
                    self.expect(&Token::RightBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Some(Token::Question) => {
                    self.advance();
                    let then_expr = self.parse_logical_or()?;
                    self.expect(&Token::Colon)?;
                    let else_expr = self.parse_logical_or()?;
                    expr = Expr::Ternary {
                        condition: Box::new(expr),
                        then_expr: Box::new(then_expr),
                        else_expr: Box::new(else_expr),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }
    fn parse_primary(&mut self) -> MintasResult<Expr> {
        let loc = self.current_location();
        match self.current_token() {
            Some(Token::Getback) => {
                self.advance();
                Ok(Expr::Getback)
            }
            Some(Token::Number(n)) => {
                let value = *n;
                self.advance();
                Ok(Expr::Number(value))
            }
            Some(Token::String(s)) => {
                let value = s.clone();
                self.advance();
                Ok(Expr::String(value))
            }
            Some(Token::Boolean(b)) => {
                let value = *b;
                self.advance();
                Ok(Expr::Boolean(value))
            }
            Some(Token::Maybe) => {
                self.advance();
                Ok(Expr::Maybe)
            }
            Some(Token::Empty) => {
                self.advance();
                Ok(Expr::Empty)
            }
            Some(Token::LeftBracket) => {
                self.advance();
                let mut elements = Vec::new();
                if !matches!(self.current_token(), Some(Token::RightBracket)) {
                    loop {
                        elements.push(self.parse_logical_or()?);
                        if matches!(self.current_token(), Some(Token::Comma)) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&Token::RightBracket)?;
                Ok(Expr::Array(elements))
            }
            Some(Token::LeftBrace) => {
                self.advance();
                let mut pairs = Vec::new();
                if !matches!(self.current_token(), Some(Token::RightBrace)) {
                    loop {
                        let key_str = match self.current_token() {
                            Some(Token::String(key)) => {
                                let k = key.clone();
                                self.advance();
                                k
                            }
                            Some(Token::Identifier(key)) => {
                                let k = key.clone();
                                self.advance();
                                k
                            }
                            _ => {
                                return Err(MintasError::ParseError {
                                    message: "Table keys must be identifiers or strings".to_string(),
                                    location: loc,
                                });
                            }
                        };
                        self.expect(&Token::Assign)?;
                        let value = self.parse_logical_or()?;
                        pairs.push((key_str, value));
                        if matches!(self.current_token(), Some(Token::Comma)) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&Token::RightBrace)?;
                Ok(Expr::Table(pairs))
            }
            Some(Token::Say) => {
                self.advance();
                self.expect(&Token::LeftParen)?;
                let arg = self.parse_logical_or()?;
                self.expect(&Token::RightParen)?;
                Ok(Expr::Call {
                    name: "say".to_string(),
                    args: vec![arg],
                })
            }
            Some(Token::Ask) => {
                self.advance();
                self.expect(&Token::LeftParen)?;
                let arg = self.parse_logical_or()?;
                self.expect(&Token::RightParen)?;
                Ok(Expr::Call {
                    name: "ask".to_string(),
                    args: vec![arg],
                })
            }
            Some(Token::This) => {
                self.advance();
                Ok(Expr::This)
            }
            Some(Token::New) => {
                self.advance();
                let class_name = match self.current_token() {
                    Some(Token::Identifier(name)) => {
                        let name = name.clone();
                        self.advance();
                        name
                    }
                    _ => return Err(MintasError::ParseError {
                        message: "Expected class name after 'new'".to_string(),
                        location: self.current_location(),
                    }),
                };
                self.expect(&Token::LeftParen)?;
                let mut args = Vec::new();
                if !matches!(self.current_token(), Some(Token::RightParen)) {
                    loop {
                        args.push(self.parse_logical_or()?);
                        if matches!(self.current_token(), Some(Token::Comma)) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&Token::RightParen)?;
                Ok(Expr::New { class_name, args })
            }
            Some(Token::Spr) => {
                self.advance();
                self.expect(&Token::LeftBrace)?;
                let inner_expr = self.parse_logical_or()?;
                self.expect(&Token::RightBrace)?;
                Ok(Expr::SuperSet(Box::new(inner_expr)))
            }
            Some(Token::Identifier(name)) => {
                let var_name = name.clone();
                self.advance();
                if let Some(Token::LeftParen) = self.current_token() {
                    self.advance();
                    let mut args = Vec::new();
                    if !matches!(self.current_token(), Some(Token::RightParen)) {
                        loop {
                            args.push(self.parse_logical_or()?);
                            if matches!(self.current_token(), Some(Token::Comma)) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(&Token::RightParen)?;
                    Ok(Expr::Call {
                        name: var_name,
                        args,
                    })
                } else if let Some(Token::PlusPlus) = self.current_token() {
                    self.advance();
                    self.validate_variable_name(&var_name)?;
                    Ok(Expr::UnaryOp {
                        op: UnaryOp::Increment,
                        expr: Box::new(Expr::Variable(var_name)),
                    })
                } else if let Some(Token::MinusMinus) = self.current_token() {
                    self.advance();
                    self.validate_variable_name(&var_name)?;
                    Ok(Expr::UnaryOp {
                        op: UnaryOp::Decrement,
                        expr: Box::new(Expr::Variable(var_name)),
                    })
                } else {
                    self.validate_variable_name(&var_name)?;
                    Ok(Expr::Variable(var_name))
                }
            }
            Some(Token::LeftParen) => {
                self.advance();
                let expr = self.parse_logical_or()?;
                self.expect(&Token::RightParen)?;
                Ok(expr)
            }
            Some(Token::Cond) => {
                self.advance();
                self.expect(&Token::LeftParen)?;
                let condition = self.parse_logical_or()?;
                self.expect(&Token::RightParen)?;
                Ok(Expr::Cond { condition: Box::new(condition) })
            }
            Some(Token::Follow) => {
                self.advance();
                let negate = if matches!(self.current_token(), Some(Token::Not)) {
                    self.advance();
                    true
                } else {
                    false
                };
                self.expect(&Token::LeftParen)?;
                let condition = self.parse_logical_or()?;
                self.expect(&Token::RightParen)?;
                Ok(Expr::Follow { condition: Box::new(condition), negate })
            }
            Some(token) => Err(MintasError::UnexpectedToken {
                expected: "expression".to_string(),
                found: format!("{:?}", token),
                location: loc,
            }),
            None => Err(MintasError::UnexpectedEndOfInput { location: loc }),
        }
    }
    fn parse_if(&mut self) -> MintasResult<Expr> {
        let loc = self.current_location();
        self.advance();
        self.expect(&Token::LeftParen)?;
        let condition = self.parse_logical_or()?;
        self.expect(&Token::RightParen)?;
        self.expect(&Token::Colon)?;
        let then_branch = self.parse_block()?;
        let mut else_if_branches = Vec::new();
        let mut else_branch = None;
        loop {
            match self.current_token() {
                Some(Token::Else) => {
                    self.advance();
                    if matches!(self.current_token(), Some(Token::If) | Some(Token::When)) {
                        self.advance();
                        self.expect(&Token::LeftParen)?;
                        let elif_condition = self.parse_logical_or()?;
                        self.expect(&Token::RightParen)?;
                        self.expect(&Token::Colon)?;
                        let elif_body = self.parse_block()?;
                        else_if_branches.push((elif_condition, elif_body));
                    } else {
                        self.expect(&Token::Colon)?;
                        else_branch = Some(self.parse_block()?);
                        break;
                    }
                }
                Some(Token::Elif) => {
                    self.advance();
                    self.expect(&Token::LeftParen)?;
                    let elif_condition = self.parse_logical_or()?;
                    self.expect(&Token::RightParen)?;
                    self.expect(&Token::Colon)?;
                    let elif_body = self.parse_block()?;
                    else_if_branches.push((elif_condition, elif_body));
                }
                Some(Token::Otherwise) => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    else_branch = Some(self.parse_block()?);
                    break;
                }
                Some(Token::End) => {
                    break;
                }
                _ => {
                    return Err(MintasError::ParseError {
                        message: "Expected 'else', 'elif', 'otherwise', or 'end'".to_string(),
                        location: loc,
                    });
                }
            }
        }
        self.expect(&Token::End)?;
        Ok(Expr::IfExpr {
            condition: Box::new(condition),
            then_branch,
            else_if_branches,
            else_branch,
        })
    }
    fn parse_block(&mut self) -> MintasResult<Vec<Expr>> {
        let mut statements = Vec::new();
        loop {
            while matches!(self.current_token(), Some(Token::Dot)) {
                self.advance();
            }
            match self.current_token() {
                Some(Token::End) | Some(Token::Else) | Some(Token::Elif) | Some(Token::Otherwise) | Some(Token::Catch) | Some(Token::Case) | Some(Token::Default) | Some(Token::EOF) | None => {
                    break;
                }
                Some(Token::Return) => {
                    self.advance();
                    if matches!(self.current_token(), Some(Token::Dot)) {
                        self.advance();
                        let response_type = match self.current_token() {
                            Some(Token::Identifier(name)) => {
                                let name = name.clone();
                                self.advance();
                                name
                            }
                            _ => return Err(MintasError::ParseError {
                                message: "Expected response type after 'return.'".to_string(),
                                location: self.current_location(),
                            }),
                        };
                        self.expect(&Token::LeftParen)?;
                        let body = self.parse_logical_or()?;
                        let mut status = None;
                        let mut data = None;
                        if matches!(self.current_token(), Some(Token::Comma)) {
                            self.advance();
                            if response_type == "inview" {
                                data = Some(Box::new(self.parse_logical_or()?));
                            } else {
                                if let Some(Token::Identifier(name)) = self.current_token() {
                                    if name == "status" {
                                        self.advance();
                                        self.expect(&Token::Assign)?;
                                    }
                                }
                                status = Some(Box::new(self.parse_logical_or()?));
                            }
                        }
                        self.expect(&Token::RightParen)?;
                        statements.push(Expr::DewReturn {
                            response_type,
                            body: Box::new(body),
                            status,
                            data,
                        });
                        break;
                    }
                    let value = if !matches!(self.current_token(), Some(Token::End) | Some(Token::Else) | Some(Token::Elif) | Some(Token::Otherwise) | Some(Token::Catch) | Some(Token::Case) | Some(Token::Default) | Some(Token::EOF) | None | Some(Token::Dot)) {
                        Some(Box::new(self.parse_logical_or()?))
                    } else {
                        None
                    };
                    statements.push(Expr::Return { value });
                    if matches!(self.current_token(), Some(Token::Dot)) {
                        self.advance();
                    }
                    break;
                }
                _ => {
                    statements.push(self.parse_statement()?);
                }
            }
        }
        Ok(statements)
    }
    fn parse_while(&mut self) -> MintasResult<Expr> {
        self.advance();
        self.expect(&Token::LeftParen)?;
        let condition = self.parse_logical_or()?;
        self.expect(&Token::RightParen)?;
        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;
        self.expect(&Token::End)?;
        Ok(Expr::WhileLoop {
            condition: Box::new(condition),
            body,
        })
    }
    fn parse_for(&mut self) -> MintasResult<Expr> {
        let loc = self.current_location();
        self.advance();
        self.expect(&Token::LeftParen)?;
        let var_name = match self.current_token() {
            Some(Token::Identifier(name)) => name.clone(),
            _ => return Err(MintasError::ParseError {
                message: "Expected variable name in for loop".to_string(),
                location: loc,
            }),
        };
        self.advance();
        if matches!(self.current_token(), Some(Token::From)) {
            self.advance();
            let start = self.parse_logical_or()?;
            self.expect(&Token::To)?;
            let end = self.parse_logical_or()?;
            self.expect(&Token::RightParen)?;
            self.expect(&Token::Colon)?;
            let body = self.parse_block()?;
            self.expect(&Token::End)?;
            Ok(Expr::ForLoop {
                var: var_name,
                start: Box::new(start),
                end: Box::new(end),
                body,
            })
        } else if matches!(self.current_token(), Some(Token::In)) {
            self.advance();
            let iterable = self.parse_logical_or()?;
            self.expect(&Token::RightParen)?;
            self.expect(&Token::Colon)?;
            let body = self.parse_block()?;
            self.expect(&Token::End)?;
            Ok(Expr::ForInLoop {
                var: var_name,
                iterable: Box::new(iterable),
                body,
            })
        } else {
            Err(MintasError::ParseError {
                message: "Expected 'from' or 'in' in for loop".to_string(),
                location: loc,
            })
        }
    }
    fn parse_function(&mut self, is_lambda: bool) -> MintasResult<Expr> {
        let loc = self.current_location();
        self.advance();
        let name = match self.current_token() {
            Some(Token::Identifier(n)) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected function name".to_string(),
                location: loc,
            }),
        };
        self.expect(&Token::LeftParen)?;
        let mut params = Vec::new();
        if !matches!(self.current_token(), Some(Token::RightParen)) {
            loop {
                if let Some(Token::Identifier(param)) = self.current_token() {
                    params.push(param.clone());
                    self.advance();
                    if matches!(self.current_token(), Some(Token::Comma)) {
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    return Err(MintasError::ParseError {
                        message: "Expected parameter name".to_string(),
                        location: self.current_location(),
                    });
                }
            }
        }
        self.expect(&Token::RightParen)?;
        self.expect(&Token::Colon)?;
        let body = if is_lambda {
            let expr = self.parse_logical_or()?;
            vec![Expr::Return { value: Some(Box::new(expr)) }]
        } else {
            let body = self.parse_block()?;
            self.expect(&Token::End)?;
            body
        };
        Ok(Expr::Function {
            name,
            params,
            body,
            is_lambda,
        })
    }
    fn parse_class(&mut self) -> MintasResult<Expr> {
        let loc = self.current_location();
        self.expect(&Token::Class)?;
        let name = match self.current_token() {
            Some(Token::Identifier(n)) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected class name".to_string(),
                location: loc,
            }),
        };
        let inheritance = if matches!(self.current_token(), Some(Token::Extends)) {
            self.advance();
            if let Some(Token::Identifier(parent_class)) = self.current_token() {
                let parent = parent_class.clone();
                self.advance();
                ClassInheritance::Extends(parent)
            } else {
                return Err(MintasError::ParseError {
                    message: "Expected parent class name after 'extends'".to_string(),
                    location: self.current_location(),
                });
            }
        } else {
            ClassInheritance::None
        };
        self.expect(&Token::Colon)?;
        let mut members = Vec::new();
        loop {
            match self.current_token() {
                Some(Token::End) | Some(Token::EOF) | None => break,
                Some(Token::Public) | Some(Token::Private) => {
                    let is_public = matches!(self.current_token(), Some(Token::Public));
                    self.advance();
                    if let Some(Token::Identifier(member_name)) = self.current_token() {
                        let member_name = member_name.clone();
                        self.advance();
                        if matches!(self.current_token(), Some(Token::LeftParen)) {
                            self.expect(&Token::LeftParen)?;
                            let mut params = Vec::new();
                            if !matches!(self.current_token(), Some(Token::RightParen)) {
                                loop {
                                    if let Some(Token::Identifier(param)) = self.current_token() {
                                        params.push(param.clone());
                                        self.advance();
                                        if matches!(self.current_token(), Some(Token::Comma)) {
                                            self.advance();
                                        } else {
                                            break;
                                        }
                                    } else {
                                        return Err(MintasError::ParseError {
                                            message: "Expected parameter name".to_string(),
                                            location: self.current_location(),
                                        });
                                    }
                                }
                            }
                            self.expect(&Token::RightParen)?;
                            self.expect(&Token::Colon)?;
                            let body = self.parse_block()?;
                            self.expect(&Token::End)?;
                            members.push(ClassMember::Method {
                                name: member_name,
                                is_public,
                                params,
                                body,
                            });
                        } else if matches!(self.current_token(), Some(Token::Assign)) {
                            self.advance();
                            let initial_value = Some(self.parse_logical_or()?);
                            members.push(ClassMember::Property {
                                name: member_name,
                                is_public,
                                initial_value,
                            });
                        } else {
                            members.push(ClassMember::Property {
                                name: member_name,
                                is_public,
                                initial_value: None,
                            });
                        }
                    } else {
                        return Err(MintasError::ParseError {
                            message: "Expected member name after public/private".to_string(),
                            location: self.current_location(),
                        });
                    }
                }
                _ => return Err(MintasError::ParseError {
                    message: "Expected 'public', 'private', or 'end' in class definition".to_string(),
                    location: self.current_location(),
                }),
            }
        }
        self.expect(&Token::End)?;
        Ok(Expr::Class {
            name,
            members,
            inheritance,
        })
    }
    fn parse_smart_condition(&mut self) -> MintasResult<Expr> {
        self.advance(); 
        self.expect(&Token::LeftParen)?;
        let condition = self.parse_logical_or()?;
        self.expect(&Token::RightParen)?;
        self.expect(&Token::Colon)?;
        let then_branch = self.parse_statement()?;
        self.expect(&Token::DoubleColon)?;
        let else_branch = self.parse_statement()?;
        Ok(Expr::SmartCondition {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }
    fn parse_smart_loop(&mut self) -> MintasResult<Expr> {
        let loc = self.current_location();
        self.advance(); 
        self.expect(&Token::LeftParen)?;
        let var = match self.current_token() {
            Some(Token::Identifier(name)) => name.clone(),
            _ => return Err(MintasError::ParseError {
                message: "Expected variable name in smart loop".to_string(),
                location: loc,
            }),
        };
        self.advance();
        if matches!(self.current_token(), Some(Token::Assign)) {
            self.advance();
        }
        let count = self.parse_logical_or()?;
        self.expect(&Token::Times)?;
        self.expect(&Token::RightParen)?;
        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;
        self.expect(&Token::End)?;
        Ok(Expr::SmartLoop {
            var,
            count: Box::new(count),
            body,
        })
    }
    fn parse_task(&mut self) -> MintasResult<Expr> {
        self.advance(); 
        let name = match self.current_token() {
            Some(Token::Identifier(n)) => {
                let name = n.clone();
                self.advance();
                name
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected task name".to_string(),
                location: self.current_location(),
            }),
        };
        self.expect(&Token::LeftParen)?;
        let mut params = Vec::new();
        if !matches!(self.current_token(), Some(Token::RightParen)) {
             loop {
                 if let Some(Token::Identifier(param)) = self.current_token() {
                     params.push(param.clone());
                     self.advance();
                     if matches!(self.current_token(), Some(Token::Comma)) {
                         self.advance();
                     } else {
                         break;
                     }
                 } else {
                      return Err(MintasError::ParseError {
                          message: "Expected parameter name".to_string(),
                          location: self.current_location(),
                      });
                 }
             }
        }
        self.expect(&Token::RightParen)?;
        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;
        self.expect(&Token::End)?;
        Ok(Expr::Task { name, params, body })
    }
    fn parse_switch(&mut self) -> MintasResult<Expr> {
        self.advance(); 
        self.expect(&Token::LeftParen)?;
        let expression = self.parse_logical_or()?;
        self.expect(&Token::RightParen)?;
        self.expect(&Token::Colon)?;
        let mut cases = Vec::new();
        let mut default_case = None;
        loop {
            match self.current_token() {
                Some(Token::Case) => {
                    self.advance();
                    let val = self.parse_logical_or()?;
                    self.expect(&Token::Colon)?;
                    let mut body = Vec::new();
                    loop {
                        match self.current_token() {
                            Some(Token::Case) | Some(Token::Default) | Some(Token::End) | Some(Token::EOF) | None => break,
                            _ => body.push(self.parse_statement()?),
                        }
                    }
                    cases.push((val, body));
                }
                Some(Token::Default) => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    let mut body = Vec::new();
                    loop {
                         match self.current_token() {
                            Some(Token::Case) | Some(Token::Default) | Some(Token::End) | Some(Token::EOF) | None => break,
                            _ => body.push(self.parse_statement()?),
                        }
                    }
                    default_case = Some(body);
                }
                Some(Token::End) => {
                    self.advance();
                    break;
                }
                _ => return Err(MintasError::ParseError {
                    message: "Expected 'case', 'default', or 'end' in switch block".to_string(),
                    location: self.current_location(),
                }),
            }
        }
        Ok(Expr::Switch {
            expression: Box::new(expression),
            cases,
            default_case,
        })
    }
    fn parse_try_catch(&mut self) -> MintasResult<Expr> {
        self.advance(); 
        self.expect(&Token::Colon)?;
        let try_block = self.parse_block()?;
        self.expect(&Token::Catch)?;
        let error_var = if let Some(Token::Identifier(name)) = self.current_token() {
             let var_name = name.clone();
             self.advance();
             Some(var_name)
        } else {
             None
        };
        self.expect(&Token::Colon)?;
        let catch_block = self.parse_block()?;
        self.expect(&Token::End)?;
        Ok(Expr::TryCatch {
            try_block,
            catch_block,
            error_var,
        })
    }
    fn parse_bring(&mut self) -> MintasResult<Expr> {
        self.advance(); 
        let module_name = match self.current_token() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected module name after 'bring'".to_string(),
                location: self.current_location(),
            }),
        };
        let alias = if matches!(self.current_token(), Some(Token::As)) {
            self.advance();
            match self.current_token() {
                Some(Token::Identifier(alias_name)) => {
                    let alias = alias_name.clone();
                    self.advance();
                    Some(alias)
                }
                _ => return Err(MintasError::ParseError {
                    message: "Expected alias name after 'as'".to_string(),
                    location: self.current_location(),
                }),
            }
        } else {
            None
        };
        Ok(Expr::Include { module_name, alias })
    }
    fn parse_dew_decorator(&mut self) -> MintasResult<Expr> {
        self.advance(); 
        let server_name = match self.current_token() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected server name after '@'".to_string(),
                location: self.current_location(),
            }),
        };
        self.expect(&Token::Dot)?;
        let method = match self.current_token() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            Some(Token::Catch) => {
                self.advance();
                "catch".to_string()
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected HTTP method after '.'".to_string(),
                location: self.current_location(),
            }),
        };
        match method.as_str() {
            "get" | "post" | "put" | "delete" | "patch" => {
                self.parse_dew_route(server_name, method)
            }
            "serve" => {
                self.parse_dew_serve(server_name)
            }
            "static" => {
                self.parse_dew_static(server_name)
            }
            "before" => {
                self.parse_dew_before(server_name)
            }
            "after" => {
                self.parse_dew_after(server_name)
            }
            "use" => {
                self.parse_dew_use(server_name)
            }
            "catch" => {
                self.parse_dew_catch(server_name)
            }
            "group" => {
                self.parse_dew_group(server_name)
            }
            "config" => {
                self.parse_dew_config(server_name)
            }
            "database" | "db" => {
                self.parse_dew_database(server_name)
            }
            "session" => {
                self.parse_dew_session(server_name)
            }
            "rate_limit" => {
                self.parse_dew_rate_limit(server_name)
            }
            _ => Err(MintasError::ParseError {
                message: format!("Unknown Dew method: {}", method),
                location: self.current_location(),
            }),
        }
    }
    fn parse_dew_route(&mut self, server_name: String, method: String) -> MintasResult<Expr> {
        self.expect(&Token::LeftParen)?;
        let path = match self.current_token() {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected path string in route definition".to_string(),
                location: self.current_location(),
            }),
        };
        self.expect(&Token::RightParen)?;
        if matches!(self.current_token(), Some(Token::Arrow)) {
            self.advance(); 
            match self.current_token() {
                Some(Token::Identifier(name)) if name == "validate" => {
                    self.advance();
                }
                _ => return Err(MintasError::ParseError {
                    message: "Expected 'validate' after '==>'".to_string(),
                    location: self.current_location(),
                }),
            }
            self.expect(&Token::LeftParen)?;
            let validation_rules = self.parse_logical_or()?;
            self.expect(&Token::RightParen)?;
            self.expect(&Token::Colon)?;
            let body = self.parse_block()?;
            self.expect(&Token::End)?;
            return Ok(Expr::DewRouteValidated {
                server: Box::new(Expr::Variable(server_name)),
                method,
                path,
                validation_rules: Box::new(validation_rules),
                body,
            });
        }
        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;
        self.expect(&Token::End)?;
        Ok(Expr::DewRoute {
            server: Box::new(Expr::Variable(server_name)),
            method,
            path,
            body,
        })
    }
    fn parse_dew_serve(&mut self, server_name: String) -> MintasResult<Expr> {
        self.expect(&Token::LeftParen)?;
        let mut port = Box::new(Expr::Number(3000.0));
        let mut host: Option<Box<Expr>> = None;
        let mut options: Vec<(String, Box<Expr>)> = Vec::new();
        if !matches!(self.current_token(), Some(Token::RightParen)) {
            loop {
                match self.current_token() {
                    Some(Token::Identifier(name)) if name == "port" => {
                        self.advance();
                        self.expect(&Token::Assign)?;
                        port = Box::new(self.parse_logical_or()?);
                    }
                    Some(Token::Identifier(name)) if name == "ip" => {
                        self.advance();
                        self.expect(&Token::Assign)?;
                        host = Some(Box::new(self.parse_logical_or()?));
                    }
                    Some(Token::Identifier(name)) if name == "timeout" || name == "debug" || 
                        name == "security" || name == "fast_reload" || name == "workers" ||
                        name == "max_connections" || name == "keep_alive" => {
                        let opt_name = name.clone();
                        self.advance();
                        self.expect(&Token::Assign)?;
                        let opt_value = Box::new(self.parse_logical_or()?);
                        options.push((opt_name, opt_value));
                    }
                    Some(Token::Number(_)) => {
                        port = Box::new(self.parse_logical_or()?);
                    }
                    _ => break,
                }
                if matches!(self.current_token(), Some(Token::Comma)) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::RightParen)?;
        Ok(Expr::DewServe {
            server: Box::new(Expr::Variable(server_name)),
            port,
            host,
        })
    }
    fn parse_dew_static(&mut self, server_name: String) -> MintasResult<Expr> {
        self.expect(&Token::LeftParen)?;
        let url_prefix = match self.current_token() {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected URL prefix string".to_string(),
                location: self.current_location(),
            }),
        };
        self.expect(&Token::Comma)?;
        let dir_path = match self.current_token() {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected directory path string".to_string(),
                location: self.current_location(),
            }),
        };
        self.expect(&Token::RightParen)?;
        Ok(Expr::DewStatic {
            server: Box::new(Expr::Variable(server_name)),
            url_path: url_prefix,
            dir_path,
        })
    }
    fn parse_dew_before(&mut self, server_name: String) -> MintasResult<Expr> {
        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;
        self.expect(&Token::End)?;
        Ok(Expr::DewBefore {
            server: Box::new(Expr::Variable(server_name)),
            body,
        })
    }
    fn parse_dew_after(&mut self, server_name: String) -> MintasResult<Expr> {
        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;
        self.expect(&Token::End)?;
        Ok(Expr::DewAfter {
            server: Box::new(Expr::Variable(server_name)),
            body,
        })
    }
    fn parse_dew_use(&mut self, server_name: String) -> MintasResult<Expr> {
        self.expect(&Token::LeftParen)?;
        let middleware = match self.current_token() {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected middleware name string".to_string(),
                location: self.current_location(),
            }),
        };
        self.expect(&Token::RightParen)?;
        Ok(Expr::DewUse {
            server: Box::new(Expr::Variable(server_name)),
            middleware,
        })
    }
    fn parse_dew_catch(&mut self, server_name: String) -> MintasResult<Expr> {
        self.expect(&Token::LeftParen)?;
        let status_code = match self.current_token() {
            Some(Token::Number(n)) => {
                let code = *n as u16;
                self.advance();
                code
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected status code number".to_string(),
                location: self.current_location(),
            }),
        };
        self.expect(&Token::RightParen)?;
        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;
        self.expect(&Token::End)?;
        Ok(Expr::DewCatch {
            server: Box::new(Expr::Variable(server_name)),
            status_code,
            body,
        })
    }
    fn parse_dew_group(&mut self, server_name: String) -> MintasResult<Expr> {
        self.expect(&Token::LeftParen)?;
        let prefix = match self.current_token() {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected group prefix string".to_string(),
                location: self.current_location(),
            }),
        };
        self.expect(&Token::RightParen)?;
        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;
        self.expect(&Token::End)?;
        Ok(Expr::DewGroup {
            server: Box::new(Expr::Variable(server_name)),
            prefix,
            body,
        })
    }
    fn parse_dew_config(&mut self, server_name: String) -> MintasResult<Expr> {
        self.expect(&Token::LeftParen)?;
        let config_path = match self.current_token() {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected config file path string".to_string(),
                location: self.current_location(),
            }),
        };
        self.expect(&Token::RightParen)?;
        Ok(Expr::DewConfig {
            server: Box::new(Expr::Variable(server_name)),
            config_path,
        })
    }
    fn parse_dew_database(&mut self, server_name: String) -> MintasResult<Expr> {
        self.expect(&Token::LeftParen)?;
        let connection_string = match self.current_token() {
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return Err(MintasError::ParseError {
                message: "Expected database connection string".to_string(),
                location: self.current_location(),
            }),
        };
        self.expect(&Token::RightParen)?;
        Ok(Expr::DewDatabase {
            server: Box::new(Expr::Variable(server_name)),
            connection_string,
        })
    }
    fn parse_dew_session(&mut self, server_name: String) -> MintasResult<Expr> {
        self.expect(&Token::LeftParen)?;
        let config = if !matches!(self.current_token(), Some(Token::RightParen)) {
            Some(Box::new(self.parse_logical_or()?))
        } else {
            None
        };
        self.expect(&Token::RightParen)?;
        Ok(Expr::DewSession {
            server: Box::new(Expr::Variable(server_name)),
            config,
        })
    }
    fn parse_dew_rate_limit(&mut self, server_name: String) -> MintasResult<Expr> {
        self.expect(&Token::LeftParen)?;
        let requests = match self.current_token() {
            Some(Token::Number(n)) => {
                let n = *n;
                self.advance();
                n as u32
            }
            _ => 100, 
        };
        let window = if matches!(self.current_token(), Some(Token::Comma)) {
            self.advance();
            match self.current_token() {
                Some(Token::Number(n)) => {
                    let n = *n;
                    self.advance();
                    n as u32
                }
                _ => 60,
            }
        } else {
            60
        };
        self.expect(&Token::RightParen)?;
        Ok(Expr::DewRateLimit {
            server: Box::new(Expr::Variable(server_name)),
            requests,
            window_seconds: window,
        })
    }
}