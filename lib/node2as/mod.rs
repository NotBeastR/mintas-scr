use crate::evaluator::Value;
use crate::errors::{MintasError, MintasResult, SourceLocation};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::io::{Write, BufRead, BufReader};

/// Node.js runtime manager - executes Node.js code from Mintas
pub struct NodeJSRuntime {
    modules: HashMap<String, String>,
}

impl NodeJSRuntime {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }
    
    /// Require a Node.js module (like require('fs'))
    pub fn require_module(&mut self, module_name: &str) -> MintasResult<String> {
        // Check if Node.js is available
        let check = Command::new("node")
            .arg("--version")
            .output();
        
        if check.is_err() {
            return Err(MintasError::RuntimeError {
                message: "Node.js not found. Please install Node.js to use node2as module.".to_string(),
                location: SourceLocation::new(0, 0),
            });
        }
        
        // Store module reference
        let module_id = format!("module_{}", self.modules.len());
        self.modules.insert(module_id.clone(), module_name.to_string());
        
        Ok(module_id)
    }
    
    /// Call a Node.js function
    pub fn call_function(&self, module_id: &str, function_name: &str, args: Vec<Value>) -> MintasResult<Value> {
        let module_name = self.modules.get(module_id)
            .ok_or_else(|| MintasError::RuntimeError {
                message: format!("Module not loaded: {}", module_id),
                location: SourceLocation::new(0, 0),
            })?;
        
        // Build Node.js code to execute
        let args_json = self.values_to_json(&args)?;
        let node_code = format!(
            r#"
            const module = require('{}');
            const result = module.{}({});
            console.log(JSON.stringify(result));
            "#,
            module_name,
            function_name,
            args_json
        );
        
        // Execute Node.js code
        let output = Command::new("node")
            .arg("-e")
            .arg(&node_code)
            .output()
            .map_err(|e| MintasError::RuntimeError {
                message: format!("Failed to execute Node.js: {}", e),
                location: SourceLocation::new(0, 0),
            })?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(MintasError::RuntimeError {
                message: format!("Node.js error: {}", error),
                location: SourceLocation::new(0, 0),
            });
        }
        
        // Parse result
        let result_str = String::from_utf8_lossy(&output.stdout);
        self.json_to_value(&result_str.trim())
    }
    
    /// Execute raw Node.js code
    pub fn execute_code(&self, code: &str) -> MintasResult<Value> {
        let output = Command::new("node")
            .arg("-e")
            .arg(code)
            .output()
            .map_err(|e| MintasError::RuntimeError {
                message: format!("Failed to execute Node.js: {}", e),
                location: SourceLocation::new(0, 0),
            })?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(MintasError::RuntimeError {
                message: format!("Node.js error: {}", error),
                location: SourceLocation::new(0, 0),
            });
        }
        
        let result = String::from_utf8_lossy(&output.stdout);
        Ok(Value::String(result.trim().to_string()))
    }
    
    fn values_to_json(&self, values: &[Value]) -> MintasResult<String> {
        let json_values: Vec<String> = values.iter().map(|v| {
            match v {
                Value::Number(n) => n.to_string(),
                Value::String(s) => format!("\"{}\"", s.replace("\"", "\\\"")),
                Value::Boolean(b) => b.to_string(),
                Value::Empty => "null".to_string(),
                _ => "null".to_string(),
            }
        }).collect();
        
        Ok(json_values.join(", "))
    }
    
    fn json_to_value(&self, json: &str) -> MintasResult<Value> {
        // Simple JSON parsing (could use serde_json for full support)
        if json == "null" || json == "undefined" {
            Ok(Value::Empty)
        } else if json == "true" {
            Ok(Value::Boolean(true))
        } else if json == "false" {
            Ok(Value::Boolean(false))
        } else if json.starts_with('"') && json.ends_with('"') {
            Ok(Value::String(json[1..json.len()-1].to_string()))
        } else if let Ok(n) = json.parse::<f64>() {
            Ok(Value::Number(n))
        } else {
            // Return as string for complex objects
            Ok(Value::String(json.to_string()))
        }
    }
}

impl Default for NodeJSRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Mintas-style API for Node.js interop
pub fn register_node2as_functions() -> HashMap<String, fn(&[Value]) -> MintasResult<Value>> {
    let mut functions: HashMap<String, fn(&[Value]) -> MintasResult<Value>> = HashMap::new();
    
    // node2as.require(module_name) - Require Node.js module
    functions.insert("node2as.require".to_string(), |args| {
        if args.len() != 1 {
            return Err(MintasError::InvalidArgumentCount {
                function: "node2as.require".to_string(),
                expected: 1,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        
        if let Value::String(module_name) = &args[0] {
            // This would need to be stored in a global runtime
            // For now, return module name as ID
            Ok(Value::String(module_name.clone()))
        } else {
            Err(MintasError::TypeError {
                message: "node2as.require expects a string module name".to_string(),
                location: SourceLocation::new(0, 0),
            })
        }
    });
    
    // node2as.call(module, function, ...args) - Call Node.js function
    functions.insert("node2as.call".to_string(), |args| {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "node2as.call".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        
        // This would call the Node.js function
        // For now, return a placeholder
        Ok(Value::Empty)
    });
    
    // node2as.eval(code) - Execute raw Node.js code
    functions.insert("node2as.eval".to_string(), |args| {
        if args.len() != 1 {
            return Err(MintasError::InvalidArgumentCount {
                function: "node2as.eval".to_string(),
                expected: 1,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        
        if let Value::String(code) = &args[0] {
            let runtime = NodeJSRuntime::new();
            runtime.execute_code(code)
        } else {
            Err(MintasError::TypeError {
                message: "node2as.eval expects a string code".to_string(),
                location: SourceLocation::new(0, 0),
            })
        }
    });
    
    functions
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nodejs_runtime() {
        let mut runtime = NodeJSRuntime::new();
        // Test would require Node.js installed
        assert!(runtime.modules.is_empty());
    }
}
