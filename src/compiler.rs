use crate::bytecode::{BytecodeProgram, Constant, Instruction};
use crate::parser::{Expr, BinaryOp, UnaryOp};
use crate::errors::{MintasError, MintasResult, SourceLocation};

/// Bytecode compiler - converts AST to bytecode
pub struct BytecodeCompiler {
    program: BytecodeProgram,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {
            program: BytecodeProgram::new(),
        }
    }
    
    /// Compile a list of expressions into bytecode
    pub fn compile(&mut self, expressions: &[Expr]) -> MintasResult<BytecodeProgram> {
        for expr in expressions {
            self.compile_expr(expr)?;
            // Pop result if not the last expression
            if expr != expressions.last().unwrap() {
                self.program.emit(Instruction::Pop);
            }
        }
        
        self.program.emit(Instruction::Halt);
        Ok(self.program.clone())
    }
    
    /// Compile a single expression
    fn compile_expr(&mut self, expr: &Expr) -> MintasResult<()> {
        match expr {
            Expr::Number(n) => {
                let idx = self.program.add_constant(Constant::Number(*n));
                self.program.emit(Instruction::LoadConst(idx));
            }
            
            Expr::String(s) => {
                let idx = self.program.add_string(s.clone());
                self.program.emit(Instruction::LoadString(idx));
            }
            
            Expr::Boolean(b) => {
                if *b {
                    self.program.emit(Instruction::LoadTrue);
                } else {
                    self.program.emit(Instruction::LoadFalse);
                }
            }
            
            Expr::Maybe => {
                self.program.emit(Instruction::LoadMaybe);
            }
            
            Expr::Empty => {
                self.program.emit(Instruction::LoadEmpty);
            }
            
            Expr::Variable(name) => {
                self.program.emit(Instruction::LoadVar(name.clone()));
            }
            
            Expr::Assign { name, value, .. } => {
                self.compile_expr(value)?;
                self.program.emit(Instruction::Dup); // Keep value on stack
                self.program.emit(Instruction::StoreVar(name.clone()));
            }
            
            Expr::BinaryOp { op, left, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                self.compile_binary_op(op);
            }
            
            Expr::UnaryOp { op, expr } => {
                self.compile_expr(expr)?;
                self.compile_unary_op(op);
            }
            
            Expr::Array(elements) => {
                for elem in elements {
                    self.compile_expr(elem)?;
                }
                self.program.emit(Instruction::MakeArray(elements.len()));
            }
            
            Expr::Table(pairs) => {
                for (key, value) in pairs {
                    let key_idx = self.program.add_string(key.clone());
                    self.program.emit(Instruction::LoadString(key_idx));
                    self.compile_expr(value)?;
                }
                self.program.emit(Instruction::MakeTable(pairs.len()));
            }
            
            Expr::Call { name, args } => {
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.program.emit(Instruction::Call(name.clone(), args.len() as u8));
            }
            
            Expr::IfExpr { condition, then_branch, else_if_branches, else_branch } => {
                self.compile_if(condition, then_branch, else_if_branches, else_branch)?;
            }
            
            Expr::WhileLoop { condition, body } => {
                self.compile_while(condition, body)?;
            }
            
            Expr::ForLoop { var, start, end, body } => {
                self.compile_for(var, start, end, body)?;
            }
            
            Expr::Return { value } => {
                if let Some(val) = value {
                    self.compile_expr(val)?;
                } else {
                    self.program.emit(Instruction::LoadEmpty);
                }
                self.program.emit(Instruction::Return);
            }
            
            _ => {
                return Err(MintasError::CompileError {
                    message: format!("Unsupported expression type: {:?}", expr),
                    location: SourceLocation::new(0, 0),
                });
            }
        }
        
        Ok(())
    }
    
    fn compile_binary_op(&mut self, op: &BinaryOp) {
        let instruction = match op {
            BinaryOp::Add => Instruction::Add,
            BinaryOp::Subtract => Instruction::Sub,
            BinaryOp::Multiply => Instruction::Mul,
            BinaryOp::Divide => Instruction::Div,
            BinaryOp::Modulo => Instruction::Mod,
            BinaryOp::Equal => Instruction::Eq,
            BinaryOp::NotEqual => Instruction::NotEq,
            BinaryOp::Greater => Instruction::Greater,
            BinaryOp::Less => Instruction::Less,
            BinaryOp::GreaterEqual => Instruction::GreaterEq,
            BinaryOp::LessEqual => Instruction::LessEq,
            BinaryOp::And => Instruction::And,
            BinaryOp::Or => Instruction::Or,
            _ => Instruction::Pop, // Fallback
        };
        self.program.emit(instruction);
    }
    
    fn compile_unary_op(&mut self, op: &UnaryOp) {
        let instruction = match op {
            UnaryOp::Negate => Instruction::Neg,
            UnaryOp::Not => Instruction::Not,
            _ => Instruction::Pop, // Fallback
        };
        self.program.emit(instruction);
    }
    
    fn compile_if(
        &mut self,
        condition: &Expr,
        then_branch: &[Expr],
        else_if_branches: &[(Expr, Vec<Expr>)],
        else_branch: &Option<Vec<Expr>>,
    ) -> MintasResult<()> {
        // Compile condition
        self.compile_expr(condition)?;
        
        // Jump to else if condition is false
        let jump_to_else = self.program.current_index();
        self.program.emit(Instruction::JumpIfFalse(0)); // Placeholder
        
        // Compile then branch
        for expr in then_branch {
            self.compile_expr(expr)?;
            if expr != then_branch.last().unwrap() {
                self.program.emit(Instruction::Pop);
            }
        }
        
        // Jump to end after then branch
        let jump_to_end = self.program.current_index();
        self.program.emit(Instruction::Jump(0)); // Placeholder
        
        // Patch jump to else
        let else_start = self.program.current_index();
        self.program.patch_jump(jump_to_else, else_start);
        
        // Compile else-if branches
        for (elif_cond, elif_body) in else_if_branches {
            self.compile_expr(elif_cond)?;
            let elif_jump = self.program.current_index();
            self.program.emit(Instruction::JumpIfFalse(0));
            
            for expr in elif_body {
                self.compile_expr(expr)?;
                if expr != elif_body.last().unwrap() {
                    self.program.emit(Instruction::Pop);
                }
            }
            
            let elif_end_jump = self.program.current_index();
            self.program.emit(Instruction::Jump(0));
            
            let next_elif = self.program.current_index();
            self.program.patch_jump(elif_jump, next_elif);
        }
        
        // Compile else branch
        if let Some(else_body) = else_branch {
            for expr in else_body {
                self.compile_expr(expr)?;
                if expr != else_body.last().unwrap() {
                    self.program.emit(Instruction::Pop);
                }
            }
        } else {
            self.program.emit(Instruction::LoadEmpty);
        }
        
        // Patch jump to end
        let end = self.program.current_index();
        self.program.patch_jump(jump_to_end, end);
        
        Ok(())
    }
    
    fn compile_while(&mut self, condition: &Expr, body: &[Expr]) -> MintasResult<()> {
        let loop_start = self.program.current_index();
        
        // Compile condition
        self.compile_expr(condition)?;
        
        // Jump to end if condition is false
        let jump_to_end = self.program.current_index();
        self.program.emit(Instruction::JumpIfFalse(0)); // Placeholder
        
        // Compile body
        for expr in body {
            self.compile_expr(expr)?;
            self.program.emit(Instruction::Pop);
        }
        
        // Jump back to start
        self.program.emit(Instruction::Jump(loop_start));
        
        // Patch jump to end
        let end = self.program.current_index();
        self.program.patch_jump(jump_to_end, end);
        
        self.program.emit(Instruction::LoadEmpty);
        
        Ok(())
    }
    
    fn compile_for(&mut self, var: &str, start: &Expr, end: &Expr, body: &[Expr]) -> MintasResult<()> {
        // Initialize loop variable
        self.compile_expr(start)?;
        self.program.emit(Instruction::StoreVar(var.to_string()));
        
        let loop_start = self.program.current_index();
        
        // Check condition: var <= end
        self.program.emit(Instruction::LoadVar(var.to_string()));
        self.compile_expr(end)?;
        self.program.emit(Instruction::LessEq);
        
        // Jump to end if condition is false
        let jump_to_end = self.program.current_index();
        self.program.emit(Instruction::JumpIfFalse(0)); // Placeholder
        
        // Compile body
        for expr in body {
            self.compile_expr(expr)?;
            self.program.emit(Instruction::Pop);
        }
        
        // Increment loop variable
        self.program.emit(Instruction::LoadVar(var.to_string()));
        let one_idx = self.program.add_constant(Constant::Number(1.0));
        self.program.emit(Instruction::LoadConst(one_idx));
        self.program.emit(Instruction::Add);
        self.program.emit(Instruction::StoreVar(var.to_string()));
        self.program.emit(Instruction::Pop);
        
        // Jump back to start
        self.program.emit(Instruction::Jump(loop_start));
        
        // Patch jump to end
        let end_idx = self.program.current_index();
        self.program.patch_jump(jump_to_end, end_idx);
        
        self.program.emit(Instruction::LoadEmpty);
        
        Ok(())
    }
}

impl Default for BytecodeCompiler {
    fn default() -> Self {
        Self::new()
    }
}
