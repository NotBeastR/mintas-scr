use crate::evaluator::Value;
use crate::errors::{MintasError, MintasResult, SourceLocation};
use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int, c_double, c_void, c_float, c_uint, c_long, c_ulong};
use std::ptr;

#[cfg(target_os = "windows")]
use libloading::os::windows::Library;
#[cfg(not(target_os = "windows"))]
use libloading::Library;

/// Represents a Rust function signature for FFI
#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub name: String,
    pub param_types: Vec<FFIType>,
    pub return_type: FFIType,
}

/// FFI-compatible types
#[derive(Clone, Debug, PartialEq)]
pub enum FFIType {
    Void,
    I32,      // int (c_int)
    I64,      // long (c_long)
    U32,      // unsigned int (c_uint)
    U64,      // unsigned long (c_ulong)
    F32,      // float (c_float)
    F64,      // double (c_double)
    String,   // const char*
    Pointer,  // void*
    Bool,     // bool / char
}

impl FFIType {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "void" => Some(FFIType::Void),
            "i32" | "int" | "c_int" => Some(FFIType::I32),
            "i64" | "long" | "c_long" => Some(FFIType::I64),
            "u32" | "uint" | "c_uint" => Some(FFIType::U32),
            "u64" | "ulong" | "c_ulong" => Some(FFIType::U64),
            "f32" | "float" | "c_float" => Some(FFIType::F32),
            "f64" | "double" | "c_double" => Some(FFIType::F64),
            "string" | "str" | "char*" | "const char*" => Some(FFIType::String),
            "ptr" | "void*" | "pointer" => Some(FFIType::Pointer),
            "bool" => Some(FFIType::Bool),
            _ => None,
        }
    }
}

/// C library manager - loads and calls C and Rust functions via FFI
pub struct CLibraryManager {
    libraries: HashMap<String, Library>,
    signatures: HashMap<String, FunctionSignature>,
}

