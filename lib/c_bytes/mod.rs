use crate::evaluator::Value;
use crate::errors::{MintasError, MintasResult, SourceLocation};
use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int, c_double, c_void};

#[cfg(target_os = "windows")]
use libloading::os::windows::Library;
#[cfg(not(target_os = "windows"))]
use libloading::Library;

/// C library manager - loads and calls C functions
pub struct CLibraryManager {
    libraries: HashMap<String, Library>,
}

impl CLibraryManager {
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
        }
    }
    
    /// Load a C shared library (.dll, .so, .dylib)
    pub fn load_library(&mut self, path: &str) -> MintasResult<()> {
        unsafe {
            let lib = Library::new(path)
                .map_err(|e| MintasError::RuntimeError {
                    message: format!("Failed to load library {}: {}", path, e),
                    location: SourceLocation::new(0, 0),
                })?;
            
            self.libraries.insert(path.to_string(), lib);
            Ok(())
        }
    }
    
    /// Call a C function with arguments
    pub fn call_function(&self, lib_path: &str, func_name: &str, args: Vec<Value>) -> MintasResult<Value> {
        let lib = self.libraries.get(lib_path)
            .ok_or_else(|| MintasError::RuntimeError {
                message: format!("Library not loaded: {}", lib_path),
                location: SourceLocation::new(0, 0),
            })?;
        
        // For now, support simple function signatures
        // This is a simplified implementation - full FFI would need type signatures
        
        unsafe {
            // Try to get function as different signatures
            if args.is_empty() {
                // void func()
                let func: libloading::Symbol<unsafe extern fn() -> c_int> = lib.get(func_name.as_bytes())
                    .map_err(|e| MintasError::RuntimeError {
                        message: format!("Function not found: {}: {}", func_name, e),
                        location: SourceLocation::new(0, 0),
                    })?;
                let result = func();
                Ok(Value::Number(result as f64))
            } else if args.len() == 1 {
                // Handle single argument functions
                match &args[0] {
                    Value::Number(n) => {
                        let func: libloading::Symbol<unsafe extern fn(c_double) -> c_double> = lib.get(func_name.as_bytes())
                            .map_err(|e| MintasError::RuntimeError {
                                message: format!("Function not found: {}: {}", func_name, e),
                                location: SourceLocation::new(0, 0),
                            })?;
                        let result = func(*n);
                        Ok(Value::Number(result))
                    }
                    Value::String(s) => {
                        let c_str = CString::new(s.as_str())
                            .map_err(|e| MintasError::RuntimeError {
                                message: format!("Invalid string: {}", e),
                                location: SourceLocation::new(0, 0),
                            })?;
                        let func: libloading::Symbol<unsafe extern fn(*const c_char) -> c_int> = lib.get(func_name.as_bytes())
                            .map_err(|e| MintasError::RuntimeError {
                                message: format!("Function not found: {}: {}", func_name, e),
                                location: SourceLocation::new(0, 0),
                            })?;
                        let result = func(c_str.as_ptr());
                        Ok(Value::Number(result as f64))
                    }
                    _ => Err(MintasError::TypeError {
                        message: "Unsupported argument type for C function".to_string(),
                        location: SourceLocation::new(0, 0),
                    })
                }
            } else if args.len() == 2 {
                // Handle two argument functions (e.g., add(a, b))
                if let (Value::Number(a), Value::Number(b)) = (&args[0], &args[1]) {
                    let func: libloading::Symbol<unsafe extern fn(c_double, c_double) -> c_double> = lib.get(func_name.as_bytes())
                        .map_err(|e| MintasError::RuntimeError {
                            message: format!("Function not found: {}: {}", func_name, e),
                            location: SourceLocation::new(0, 0),
                        })?;
                    let result = func(*a, *b);
                    Ok(Value::Number(result))
                } else {
                    Err(MintasError::TypeError {
                        message: "Expected two numbers for C function".to_string(),
                        location: SourceLocation::new(0, 0),
                    })
                }
            } else {
                Err(MintasError::RuntimeError {
                    message: "C functions with >2 arguments not yet supported".to_string(),
                    location: SourceLocation::new(0, 0),
                })
            }
        }
    }
}

impl Default for CLibraryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Mintas-style API for C interop
pub fn register_c_bytes_functions() -> HashMap<String, fn(&[Value]) -> MintasResult<Value>> {
    let mut functions: HashMap<String, fn(&[Value]) -> MintasResult<Value>> = HashMap::new();
    
    // c_bytes.load(path) - Load C library
    functions.insert("c_bytes.load".to_string(), |args| {
        if args.len() != 1 {
            return Err(MintasError::InvalidArgumentCount {
                function: "c_bytes.load".to_string(),
                expected: 1,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        
        if let Value::String(path) = &args[0] {
            // This would need to be stored in a global manager
            // For now, return success
            Ok(Value::Boolean(true))
        } else {
            Err(MintasError::TypeError {
                message: "c_bytes.load expects a string path".to_string(),
                location: SourceLocation::new(0, 0),
            })
        }
    });
    
    // c_bytes.call(func_name, ...args) - Call C function
    functions.insert("c_bytes.call".to_string(), |args| {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "c_bytes.call".to_string(),
                expected: 1,
                got: 0,
                location: SourceLocation::new(0, 0),
            });
        }
        
        if let Value::String(func_name) = &args[0] {
            // This would call the C function with remaining args
            // For now, return a placeholder
            Ok(Value::Empty)
        } else {
            Err(MintasError::TypeError {
                message: "c_bytes.call expects function name as first argument".to_string(),
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
    fn test_c_library_manager() {
        let mut manager = CLibraryManager::new();
        // Test would require an actual C library
        assert!(manager.libraries.is_empty());
    }
}
