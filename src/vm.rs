use crate::bytecode::{BytecodeProgram, Constant, Instruction};
use crate::evaluator::Value;
use crate::errors::{MintasError, MintasResult, SourceLocation};
use std::collections::HashMap;

/// Stack-based virtual machine for executing bytecode
pub struct BytecodeVM {
    stack: Vec<Value>,
    variables: HashMap<String, Value>,
    program: BytecodeProgram,
    ip: usize, // Instruction pointer
}

impl BytecodeVM {
    pub fn new(program: BytecodeProgram) -> Self {
        Self {
            stack: Vec::new(),
            variables: HashMap::new(),
            program,
            ip: 0,
        }
    }
    
    /// Execute the bytecode program
    pub fn execute(&mut self) -> MintasResult<Value> {
        while self.ip < self.program.instructions.len() {
            let instruction = self.program.instructions[self.ip].clone();
            self.ip += 1;
            
            match instruction {
                Instruction::LoadConst(idx) => {
                    let constant = &self.program.constants[idx];
                    let value = match constant {
                        Constant::Number(n) => Value::Number(*n),
                        Constant::String(s) => Value::String(s.clone()),
                        Constant::Boolean(b) => Value::Boolean(*b),
                    };
                    self.stack.push(value);
                }
                
                Instruction::LoadString(idx) => {
                    let s = self.program.strings[idx].clone();
                    self.stack.push(Value::String(s));
                }
                
                Instruction::LoadTrue => self.stack.push(Value::Boolean(true)),
                Instruction::LoadFalse => self.stack.push(Value::Boolean(false)),
                Instruction::LoadMaybe => self.stack.push(Value::Maybe),
                Instruction::LoadEmpty => self.stack.push(Value::Empty),
                
                Instruction::LoadVar(name) => {
                    let value = self.variables.get(&name)
                        .cloned()
                        .ok_or_else(|| MintasError::UndefinedVariable {
                            name: name.clone(),
                            location: SourceLocation::new(0, 0),
                        })?;
                    self.stack.push(value);
                }
                
                Instruction::StoreVar(name) => {
                    let value = self.stack.pop()
                        .ok_or_else(|| MintasError::RuntimeError {
                            message: "Stack underflow".to_string(),
                            location: SourceLocation::new(0, 0),
                        })?;
                    self.variables.insert(name, value);
                }
                
                Instruction::Add => self.binary_op(|a, b| a + b)?,
                Instruction::Sub => self.binary_op(|a, b| a - b)?,
                Instruction::Mul => self.binary_op(|a, b| a * b)?,
                Instruction::Div => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    if b == 0.0 {
                        return Err(MintasError::DivisionByZero {
                            location: SourceLocation::new(0, 0),
                        });
                    }
                    self.stack.push(Value::Number(a / b));
                }
                Instruction::Mod => self.binary_op(|a, b| a % b)?,
                Instruction::Neg => {
                    let n = self.pop_number()?;
                    self.stack.push(Value::Number(-n));
                }
                
                Instruction::Eq => self.comparison_op(|a, b| (a - b).abs() < f64::EPSILON)?,
                Instruction::NotEq => self.comparison_op(|a, b| (a - b).abs() >= f64::EPSILON)?,
                Instruction::Greater => self.comparison_op(|a, b| a > b)?,
                Instruction::Less => self.comparison_op(|a, b| a < b)?,
                Instruction::GreaterEq => self.comparison_op(|a, b| a >= b)?,
                Instruction::LessEq => self.comparison_op(|a, b| a <= b)?,
                
                Instruction::And => {
                    let b = self.stack.pop().ok_or_else(|| self.stack_underflow())?;
                    let a = self.stack.pop().ok_or_else(|| self.stack_underflow())?;
                    let result = a.is_truthy() && b.is_truthy();
                    self.stack.push(Value::Boolean(result));
                }
                
                Instruction::Or => {
                    let b = self.stack.pop().ok_or_else(|| self.stack_underflow())?;
                    let a = self.stack.pop().ok_or_else(|| self.stack_underflow())?;
                    let result = a.is_truthy() || b.is_truthy();
                    self.stack.push(Value::Boolean(result));
                }
                
                Instruction::Not => {
                    let val = self.stack.pop().ok_or_else(|| self.stack_underflow())?;
                    self.stack.push(Value::Boolean(!val.is_truthy()));
                }
                
                Instruction::Jump(target) => {
                    self.ip = target;
                }
                
                Instruction::JumpIfFalse(target) => {
                    let condition = self.stack.pop().ok_or_else(|| self.stack_underflow())?;
                    if !condition.is_truthy() {
                        self.ip = target;
                    }
                }
                
                Instruction::JumpIfTrue(target) => {
                    let condition = self.stack.pop().ok_or_else(|| self.stack_underflow())?;
                    if condition.is_truthy() {
                        self.ip = target;
                    }
                }
                
                Instruction::Call(name, argc) => {
                    // For now, handle built-in functions
                    if name == "say" {
                        let arg = if argc > 0 {
                            self.stack.pop().ok_or_else(|| self.stack_underflow())?
                        } else {
                            Value::Empty
                        };
                        self.print_value(&arg);
                        self.stack.push(Value::Empty);
                    } else {
                        return Err(MintasError::UnknownFunction {
                            name,
                            location: SourceLocation::new(0, 0),
                        });
                    }
                }
                
                Instruction::Return => {
                    let value = self.stack.pop().unwrap_or(Value::Empty);
                    return Ok(value);
                }
                
                Instruction::MakeArray(count) => {
                    let mut elements = Vec::new();
                    for _ in 0..count {
                        elements.push(self.stack.pop().ok_or_else(|| self.stack_underflow())?);
                    }
                    elements.reverse();
                    self.stack.push(Value::Array(elements));
                }
                
                Instruction::MakeTable(count) => {
                    let mut map = HashMap::new();
                    for _ in 0..count {
                        let value = self.stack.pop().ok_or_else(|| self.stack_underflow())?;
                        let key = self.stack.pop().ok_or_else(|| self.stack_underflow())?;
                        if let Value::String(k) = key {
                            map.insert(k, value);
                        }
                    }
                    self.stack.push(Value::Table(map));
                }
                
                Instruction::Pop => {
                    self.stack.pop();
                }
                
                Instruction::Dup => {
                    let value = self.stack.last()
                        .ok_or_else(|| self.stack_underflow())?
                        .clone();
                    self.stack.push(value);
                }
                
                Instruction::Halt => {
                    break;
                }
                
                _ => {
                    return Err(MintasError::RuntimeError {
                        message: format!("Unimplemented instruction: {:?}", instruction),
                        location: SourceLocation::new(0, 0),
                    });
                }
            }
        }
        