impl CLibraryManager {
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
            signatures: HashMap::new(),
        }
    }
    
    /// Load a C/Rust shared library (.dll, .so, .dylib)
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
    
    /// Register a function signature for type-safe FFI
    pub fn register_signature(&mut self, signature: FunctionSignature) {
        self.signatures.insert(signature.name.clone(), signature);
    }
    
    /// Call a C/Rust function with type conversion
    pub fn call_function(&self, lib_path: &str, func_name: &str, args: Vec<Value>) -> MintasResult<Value> {
        let lib = self.libraries.get(lib_path)
            .ok_or_else(|| MintasError::RuntimeError {
                message: format!("Library not loaded: {}", lib_path),
                location: SourceLocation::new(0, 0),
            })?;
        
        unsafe {
            // Check if we have a registered signature for type safety
            if let Some(sig) = self.signatures.get(func_name) {
                if args.len() != sig.param_types.len() {
                    return Err(MintasError::InvalidArgumentCount {
                        function: func_name.to_string(),
                        expected: sig.param_types.len(),
                        got: args.len(),
                        location: SourceLocation::new(0, 0),
                    });
                }
                
                // Call with type-safe conversion
                return self.call_with_signature(lib, func_name, &args, sig);
            }
            
            // Fallback: Try to auto-detect function signature from arguments
            self.call_auto_detect(lib, func_name, &args)
        }
    }
    
    /// Call function with a registered signature
    unsafe fn call_with_signature(&self, lib: &Library, func_name: &str, args: &[Value], sig: &FunctionSignature) -> MintasResult<Value> {
        match (sig.param_types.len(), &sig.return_type) {
            (0, FFIType::I32) => {
                let func: libloading::Symbol<unsafe extern "C" fn() -> c_int> = lib.get(func_name.as_bytes())?;
                Ok(Value::Number(func() as f64))
            }
            (0, FFIType::F64) => {
                let func: libloading::Symbol<unsafe extern "C" fn() -> c_double> = lib.get(func_name.as_bytes())?;
                Ok(Value::Number(func()))
            }
            (1, FFIType::F64) if sig.param_types[0] == FFIType::F64 => {
                let n = self.value_to_f64(&args[0])?;
                let func: libloading::Symbol<unsafe extern "C" fn(c_double) -> c_double> = lib.get(func_name.as_bytes())?;
                Ok(Value::Number(func(n)))
            }
            (1, FFIType::I32) if sig.param_types[0] == FFIType::I32 => {
                let n = self.value_to_i32(&args[0])?;
                let func: libloading::Symbol<unsafe extern "C" fn(c_int) -> c_int> = lib.get(func_name.as_bytes())?;
                Ok(Value::Number(func(n) as f64))
            }
            (2, FFIType::F64) if sig.param_types == vec![FFIType::F64, FFIType::F64] => {
                let a = self.value_to_f64(&args[0])?;
                let b = self.value_to_f64(&args[1])?;
                let func: libloading::Symbol<unsafe extern "C" fn(c_double, c_double) -> c_double> = lib.get(func_name.as_bytes())?;
                Ok(Value::Number(func(a, b)))
            }
            (3, FFIType::F64) if sig.param_types == vec![FFIType::F64, FFIType::F64, FFIType::F64] => {
                let a = self.value_to_f64(&args[0])?;
                let b = self.value_to_f64(&args[1])?;
                let c = self.value_to_f64(&args[2])?;
                let func: libloading::Symbol<unsafe extern "C" fn(c_double, c_double, c_double) -> c_double> = lib.get(func_name.as_bytes())?;
                Ok(Value::Number(func(a, b, c)))
            }
            (1, FFIType::String) if sig.param_types[0] == FFIType::String => {
                let s = self.value_to_string(&args[0])?;
                let c_str = CString::new(s)?;
                let func: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> c_int> = lib.get(func_name.as_bytes())?;
                Ok(Value::Number(func(c_str.as_ptr()) as f64))
            }
            _ => Err(MintasError::RuntimeError {
                message: format!("Unsupported function signature for: {}", func_name),
                location: SourceLocation::new(0, 0),
            })
        }
    }
    
    /// Auto-detect and call function based on argument types
    unsafe fn call_auto_detect(&self, lib: &Library, func_name: &str, args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            // Try void()
            if let Ok(func) = lib.get::<libloading::Symbol<unsafe extern "C" fn() -> c_int>>(func_name.as_bytes()) {
                return Ok(Value::Number(func() as f64));
            }
        } else if args.len() == 1 {
            match &args[0] {
                Value::Number(n) => {
                    if let Ok(func) = lib.get::<libloading::Symbol<unsafe extern "C" fn(c_double) -> c_double>>(func_name.as_bytes()) {
                        return Ok(Value::Number(func(*n)));
                    }
                }
                Value::String(s) => {
                    let c_str = CString::new(s.as_str())?;
                    if let Ok(func) = lib.get::<libloading::Symbol<unsafe extern "C" fn(*const c_char) -> c_int>>(func_name.as_bytes()) {
                        return Ok(Value::Number(func(c_str.as_ptr()) as f64));
                    }
                }
                _ => {}
            }
        } else if args.len() == 2 {
            if let (Value::Number(a), Value::Number(b)) = (&args[0], &args[1]) {
                if let Ok(func) = lib.get::<libloading::Symbol<unsafe extern "C" fn(c_double, c_double) -> c_double>>(func_name.as_bytes()) {
                    return Ok(Value::Number(func(*a, *b)));
                }
            }
        }
        
        Err(MintasError::RuntimeError {
            message: format!("Could not find matching signature for: {}", func_name),
            location: SourceLocation::new(0, 0),
        })
    }
    
    // Helper methods for value conversion
    fn value_to_f64(&self, val: &Value) -> MintasResult<c_double> {
        match val {
            Value::Number(n) => Ok(*n),
            Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err(MintasError::TypeError {
                message: "Cannot convert value to f64".to_string(),
                location: SourceLocation::new(0, 0),
            })
        }
    }
    
    fn value_to_i32(&self, val: &Value) -> MintasResult<c_int> {
        match val {
            Value::Number(n) => Ok(*n as c_int),
            Value::Boolean(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err(MintasError::TypeError {
                message: "Cannot convert value to i32".to_string(),
                location: SourceLocation::new(0, 0),
            })
        }
    }
    
    fn value_to_string(&self, val: &Value) -> MintasResult<String> {
        match val {
            Value::String(s) => Ok(s.clone()),
            Value::Number(n) => Ok(n.to_string()),
            Value::Boolean(b) => Ok(b.to_string()),
            _ => Err(MintasError::TypeError {
                message: "Cannot convert value to string".to_string(),
                location: SourceLocation::new(0, 0),
            })
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
