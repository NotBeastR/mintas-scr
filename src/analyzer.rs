use crate::errors::MintasResult;
use crate::parser::Expr;
use std::collections::HashMap;

// SECURITY THREAT DETECTION LEVELS
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ThreatLevel {
    Safe,
    Suspicious,
    Dangerous,
    Critical,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SecurityThreat {
    pub level: ThreatLevel,
    pub description: String,
    pub line: usize,
    pub mitigation: String,
}

#[allow(dead_code)]
pub struct CodeAnalyzer {
    scopes: Vec<HashMap<String, VariableInfo>>,
    functions: HashMap<String, FunctionInfo>,
    warnings: Vec<String>,
    // SECURITY SUPERPOWERS
    security_threats: Vec<SecurityThreat>,
    recursion_depth: usize,
    loop_nesting: usize,
    memory_allocations: usize,
    suspicious_patterns: Vec<String>,
}

#[derive(Debug, Clone)]
struct VariableInfo {
    defined_at: usize,
    used_count: usize,
    #[allow(dead_code)]
    is_constant: bool,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    defined_at: usize,
    has_return: bool,
    param_count: usize,
}

impl CodeAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Start with global scope
            functions: HashMap::new(),
            warnings: Vec::new(),
            // SECURITY SUPERPOWERS INITIALIZATION
            security_threats: Vec::new(),
            recursion_depth: 0,
            loop_nesting: 0,
            memory_allocations: 0,
            suspicious_patterns: Vec::new(),
        }
    }

    /// SUPERPOWER: Advanced security analysis with threat detection
    pub fn analyze(&mut self, statements: &[Expr]) -> MintasResult<()> {
        self.warnings.clear();
        self.security_threats.clear();
        self.suspicious_patterns.clear();
        self.scopes.clear();
        self.scopes.push(HashMap::new()); // Reset to global scope

        // Silent analysis - no debug output

        // SUPERPOWER PASS 1: Security threat detection
        for (line_num, stmt) in statements.iter().enumerate() {
            self.detect_security_threats(stmt, line_num)?;
        }

        // SUPERPOWER PASS 2: Pattern analysis for attack vectors
        for (line_num, stmt) in statements.iter().enumerate() {
            self.analyze_attack_patterns(stmt, line_num)?;
        }

        // SUPERPOWER PASS 3: Resource usage analysis
        for (line_num, stmt) in statements.iter().enumerate() {
            self.analyze_resource_usage(stmt, line_num)?;
        }

        // Enhanced 3-pass analysis for Mintas-2
        
        // Pass 4: Syntax and structure validation
        for (line_num, stmt) in statements.iter().enumerate() {
            self.validate_syntax(stmt, line_num)?;
        }

        // Pass 5: Logic analysis - collect definitions and check usage
        for (line_num, stmt) in statements.iter().enumerate() {
            self.collect_functions(stmt, line_num);
        }
        
        for (line_num, stmt) in statements.iter().enumerate() {
            self.analyze_statement(stmt, line_num)?;
        }

        // Pass 3: Optimization and advanced error detection
        self.check_for_issues();
        self.check_logical_errors(statements);
        self.check_security_issues(statements);

        Ok(())
    }

    fn collect_functions(&mut self, expr: &Expr, line_num: usize) {
        match expr {
            Expr::Function { name, params, .. } => {
                self.functions.insert(name.clone(), FunctionInfo {
                    defined_at: line_num,
                    has_return: false,
                    param_count: params.len(),
                });
            }
            _ => {}
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            // Check for unused variables in the scope being exited
            for (name, info) in scope {
                if info.used_count == 0 && !name.starts_with('_') {
                    self.warnings.push(format!("Line {}: Unused variable '{}'. Consider removing or using the variable to improve code clarity.",
                        info.defined_at + 1, name));
                }
            }
        }
    }

    fn define_variable(&mut self, name: String, line_num: usize, is_constant: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, VariableInfo {
                defined_at: line_num,
                used_count: 0,
                is_constant,
            });
        }
    }

    fn use_variable(&mut self, name: &str) -> bool {
        // Check scopes from innermost to outermost
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                var_info.used_count += 1;
                return true;
            }
        }
        false
    }

    fn analyze_statement(&mut self, expr: &Expr, line_num: usize) -> MintasResult<()> {
        match expr {
            Expr::Variable(name) => {
                if !self.use_variable(name) {
                    if !self.functions.contains_key(name) {
                        self.warnings.push(format!("Line {}: Reference to undefined variable '{}'. Consider checking variable scope or initialization order.", line_num + 1, name));
                    }
                }
            }
            Expr::Assign { name, value, is_const } => {
                self.analyze_expression(value, line_num)?;
                // If variable exists in any scope, mark it as used/updated
                // Otherwise define it in current scope
                if !self.use_variable(name) {
                    self.define_variable(name.clone(), line_num, *is_const);
                }
            }
            Expr::MultiAssign { names, values, is_const } => {
                for value in values {
                    self.analyze_expression(value, line_num)?;
                }
                for name in names {
                    if !self.use_variable(name) {
                        self.define_variable(name.clone(), line_num, *is_const);
                    }
                }
            }
            Expr::CompoundAssign { name, value, .. } => {
                self.analyze_expression(value, line_num)?;
                if !self.use_variable(name) {
                     self.warnings.push(format!("Line {}: Reference to undefined variable '{}' in compound assignment.", line_num + 1, name));
                }
            }
            Expr::Call { name, args } => {
                if !self.functions.contains_key(name) && !self.is_builtin_function(name) {
                    // Check if it's a variable holding a function (lambda or assigned function)
                     if !self.use_variable(name) {
                        self.warnings.push(format!("Line {}: Call to undefined function '{}'. Function must be defined before use or imported from a module.", line_num + 1, name));
                    }
                }
                for arg in args {
                    self.analyze_expression(arg, line_num)?;
                }
            }
            Expr::IfExpr { condition, then_branch, else_if_branches, else_branch } => {
                self.analyze_expression(condition, line_num)?;
                // New scope for then block? Usually blocks share scope in simple scripting, but safer to add scope
                // Assuming block scope for safety
                self.enter_scope();
                self.analyze_block(then_branch, line_num)?;
                self.exit_scope();

                for (cond, branch) in else_if_branches {
                    self.analyze_expression(cond, line_num)?;
                    self.enter_scope();
                    self.analyze_block(branch, line_num)?;
                    self.exit_scope();
                }
                if let Some(branch) = else_branch {
                    self.enter_scope();
                    self.analyze_block(branch, line_num)?;
                    self.exit_scope();
                }
            }
            Expr::WhileLoop { condition, body } => {
                self.analyze_expression(condition, line_num)?;
                self.enter_scope();
                self.analyze_block(body, line_num)?;
                self.exit_scope();
            }
            Expr::ForLoop { var, start, end, body } => {
                self.analyze_expression(start, line_num)?;
                self.analyze_expression(end, line_num)?;
                self.enter_scope();
                self.define_variable(var.clone(), line_num, false);
                self.analyze_block(body, line_num)?;
                self.exit_scope();
            }
            Expr::ForInLoop { var, iterable, body } => {
                self.analyze_expression(iterable, line_num)?;
                self.enter_scope();
                self.define_variable(var.clone(), line_num, false);
                self.analyze_block(body, line_num)?;
                self.exit_scope();
            }
            Expr::Function { params, body, .. } => {
                self.enter_scope();
                for param in params {
                    self.define_variable(param.clone(), line_num, false);
                }
                self.analyze_block(body, line_num)?;
                self.exit_scope();
            }
            Expr::Return { value } => {
                if let Some(val) = value {
                    self.analyze_expression(val, line_num)?;
                }
            }
            Expr::BinaryOp { left, right, .. } => {
                self.analyze_expression(left, line_num)?;
                self.analyze_expression(right, line_num)?;
            }
            Expr::UnaryOp { expr, .. } => {
                self.analyze_expression(expr, line_num)?;
            }
            Expr::Cond { condition } => {
                self.analyze_expression(condition, line_num)?;
            }
            Expr::Follow { condition, .. } => {
                self.analyze_expression(condition, line_num)?;
            }
            Expr::Task { body, .. } => {
                self.enter_scope();
                self.analyze_block(body, line_num)?;
                self.exit_scope();
            }
            Expr::Switch { expression, cases, default_case } => {
                self.analyze_expression(expression, line_num)?;
                for (case_value, case_body) in cases {
                    self.analyze_expression(case_value, line_num)?;
                    self.enter_scope();
                    self.analyze_block(case_body, line_num)?;
                    self.exit_scope();
                }
                if let Some(default_body) = default_case {
                    self.enter_scope();
                    self.analyze_block(default_body, line_num)?;
                    self.exit_scope();
                }
            }
            // Dew Web Framework expressions - skip analysis (handled at runtime)
            Expr::DewRoute { server, body, .. } => {
                self.analyze_expression(server, line_num)?;
                self.enter_scope();
                self.analyze_block(body, line_num)?;
                self.exit_scope();
            }
            Expr::DewServe { server, port, host } => {
                self.analyze_expression(server, line_num)?;
                self.analyze_expression(port, line_num)?;
                if let Some(h) = host {
                    self.analyze_expression(h, line_num)?;
                }
            }
            Expr::DewReturn { body, status, data, .. } => {
                self.analyze_expression(body, line_num)?;
                if let Some(s) = status {
                    self.analyze_expression(s, line_num)?;
                }
                if let Some(d) = data {
                    self.analyze_expression(d, line_num)?;
                }
            }
            Expr::DewBefore { server, body } => {
                self.analyze_expression(server, line_num)?;
                self.enter_scope();
                self.analyze_block(body, line_num)?;
                self.exit_scope();
            }
            Expr::DewAfter { server, body } => {
                self.analyze_expression(server, line_num)?;
                self.enter_scope();
                self.analyze_block(body, line_num)?;
                self.exit_scope();
            }
            Expr::DewUse { server, .. } => {
                self.analyze_expression(server, line_num)?;
            }
            Expr::DewCatch { server, body, .. } => {
                self.analyze_expression(server, line_num)?;
                self.enter_scope();
                self.analyze_block(body, line_num)?;
                self.exit_scope();
            }
            Expr::DewGroup { server, body, .. } => {
                self.analyze_expression(server, line_num)?;
                self.enter_scope();
                self.analyze_block(body, line_num)?;
                self.exit_scope();
            }
            Expr::DewStatic { server, .. } => {
                self.analyze_expression(server, line_num)?;
            }
            Expr::DewRouteValidated { server, validation_rules, body, .. } => {
                self.analyze_expression(server, line_num)?;
                self.analyze_expression(validation_rules, line_num)?;
                self.enter_scope();
                self.analyze_block(body, line_num)?;
                self.exit_scope();
            }
            Expr::Getback => {
                // Getback is a special variable available in route handlers
            }
            _ => {}
        }
        Ok(())
    }

    pub fn analyze_expression(&mut self, expr: &Expr, line_num: usize) -> MintasResult<()> {
        self.analyze_statement(expr, line_num)
    }

    fn analyze_block(&mut self, statements: &[Expr], line_num: usize) -> MintasResult<()> {
        for stmt in statements {
            self.analyze_statement(stmt, line_num)?;
        }
        Ok(())
    }

    fn check_for_issues(&mut self) {
        // Only check global scope variables here, as local ones are checked on exit_scope
        if let Some(global_scope) = self.scopes.first() {
            for (name, info) in global_scope {
                if info.used_count == 0 && !name.starts_with('_') {
                    self.warnings.push(format!("Line {}: Unused variable '{}'. Consider removing or using the variable to improve code clarity.",
                        info.defined_at + 1, name));
                }
            }
        }

        for (name, info) in &self.functions {
            if !info.has_return && info.param_count > 0 {
                // This warning is a bit simplistic as it doesn't check control flow properly
                // Keeping it for now but it might be one of the "useless" warnings
                self.warnings.push(format!("Line {}: Function '{}' may not return a value on all execution paths. Consider adding return statements or default values.",
                    info.defined_at + 1, name));
            }
        }
    }

    fn is_builtin_function(&self, name: &str) -> bool {
        if name.contains('.') {
            if let Some(dot_pos) = name.find('.') {
                let module_name = &name[..dot_pos];
                let func_name = &name[dot_pos + 1..];

                match module_name {
                    "math" => {
                        matches!(func_name, "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "atan2" |
                                 "sinh" | "cosh" | "tanh" | "sqrt" | "cbrt" | "pow" | "exp" | "exp2" |
                                 "ln" | "log10" | "log2" | "abs" | "floor" | "ceil" | "round" | "trunc" |
                                 "min" | "max" | "random" | "pi" | "e")
                    }
                    "datetime" => {
                        matches!(func_name, "now" | "today" | "utcnow" | "timestamp" | "fromtimestamp" |
                                 "strptime" | "strftime" | "add_days" | "add_hours" | "add_minutes" | "add_seconds" |
                                 "diff_days" | "diff_hours" | "diff_minutes" | "diff_seconds" |
                                 "is_leap_year" | "days_in_month" | "days_in_year" | "weekday" | "isoformat" | "parse")
                    }
                    "json" => {
                        matches!(func_name, "encode" | "decode" | "pretty" | "stringify" | "parse" |
                                 "get" | "set" | "keys" | "values" | "has_key" | "is_valid" |
                                 "merge" | "to_table" | "from_table")
                    }
                    _ => false
                }
            } else {
                false
            }
        } else {
            matches!(name, "say" | "ask" | "read" | "write" | "append" | "exists" |
                     "len" | "upper" | "lower" | "trim" | "push" | "pop" | "insert" |
                     "remove" | "sort" | "reverse" | "contains" | "find" | "replace" |
                     "split" | "join" | "keys" | "values" | "has" | "merge" |
                     "typeof" | "tostring" | "tonumber" | "assert" | "test" |
                     "cond" | "follow")
        }
    }

    #[allow(dead_code)]
    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }

    #[allow(dead_code)]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    #[allow(dead_code)]
    pub fn print_warnings(&self) {
        if self.has_warnings() {
            println!("⚠️  Code Analysis Warnings:");
            for warning in &self.warnings {
                println!("   {}", warning);
            }
            println!();
        }
    }

    // Enhanced analysis methods for Mintas-2

    fn validate_syntax(&mut self, expr: &Expr, line_num: usize) -> MintasResult<()> {
        // Pass 1: Syntax validation - check for structural issues
        match expr {
            Expr::Variable(name) => {
                if name.len() > 32 {
                    self.warnings.push(format!("Line {}: Variable name '{}' exceeds 32 character limit", line_num + 1, name));
                }
                if name.chars().any(|c| !c.is_ascii_alphanumeric() && c != '_') {
                    self.warnings.push(format!("Line {}: Variable name '{}' contains invalid characters", line_num + 1, name));
                }
            }
            Expr::BinaryOp { left, right, .. } => {
                self.validate_syntax(left, line_num)?;
                self.validate_syntax(right, line_num)?;
            }
            Expr::UnaryOp { expr, .. } => {
                self.validate_syntax(expr, line_num)?;
            }
            Expr::Array(elements) => {
                for elem in elements {
                    self.validate_syntax(elem, line_num)?;
                }
            }
            Expr::Table(pairs) => {
                for (_, value) in pairs {
                    self.validate_syntax(value, line_num)?;
                }
            }
            Expr::SuperSet(inner) => {
                self.validate_syntax(inner, line_num)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn check_logical_errors(&mut self, statements: &[Expr]) {
        // Pass 3: Advanced logical error detection
        for (line_num, stmt) in statements.iter().enumerate() {
            self.check_statement_logic(stmt, line_num);
        }
    }

    fn check_statement_logic(&mut self, expr: &Expr, line_num: usize) {
        match expr {
            Expr::BinaryOp { op, left, right } => {
                // Check for potential division by zero
                if matches!(op, crate::parser::BinaryOp::Divide) {
                    if let Expr::Number(n) = &**right {
                        if *n == 0.0 {
                            self.warnings.push(format!("Line {}: Potential division by zero detected", line_num + 1));
                        }
                    }
                }
                
                // Check for comparison with different types
                self.check_type_consistency(left, right, line_num);
                self.check_statement_logic(left, line_num);
                self.check_statement_logic(right, line_num);
            }
            Expr::IfExpr { condition, then_branch, else_if_branches, else_branch } => {
                // Check for unreachable code
                if let Expr::Boolean(false) = &**condition {
                    self.warnings.push(format!("Line {}: Unreachable code - condition is always false", line_num + 1));
                }
                
                // Check if all branches have consistent return behavior
                let then_has_exit = self.has_exit_in_block(then_branch);
                for (_, branch) in else_if_branches {
                    let branch_has_exit = self.has_exit_in_block(branch);
                    if then_has_exit != branch_has_exit {
                        self.warnings.push(format!("Line {}: Inconsistent exit behavior between branches", line_num + 1));
                    }
                }
                if let Some(else_body) = else_branch {
                    let else_has_exit = self.has_exit_in_block(else_body);
                    if then_has_exit != else_has_exit {
                        self.warnings.push(format!("Line {}: Inconsistent exit behavior between if and else", line_num + 1));
                    }
                }
            }
            Expr::Array(elements) => {
                // Check for mixed types in arrays (warning)
                if elements.len() > 1 {
                    let first_type = self.infer_expression_type(&elements[0]);
                    for (i, elem) in elements.iter().enumerate().skip(1) {
                        let elem_type = self.infer_expression_type(elem);
                        if first_type != elem_type && first_type != "unknown" && elem_type != "unknown" {
                            self.warnings.push(format!("Line {}: Mixed types in array at index {} (expected {}, got {})", 
                                line_num + 1, i + 1, first_type, elem_type));
                            break;
                        }
                    }
                }
                for elem in elements {
                    self.check_statement_logic(elem, line_num);
                }
            }
            Expr::WhileLoop { condition, body } => {
                // Check for potential infinite loops
                if let Expr::Boolean(true) = &**condition {
                    if !self.has_exit_in_block(body) {
                        self.warnings.push(format!("Line {}: Potential infinite loop detected (condition is always true)", line_num + 1));
                    }
                }
                self.check_statement_logic(condition, line_num);
                for stmt in body {
                    self.check_statement_logic(stmt, line_num);
                }
            }
            _ => {}
        }
    }

    fn check_security_issues(&mut self, statements: &[Expr]) {
        // Security analysis - detect unsafe patterns
        for (line_num, stmt) in statements.iter().enumerate() {
            self.check_statement_security(stmt, line_num);
        }
    }

    fn check_statement_security(&mut self, expr: &Expr, line_num: usize) {
        match expr {
            Expr::Call { name, args } => {
                // Check for potentially unsafe operations
                match name.as_str() {
                    "write" | "append" => {
                        if let Some(Expr::String(path)) = args.get(0) {
                            if path.contains("..") || path.starts_with('/') {
                                self.warnings.push(format!("Line {}: Potentially unsafe file path: '{}'", line_num + 1, path));
                            }
                        }
                    }
                    "ask" => {
                        // Check for input validation
                        self.warnings.push(format!("Line {}: Consider validating user input from ask()", line_num + 1));
                    }
                    _ => {}
                }
                for arg in args {
                    self.check_statement_security(arg, line_num);
                }
            }
            Expr::BinaryOp { left, right, .. } => {
                self.check_statement_security(left, line_num);
                self.check_statement_security(right, line_num);
            }
            _ => {}
        }
    }

    fn check_type_consistency(&mut self, left: &Expr, right: &Expr, line_num: usize) {
        let left_type = self.infer_expression_type(left);
        let right_type = self.infer_expression_type(right);
        
        if left_type != right_type && left_type != "unknown" && right_type != "unknown" {
            self.warnings.push(format!("Line {}: Type mismatch in operation ({} vs {})", 
                line_num + 1, left_type, right_type));
        }
    }

    fn infer_expression_type(&self, expr: &Expr) -> &'static str {
        match expr {
            Expr::Number(_) => "number",
            Expr::String(_) => "string",
            Expr::Boolean(_) => "boolean",
            Expr::Maybe => "maybe",
            Expr::Empty => "empty",
            Expr::Array(_) => "array",
            Expr::Table(_) => "table",
            Expr::SuperSet(_) => "superset",
            _ => "unknown",
        }
    }

    fn has_exit_in_block(&self, block: &[Expr]) -> bool {
        for stmt in block {
            match stmt {
                Expr::Exit => return true,
                Expr::Return { .. } => return true,
                Expr::IfExpr { then_branch, else_branch, .. } => {
                    if self.has_exit_in_block(then_branch) {
                        if let Some(else_body) = else_branch {
                            if self.has_exit_in_block(else_body) {
                                return true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    // ============================================================================
    // SECURITY SUPERPOWERS - Advanced threat detection beyond Rust's guarantees
    // ============================================================================

    /// SUPERPOWER: Detect security threats at compile time
    fn detect_security_threats(&mut self, expr: &Expr, line_num: usize) -> MintasResult<()> {
        match expr {
            // Detect infinite recursion patterns
            Expr::Function { name, body, .. } => {
                if self.contains_self_call(body, name) {
                    self.add_security_threat(
                        ThreatLevel::Critical,
                        format!("Potential infinite recursion detected in function '{}'", name),
                        line_num,
                        "Add base case or recursion limit checks".to_string(),
                    );
                }
            }
            
            // Detect memory bomb patterns
            Expr::ForLoop { start, end, body, .. } => {
                if let (Expr::Number(s), Expr::Number(e)) = (start.as_ref(), end.as_ref()) {
                    let iterations = (e - s).abs() as usize;
                    if iterations > 1_000_000 {
                        self.add_security_threat(
                            ThreatLevel::Dangerous,
                            format!("Large loop detected: {} iterations may cause DoS", iterations),
                            line_num,
                            "Consider reducing loop size or adding progress checks".to_string(),
                        );
                    }
                }
                
                // Check for memory allocations in loops
                if self.contains_memory_allocation(body) {
                    self.add_security_threat(
                        ThreatLevel::Suspicious,
                        "Memory allocation inside loop detected".to_string(),
                        line_num,
                        "Pre-allocate memory outside loop when possible".to_string(),
                    );
                }
            }
            
            // Detect array size bombs
            Expr::Array(elements) => {
                if elements.len() > 10000 {
                    self.add_security_threat(
                        ThreatLevel::Dangerous,
                        format!("Large array literal: {} elements", elements.len()),
                        line_num,
                        "Consider dynamic allocation or data streaming".to_string(),
                    );
                }
            }
            
            // Detect string bombs
            Expr::String(s) => {
                if s.len() > 100000 {
                    self.add_security_threat(
                        ThreatLevel::Dangerous,
                        format!("Large string literal: {} characters", s.len()),
                        line_num,
                        "Consider file-based storage for large text data".to_string(),
                    );
                }
            }
            
            _ => {}
        }
        Ok(())
    }

    /// SUPERPOWER: Analyze attack patterns
    fn analyze_attack_patterns(&mut self, expr: &Expr, line_num: usize) -> MintasResult<()> {
        match expr {
            Expr::Call { name, args } => {
                // Detect potential injection patterns
                for arg in args {
                    if let Expr::String(s) = arg {
                        if self.is_injection_pattern(s) {
                            self.add_security_threat(
                                ThreatLevel::Critical,
                                "Potential injection attack pattern detected".to_string(),
                                line_num,
                                "Sanitize input and use parameterized queries".to_string(),
                            );
                        }
                    }
                }
                
                // Detect dangerous function calls
                if self.is_dangerous_function(name) {
                    self.add_security_threat(
                        ThreatLevel::Suspicious,
                        format!("Potentially dangerous function call: {}", name),
                        line_num,
                        "Ensure proper input validation and error handling".to_string(),
                    );
                }
            }
            
            Expr::BinaryOp { op, left: _, right } => {
                // Detect division by zero patterns
                if matches!(op, crate::parser::BinaryOp::Divide) {
                    if let Expr::Number(n) = right.as_ref() {
                        if *n == 0.0 {
                            self.add_security_threat(
                                ThreatLevel::Dangerous,
                                "Division by zero detected".to_string(),
                                line_num,
                                "Add zero check before division".to_string(),
                            );
                        }
                    }
                }
            }
            
            _ => {}
        }
        Ok(())
    }

    /// SUPERPOWER: Analyze resource usage patterns
    fn analyze_resource_usage(&mut self, expr: &Expr, line_num: usize) -> MintasResult<()> {
        match expr {
            Expr::WhileLoop { condition, .. } => {
                // Check for potential infinite loops
                if let Expr::Boolean(true) = condition.as_ref() {
                    self.add_security_threat(
                        ThreatLevel::Critical,
                        "Infinite loop detected: while(true)".to_string(),
                        line_num,
                        "Add break condition or timeout mechanism".to_string(),
                    );
                }
            }
            
            Expr::ForLoop { .. } => {
                self.loop_nesting += 1;
                if self.loop_nesting > 3 {
                    self.add_security_threat(
                        ThreatLevel::Suspicious,
                        format!("Deep loop nesting: {} levels", self.loop_nesting),
                        line_num,
                        "Consider refactoring to reduce complexity".to_string(),
                    );
                }
            }
            
            _ => {}
        }
        Ok(())
    }

    /// Helper: Add security threat to the list
    fn add_security_threat(&mut self, level: ThreatLevel, description: String, line: usize, mitigation: String) {
        self.security_threats.push(SecurityThreat {
            level,
            description,
            line: line + 1, // Convert to 1-based line numbers
            mitigation,
        });
    }

    /// Helper: Check if function contains self-call (recursion)
    fn contains_self_call(&self, body: &[Expr], function_name: &str) -> bool {
        for expr in body {
            if let Expr::Call { name, .. } = expr {
                if name == function_name {
                    return true;
                }
            }
        }
        false
    }

    /// Helper: Check if code contains memory allocation
    fn contains_memory_allocation(&self, body: &[Expr]) -> bool {
        for expr in body {
            match expr {
                Expr::Array(_) | Expr::String(_) => return true,
                Expr::Call { name, .. } if name == "push" => return true,
                _ => {}
            }
        }
        false
    }

    /// Helper: Check for injection patterns
    fn is_injection_pattern(&self, s: &str) -> bool {
        let dangerous_patterns = [
            "'; DROP TABLE",
            "' OR '1'='1",
            "<script>",
            "javascript:",
            "../../../",
            "rm -rf",
            "system(",
            "exec(",
            "eval(",
        ];
        
        for pattern in &dangerous_patterns {
            if s.to_lowercase().contains(&pattern.to_lowercase()) {
                return true;
            }
        }
        false
    }

    /// Helper: Check for dangerous functions
    fn is_dangerous_function(&self, name: &str) -> bool {
        matches!(name, "system" | "exec" | "eval" | "shell" | "subprocess.run")
    }

    /// SUPERPOWER: Get comprehensive security report
    #[allow(dead_code)]
    pub fn get_security_report(&self) -> String {
        let mut report = String::new();
        
        if self.security_threats.is_empty() {
            report.push_str("🛡️ SECURITY STATUS: CLEAN\n");
            report.push_str("No security threats detected.\n");
        } else {
            report.push_str("🚨 SECURITY THREATS DETECTED:\n\n");
            
            let mut critical = 0;
            let mut dangerous = 0;
            let mut suspicious = 0;
            
            for threat in &self.security_threats {
                match threat.level {
                    ThreatLevel::Critical => {
                        critical += 1;
                        report.push_str(&format!("🔴 CRITICAL (Line {}): {}\n", threat.line, threat.description));
                        report.push_str(&format!("   Mitigation: {}\n\n", threat.mitigation));
                    }
                    ThreatLevel::Dangerous => {
                        dangerous += 1;
                        report.push_str(&format!("🟠 DANGEROUS (Line {}): {}\n", threat.line, threat.description));
                        report.push_str(&format!("   Mitigation: {}\n\n", threat.mitigation));
                    }
                    ThreatLevel::Suspicious => {
                        suspicious += 1;
                        report.push_str(&format!("🟡 SUSPICIOUS (Line {}): {}\n", threat.line, threat.description));
                        report.push_str(&format!("   Mitigation: {}\n\n", threat.mitigation));
                    }
                    ThreatLevel::Safe => {}
                }
            }
            
            report.push_str(&format!("THREAT SUMMARY:\n"));
            report.push_str(&format!("- Critical: {}\n", critical));
            report.push_str(&format!("- Dangerous: {}\n", dangerous));
            report.push_str(&format!("- Suspicious: {}\n", suspicious));
            
            if critical > 0 {
                report.push_str("\n❌ SECURITY VERDICT: UNSAFE - Critical threats must be resolved\n");
            } else if dangerous > 0 {
                report.push_str("\n⚠️ SECURITY VERDICT: RISKY - Dangerous patterns detected\n");
            } else {
                report.push_str("\n✅ SECURITY VERDICT: ACCEPTABLE - Only minor issues\n");
            }
        }
        
        report
    }

    /// SUPERPOWER: Check if code passes security validation
    #[allow(dead_code)]
    pub fn is_secure(&self) -> bool {
        !self.security_threats.iter().any(|t| matches!(t.level, ThreatLevel::Critical | ThreatLevel::Dangerous))
    }
}