        // Return the top of the stack or Empty
        Ok(self.stack.pop().unwrap_or(Value::Empty))
    }
    
    fn pop_number(&mut self) -> MintasResult<f64> {
        match self.stack.pop() {
            Some(Value::Number(n)) => Ok(n),
            Some(other) => Err(MintasError::TypeError {
                message: format!("Expected number, got {}", other.type_name()),
                location: SourceLocation::new(0, 0),
            }),
            None => Err(self.stack_underflow()),
        }
    }
    
    fn binary_op<F>(&mut self, op: F) -> MintasResult<()>
    where
        F: FnOnce(f64, f64) -> f64,
    {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.stack.push(Value::Number(op(a, b)));
        Ok(())
    }
    
    fn comparison_op<F>(&mut self, op: F) -> MintasResult<()>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let b = self.pop_number()?;
        let a = self.pop_number()?;
        self.stack.push(Value::Boolean(op(a, b)));
        Ok(())
    }
    
    fn stack_underflow(&self) -> MintasError {
        MintasError::RuntimeError {
            message: "Stack underflow".to_string(),
            location: SourceLocation::new(0, 0),
        }
    }
    
    fn print_value(&self, val: &Value) {
        match val {
            Value::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    println!("{}", *n as i64);
                } else {
                    println!("{}", n);
                }
            }
            Value::String(s) => println!("{}", s),
            Value::Boolean(b) => println!("{}", b),
            Value::Empty => println!("empty"),
            Value::Maybe => println!("maybe"),
            Value::Array(arr) => {
                print!("[");
                for (i, elem) in arr.iter().enumerate() {
                    if i > 0 { print!(", "); }
                    print!("{:?}", elem);
                }
                println!("]");
            }
            Value::Table(map) => {
                println!("{:?}", map);
            }
            _ => println!("{:?}", val),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::BytecodeCompiler;
    use crate::parser::{Parser, Expr};
    use crate::lexer::Lexer;
    
    fn compile_and_run(code: &str) -> MintasResult<Value> {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        let mut compiler = BytecodeCompiler::new();
        let program = compiler.compile(&ast)?;
        
        let mut vm = BytecodeVM::new(program);
        vm.execute()
    }
    
    #[test]
    fn test_arithmetic() {
        let result = compile_and_run("2 + 3 * 4").unwrap();
        assert_eq!(result, Value::Number(14.0));
    }
    
    #[test]
    fn test_variables() {
        let result = compile_and_run("x = 10\ny = 20\nx + y").unwrap();
        assert_eq!(result, Value::Number(30.0));
    }
}
