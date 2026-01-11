use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Bytecode instruction set for Mintas
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    // Constants and literals
    LoadConst(usize),           // Load constant from pool by index
    LoadString(usize),          // Load string from string table
    LoadTrue,
    LoadFalse,
    LoadMaybe,
    LoadEmpty,
    
    // Variables
    LoadVar(String),            // Load variable by name
    StoreVar(String),           // Store to variable
    
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    
    // Comparison
    Eq,
    NotEq,
    Greater,
    Less,
    GreaterEq,
    LessEq,
    
    // Logical
    And,
    Or,
    Not,
    
    // Control flow
    Jump(usize),                // Unconditional jump to instruction index
    JumpIfFalse(usize),         // Jump if top of stack is false
    JumpIfTrue(usize),          // Jump if top of stack is true
    
    // Functions
    Call(String, u8),           // Call function by name with arg count
    CallMethod(String, u8),     // Call method on object
    Return,
    
    // Arrays and Tables
    MakeArray(usize),           // Create array from top N stack items
    MakeTable(usize),           // Create table from top N key-value pairs
    IndexGet,                   // Get array/table element
    IndexSet,                   // Set array/table element
    
    // Stack operations
    Pop,
    Dup,
    
    // Special
    Halt,                       // End of program
}

/// Constant pool entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constant {
    Number(f64),
    String(String),
    Boolean(bool),
}

/// Compiled bytecode program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeProgram {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Constant>,
    pub strings: Vec<String>,
    pub functions: HashMap<String, FunctionMetadata>,
}

/// Function metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    pub name: String,
    pub param_count: usize,
    pub start_index: usize,
    pub end_index: usize,
}

impl BytecodeProgram {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            strings: Vec::new(),
            functions: HashMap::new(),
        }
    }
    
    /// Add a constant to the pool and return its index
    pub fn add_constant(&mut self, constant: Constant) -> usize {
        // Check if constant already exists
        for (i, c) in self.constants.iter().enumerate() {
            let matches = match (c, &constant) {
                (Constant::Number(a), Constant::Number(b)) if (a - b).abs() < f64::EPSILON => true,
                (Constant::String(a), Constant::String(b)) if a == b => true,
                (Constant::Boolean(a), Constant::Boolean(b)) if a == b => true,
                _ => false,
            };
            
            if matches {
                return i;
            }
        }
        
        self.constants.push(constant);
        self.constants.len() - 1
    }
    
    /// Add a string to the string table and return its index
    pub fn add_string(&mut self, s: String) -> usize {
        if let Some(index) = self.strings.iter().position(|str| str == &s) {
            return index;
        }
        
        self.strings.push(s);
        self.strings.len() - 1
    }
    
    /// Add an instruction
    pub fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
    
    /// Get current instruction index (for jump targets)
    pub fn current_index(&self) -> usize {
        self.instructions.len()
    }
    
    /// Patch a jump instruction at the given index
    pub fn patch_jump(&mut self, index: usize, target: usize) {
        match &mut self.instructions[index] {
            Instruction::Jump(ref mut t) |
            Instruction::JumpIfFalse(ref mut t) |
            Instruction::JumpIfTrue(ref mut t) => {
                *t = target;
            }
            _ => panic!("Attempted to patch non-jump instruction"),
        }
    }
}

impl Default for BytecodeProgram {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constant_pool() {
        let mut program = BytecodeProgram::new();
        
        let idx1 = program.add_constant(Constant::Number(42.0));
        let idx2 = program.add_constant(Constant::String("hello".to_string()));
        let idx3 = program.add_constant(Constant::Number(42.0)); // Duplicate
        
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 0); // Should reuse existing constant
        assert_eq!(program.constants.len(), 2);
    }
    
    #[test]
    fn test_string_table() {
        let mut program = BytecodeProgram::new();
        
        let idx1 = program.add_string("test".to_string());
        let idx2 = program.add_string("hello".to_string());
        let idx3 = program.add_string("test".to_string()); // Duplicate
        
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 0); // Should reuse existing string
        assert_eq!(program.strings.len(), 2);
    }
    
    #[test]
    fn test_emit_instructions() {
        let mut program = BytecodeProgram::new();
        
        program.emit(Instruction::LoadConst(0));
        program.emit(Instruction::LoadConst(1));
        program.emit(Instruction::Add);
        
        assert_eq!(program.instructions.len(), 3);
        assert_eq!(program.current_index(), 3);
    }
}
