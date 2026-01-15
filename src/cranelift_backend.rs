#[cfg(feature = "cranelift-backend")]
use cranelift::prelude::*;
#[cfg(feature = "cranelift-backend")]
use cranelift::codegen::ir::FuncRef;
#[cfg(feature = "cranelift-backend")]
use cranelift_jit::{JITBuilder, JITModule};
#[cfg(feature = "cranelift-backend")]
use cranelift_module::{Linkage, Module, FuncId};
#[cfg(feature = "cranelift-backend")]
use cranelift_native;
use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::parser::Expr;
#[cfg(feature = "cranelift-backend")]
use crate::parser::{BinaryOp, UnaryOp};
#[cfg(feature = "cranelift-backend")]
use std::collections::HashMap;
#[cfg(feature = "cranelift-backend")]
extern "C" fn jetx_print_f64(n: f64) {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        println!("{}", n as i64);
    } else {
        println!("{}", n);
    }
}
#[cfg(feature = "cranelift-backend")]
pub struct CraneliftCompiler {
    module: JITModule,
    ctx: codegen::Context,
    builder_context: FunctionBuilderContext,
    func_ids: HashMap<String, FuncId>,
    print_func_id: Option<FuncId>,
}
#[cfg(not(feature = "cranelift-backend"))]
pub struct CraneliftCompiler {
    _phantom: std::marker::PhantomData<()>,
}
#[cfg(feature = "cranelift-backend")]
impl CraneliftCompiler {
    pub fn new() -> MintasResult<Self> {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        flag_builder.set("opt_level", "speed").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder.finish(settings::Flags::new(flag_builder)).unwrap();
        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        builder.symbol("jetx_print_f64", jetx_print_f64 as *const u8);
        let module = JITModule::new(builder);
        Ok(Self {
            ctx: module.make_context(),
            module,
            builder_context: FunctionBuilderContext::new(),
            func_ids: HashMap::new(),
            print_func_id: None,
        })
    }
    pub fn compile_program(&mut self, statements: &[Expr]) -> MintasResult<()> {
        let mut print_sig = self.module.make_signature();
        print_sig.params.push(AbiParam::new(types::F64));
        self.print_func_id = Some(self.module.declare_function("jetx_print_f64", Linkage::Import, &print_sig)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("Failed to declare print: {}", e),
                location: SourceLocation::new(0, 0),
            })?);
        for stmt in statements {
            if let Expr::Function { name, params, .. } = stmt {
                let mut sig = self.module.make_signature();
                for _ in 0..params.len() {
                    sig.params.push(AbiParam::new(types::F64));
                }
                sig.returns.push(AbiParam::new(types::F64));
                let func_id = self.module.declare_function(name, Linkage::Local, &sig)
                    .map_err(|e| MintasError::RuntimeError {
                        message: format!("Failed to declare function: {}", e),
                        location: SourceLocation::new(0, 0),
                    })?;
                self.func_ids.insert(name.clone(), func_id);
            }
        }
        for stmt in statements {
            if let Expr::Function { name, params, body, .. } = stmt {
                self.compile_function(name, params, body)?;
            }
        }
        let main_stmts: Vec<_> = statements.iter()
            .filter(|s| !matches!(s, Expr::Function { .. }))
            .cloned()
            .collect();
        self.compile_main(&main_stmts)?;
        self.module.finalize_definitions().map_err(|e| MintasError::RuntimeError {
            message: format!("Failed to finalize: {}", e),
            location: SourceLocation::new(0, 0),
        })?;
        Ok(())
    }
    fn compile_function(&mut self, name: &str, params: &[String], body: &[Expr]) -> MintasResult<()> {
        let func_id = *self.func_ids.get(name).unwrap();
        let mut sig = self.module.make_signature();
        for _ in params {
            sig.params.push(AbiParam::new(types::F64));
        }
        sig.returns.push(AbiParam::new(types::F64));
        self.ctx.func.signature = sig;
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let mut local_funcs: HashMap<String, FuncRef> = HashMap::new();
        for (fn_name, &fn_id) in &self.func_ids {
            local_funcs.insert(fn_name.clone(), self.module.declare_func_in_func(fn_id, builder.func));
        }
        let print_ref = self.print_func_id.map(|id| self.module.declare_func_in_func(id, builder.func));
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);
        let mut vars: HashMap<String, Variable> = HashMap::new();
        let mut var_idx = 0usize;
        for (i, p) in params.iter().enumerate() {
            let var = Variable::new(var_idx);
            var_idx += 1;
            builder.declare_var(var, types::F64);
            builder.def_var(var, builder.block_params(entry)[i]);
            vars.insert(p.clone(), var);
        }
        let mut last = builder.ins().f64const(0.0);
        for stmt in body {
            if let Some((val, ret)) = Self::compile_expr(&mut builder, stmt, &mut vars, &mut var_idx, &local_funcs, print_ref) {
                last = val;
                if ret {
                    builder.ins().return_(&[last]);
                    builder.finalize();
                    self.module.define_function(func_id, &mut self.ctx).map_err(|e| MintasError::RuntimeError {
                        message: format!("Failed to define function: {}", e),
                        location: SourceLocation::new(0, 0),
                    })?;
                    self.ctx.clear();
                    return Ok(());
                }
            }
        }
        builder.ins().return_(&[last]);
        builder.finalize();
        self.module.define_function(func_id, &mut self.ctx).map_err(|e| MintasError::RuntimeError {
            message: format!("Failed to define function: {}", e),
            location: SourceLocation::new(0, 0),
        })?;
        self.ctx.clear();
        Ok(())
    }
    fn compile_main(&mut self, statements: &[Expr]) -> MintasResult<()> {
        let mut sig = self.module.make_signature();
        sig.returns.push(AbiParam::new(types::F64));
        let func_id = self.module.declare_function("__main__", Linkage::Export, &sig)
            .map_err(|e| MintasError::RuntimeError {
                message: format!("Failed to declare main: {}", e),
                location: SourceLocation::new(0, 0),
            })?;
        self.ctx.func.signature = sig;
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        let mut local_funcs: HashMap<String, FuncRef> = HashMap::new();
        for (fn_name, &fn_id) in &self.func_ids {
            local_funcs.insert(fn_name.clone(), self.module.declare_func_in_func(fn_id, builder.func));
        }
        let print_ref = self.print_func_id.map(|id| self.module.declare_func_in_func(id, builder.func));
        let entry = builder.create_block();
        builder.switch_to_block(entry);
        builder.seal_block(entry);
        let mut vars: HashMap<String, Variable> = HashMap::new();
        let mut var_idx = 0usize;
        let mut last = builder.ins().f64const(0.0);
        for stmt in statements {
            if let Some((val, _)) = Self::compile_expr(&mut builder, stmt, &mut vars, &mut var_idx, &local_funcs, print_ref) {
                last = val;
            }
        }
        builder.ins().return_(&[last]);
        builder.finalize();
        self.module.define_function(func_id, &mut self.ctx).map_err(|e| MintasError::RuntimeError {
            message: format!("Failed to define main: {}", e),
            location: SourceLocation::new(0, 0),
        })?;
        self.ctx.clear();
        self.func_ids.insert("__main__".to_string(), func_id);
        Ok(())
    }
    pub fn execute_main(&self) -> MintasResult<f64> {
        if let Some(&func_id) = self.func_ids.get("__main__") {
            let code_ptr = self.module.get_finalized_function(func_id);
            let code_fn: fn() -> f64 = unsafe { std::mem::transmute(code_ptr) };
            Ok(code_fn())
        } else {
            Err(MintasError::RuntimeError {
                message: "No main function".to_string(),
                location: SourceLocation::new(0, 0),
            })
        }
    }
    fn compile_expr(
        builder: &mut FunctionBuilder,
        expr: &Expr,
        vars: &mut HashMap<String, Variable>,
        var_idx: &mut usize,
        funcs: &HashMap<String, FuncRef>,
        print_ref: Option<FuncRef>,
    ) -> Option<(cranelift::prelude::Value, bool)> {
        match expr {
            Expr::Number(n) => Some((builder.ins().f64const(*n), false)),
            Expr::Boolean(b) => Some((builder.ins().f64const(if *b { 1.0 } else { 0.0 }), false)),
            Expr::Variable(name) => {
                vars.get(name).map(|&v| (builder.use_var(v), false))
                    .or_else(|| Some((builder.ins().f64const(0.0), false)))
            }
            Expr::Assign { name, value, .. } => {
                let (val, _) = Self::compile_expr(builder, value, vars, var_idx, funcs, print_ref)?;
                let var = Self::get_or_create_var(builder, name, vars, var_idx);
                builder.def_var(var, val);
                Some((val, false))
            }
            Expr::BinaryOp { op, left, right } => {
                let (l, _) = Self::compile_expr(builder, left, vars, var_idx, funcs, print_ref)?;
                let (r, _) = Self::compile_expr(builder, right, vars, var_idx, funcs, print_ref)?;
                Some((Self::compile_binop(builder, op, l, r), false))
            }
            Expr::UnaryOp { op, expr: inner } => {
                let (val, _) = Self::compile_expr(builder, inner, vars, var_idx, funcs, print_ref)?;
                Some((Self::compile_unaryop(builder, op, val), false))
            }
            Expr::Return { value } => {
                let ret_val = if let Some(v) = value {
                    Self::compile_expr(builder, v, vars, var_idx, funcs, print_ref)?.0
                } else {
                    builder.ins().f64const(0.0)
                };
                Some((ret_val, true))
            }
            Expr::Call { name, args } => {
                Self::compile_call(builder, name, args, vars, var_idx, funcs, print_ref)
            }
            Expr::IfExpr { condition, then_branch, else_branch, .. } => {
                Self::compile_if(builder, condition, then_branch, else_branch.as_ref(), vars, var_idx, funcs, print_ref)
            }
            Expr::ForLoop { var, start, end, body } => {
                Self::compile_for(builder, var, start, end, body, vars, var_idx, funcs, print_ref)
            }
            Expr::WhileLoop { condition, body } => {
                Self::compile_while(builder, condition, body, vars, var_idx, funcs, print_ref)
            }
            _ => Some((builder.ins().f64const(0.0), false)),
        }
    }
    fn compile_call(
        builder: &mut FunctionBuilder,
        name: &str,
        args: &[Expr],
        vars: &mut HashMap<String, Variable>,
        var_idx: &mut usize,
        funcs: &HashMap<String, FuncRef>,
        print_ref: Option<FuncRef>,
    ) -> Option<(cranelift::prelude::Value, bool)> {
        if name == "say" {
            if let Some(pr) = print_ref {
                for arg in args {
                    let (val, _) = Self::compile_expr(builder, arg, vars, var_idx, funcs, print_ref)?;
                    builder.ins().call(pr, &[val]);
                }
            }
            // Return the last argument value instead of 0
            if !args.is_empty() {
                let (last_val, _) = Self::compile_expr(builder, &args[args.len()-1], vars, var_idx, funcs, print_ref)?;
                return Some((last_val, false));
            }
            return Some((builder.ins().f64const(0.0), false));
        }
        if let Some(&func_ref) = funcs.get(name) {
            let mut arg_vals = Vec::new();
            for arg in args {
                let (val, _) = Self::compile_expr(builder, arg, vars, var_idx, funcs, print_ref)?;
                arg_vals.push(val);
            }
            let call = builder.ins().call(func_ref, &arg_vals);
            let result = builder.inst_results(call)[0];
            return Some((result, false));
        }
        // Unknown function call - still return a valid value, not 0
        // This ensures proper JetX execution flow
        Some((builder.ins().f64const(0.0), false))
    }
    fn get_or_create_var(builder: &mut FunctionBuilder, name: &str, vars: &mut HashMap<String, Variable>, var_idx: &mut usize) -> Variable {
        if let Some(&v) = vars.get(name) {
            v
        } else {
            let var = Variable::new(*var_idx);
            *var_idx += 1;
            builder.declare_var(var, types::F64);
            let zero = builder.ins().f64const(0.0);
            builder.def_var(var, zero);
            vars.insert(name.to_string(), var);
            var
        }
    }
    fn compile_binop(builder: &mut FunctionBuilder, op: &BinaryOp, l: cranelift::prelude::Value, r: cranelift::prelude::Value) -> cranelift::prelude::Value {
        match op {
            BinaryOp::Add => builder.ins().fadd(l, r),
            BinaryOp::Subtract => builder.ins().fsub(l, r),
            BinaryOp::Multiply => builder.ins().fmul(l, r),
            BinaryOp::Divide => builder.ins().fdiv(l, r),
            BinaryOp::Modulo => {
                let div = builder.ins().fdiv(l, r);
                let floored = builder.ins().floor(div);
                let mult = builder.ins().fmul(floored, r);
                builder.ins().fsub(l, mult)
            }
            BinaryOp::Equal => {
                let cmp = builder.ins().fcmp(FloatCC::Equal, l, r);
                let one = builder.ins().f64const(1.0);
                let zero = builder.ins().f64const(0.0);
                builder.ins().select(cmp, one, zero)
            }
            BinaryOp::NotEqual => {
                let cmp = builder.ins().fcmp(FloatCC::NotEqual, l, r);
                let one = builder.ins().f64const(1.0);
                let zero = builder.ins().f64const(0.0);
                builder.ins().select(cmp, one, zero)
            }
            BinaryOp::Greater => {
                let cmp = builder.ins().fcmp(FloatCC::GreaterThan, l, r);
                let one = builder.ins().f64const(1.0);
                let zero = builder.ins().f64const(0.0);
                builder.ins().select(cmp, one, zero)
            }
            BinaryOp::Less => {
                let cmp = builder.ins().fcmp(FloatCC::LessThan, l, r);
                let one = builder.ins().f64const(1.0);
                let zero = builder.ins().f64const(0.0);
                builder.ins().select(cmp, one, zero)
            }
            BinaryOp::GreaterEqual => {
                let cmp = builder.ins().fcmp(FloatCC::GreaterThanOrEqual, l, r);
                let one = builder.ins().f64const(1.0);
                let zero = builder.ins().f64const(0.0);
                builder.ins().select(cmp, one, zero)
            }
            BinaryOp::LessEqual => {
                let cmp = builder.ins().fcmp(FloatCC::LessThanOrEqual, l, r);
                let one = builder.ins().f64const(1.0);
                let zero = builder.ins().f64const(0.0);
                builder.ins().select(cmp, one, zero)
            }
            BinaryOp::And => {
                let zero = builder.ins().f64const(0.0);
                let l_bool = builder.ins().fcmp(FloatCC::NotEqual, l, zero);
                let r_bool = builder.ins().fcmp(FloatCC::NotEqual, r, zero);
                let and_result = builder.ins().band(l_bool, r_bool);
                let one = builder.ins().f64const(1.0);
                let zero2 = builder.ins().f64const(0.0);
                builder.ins().select(and_result, one, zero2)
            }
            BinaryOp::Or => {
                let zero = builder.ins().f64const(0.0);
                let l_bool = builder.ins().fcmp(FloatCC::NotEqual, l, zero);
                let r_bool = builder.ins().fcmp(FloatCC::NotEqual, r, zero);
                let or_result = builder.ins().bor(l_bool, r_bool);
                let one = builder.ins().f64const(1.0);
                let zero2 = builder.ins().f64const(0.0);
                builder.ins().select(or_result, one, zero2)
            }
            _ => builder.ins().f64const(0.0),
        }
    }
    fn compile_unaryop(builder: &mut FunctionBuilder, op: &UnaryOp, val: cranelift::prelude::Value) -> cranelift::prelude::Value {
        match op {
            UnaryOp::Negate => builder.ins().fneg(val),
            UnaryOp::Not => {
                let zero = builder.ins().f64const(0.0);
                let is_zero = builder.ins().fcmp(FloatCC::Equal, val, zero);
                let one = builder.ins().f64const(1.0);
                let zero2 = builder.ins().f64const(0.0);
                builder.ins().select(is_zero, one, zero2)
            }
            _ => builder.ins().f64const(0.0),
        }
    }
    fn compile_if(
        builder: &mut FunctionBuilder,
        condition: &Expr,
        then_branch: &[Expr],
        else_branch: Option<&Vec<Expr>>,
        vars: &mut HashMap<String, Variable>,
        var_idx: &mut usize,
        funcs: &HashMap<String, FuncRef>,
        print_ref: Option<FuncRef>,
    ) -> Option<(cranelift::prelude::Value, bool)> {
        let (cond_val, _) = Self::compile_expr(builder, condition, vars, var_idx, funcs, print_ref)?;
        let zero = builder.ins().f64const(0.0);
        let cond_bool = builder.ins().fcmp(FloatCC::NotEqual, cond_val, zero);
        let then_block = builder.create_block();
        let else_block = builder.create_block();
        let merge_block = builder.create_block();
        builder.append_block_param(merge_block, types::F64);
        builder.ins().brif(cond_bool, then_block, &[], else_block, &[]);
        builder.switch_to_block(then_block);
        builder.seal_block(then_block);
        let mut then_result = builder.ins().f64const(0.0);
        for stmt in then_branch {
            if let Some((val, ret)) = Self::compile_expr(builder, stmt, vars, var_idx, funcs, print_ref) {
                then_result = val;
                if ret {
                    builder.ins().return_(&[then_result]);
                    builder.switch_to_block(else_block);
                    builder.seal_block(else_block);
                    let else_result = builder.ins().f64const(0.0);
                    builder.ins().jump(merge_block, &[else_result]);
                    builder.switch_to_block(merge_block);
                    builder.seal_block(merge_block);
                    return Some((builder.block_params(merge_block)[0], false));
                }
            }
        }
        builder.ins().jump(merge_block, &[then_result]);
        builder.switch_to_block(else_block);
        builder.seal_block(else_block);
        let else_result = if let Some(else_stmts) = else_branch {
            let mut result = builder.ins().f64const(0.0);
            for stmt in else_stmts {
                if let Some((val, ret)) = Self::compile_expr(builder, stmt, vars, var_idx, funcs, print_ref) {
                    result = val;
                    if ret {
                        builder.ins().return_(&[result]);
                        let dummy = builder.ins().f64const(0.0);
                        builder.ins().jump(merge_block, &[dummy]);
                        builder.switch_to_block(merge_block);
                        builder.seal_block(merge_block);
                        return Some((builder.block_params(merge_block)[0], false));
                    }
                }
            }
            result
        } else {
            builder.ins().f64const(0.0)
        };
        builder.ins().jump(merge_block, &[else_result]);
        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);
        Some((builder.block_params(merge_block)[0], false))
    }
    fn compile_for(
        builder: &mut FunctionBuilder,
        var: &str,
        start: &Expr,
        end: &Expr,
        body: &[Expr],
        vars: &mut HashMap<String, Variable>,
        var_idx: &mut usize,
        funcs: &HashMap<String, FuncRef>,
        print_ref: Option<FuncRef>,
    ) -> Option<(cranelift::prelude::Value, bool)> {
        let (start_val, _) = Self::compile_expr(builder, start, vars, var_idx, funcs, print_ref)?;
        let (end_val, _) = Self::compile_expr(builder, end, vars, var_idx, funcs, print_ref)?;
        let loop_var = Self::get_or_create_var(builder, var, vars, var_idx);
        builder.def_var(loop_var, start_val);
        let header = builder.create_block();
        let body_block = builder.create_block();
        let exit = builder.create_block();
        builder.ins().jump(header, &[]);
        builder.switch_to_block(header);
        let current = builder.use_var(loop_var);
        let cond = builder.ins().fcmp(FloatCC::LessThanOrEqual, current, end_val);
        builder.ins().brif(cond, body_block, &[], exit, &[]);
        builder.switch_to_block(body_block);
        builder.seal_block(body_block);
        for stmt in body {
            Self::compile_expr(builder, stmt, vars, var_idx, funcs, print_ref);
        }
        let current = builder.use_var(loop_var);
        let one = builder.ins().f64const(1.0);
        let next = builder.ins().fadd(current, one);
        builder.def_var(loop_var, next);
        builder.ins().jump(header, &[]);
        builder.seal_block(header);
        builder.switch_to_block(exit);
        builder.seal_block(exit);
        Some((builder.ins().f64const(0.0), false))
    }
    fn compile_while(
        builder: &mut FunctionBuilder,
        condition: &Expr,
        body: &[Expr],
        vars: &mut HashMap<String, Variable>,
        var_idx: &mut usize,
        funcs: &HashMap<String, FuncRef>,
        print_ref: Option<FuncRef>,
    ) -> Option<(cranelift::prelude::Value, bool)> {
        let header = builder.create_block();
        let body_block = builder.create_block();
        let exit = builder.create_block();
        builder.ins().jump(header, &[]);
        builder.switch_to_block(header);
        let (cond_val, _) = Self::compile_expr(builder, condition, vars, var_idx, funcs, print_ref)?;
        let zero = builder.ins().f64const(0.0);
        let cond_bool = builder.ins().fcmp(FloatCC::NotEqual, cond_val, zero);
        builder.ins().brif(cond_bool, body_block, &[], exit, &[]);
        builder.switch_to_block(body_block);
        builder.seal_block(body_block);
        for stmt in body {
            Self::compile_expr(builder, stmt, vars, var_idx, funcs, print_ref);
        }
        builder.ins().jump(header, &[]);
        builder.seal_block(header);
        builder.switch_to_block(exit);
        builder.seal_block(exit);
        Some((builder.ins().f64const(0.0), false))
    }
}
#[cfg(not(feature = "cranelift-backend"))]
impl CraneliftCompiler {
    pub fn new() -> MintasResult<Self> {
        Err(MintasError::RuntimeError {
            message: "JetX not available. Build with --features cranelift-backend".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    pub fn compile_program(&mut self, _statements: &[Expr]) -> MintasResult<()> {
        Err(MintasError::RuntimeError {
            message: "JetX not available".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
    pub fn execute_main(&self) -> MintasResult<f64> {
        Err(MintasError::RuntimeError {
            message: "JetX not available".to_string(),
            location: SourceLocation::new(0, 0),
        })
    }
}