use crate::ast::*;
use crate::lexer::Token;
use cranelift::prelude::*;
use cranelift_codegen::ir::{StackSlot, StackSlotData, StackSlotKind, UserFuncName};
use cranelift_codegen::settings;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_native;
use std::collections::HashMap;
use std::mem;

/// Core JIT compiler for generating and executing machine code from AST.
pub struct JITCompiler {
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    module: JITModule,
    print_func_id: FuncId,
}

impl JITCompiler {
    /// Creates a new JIT compiler instance.
    pub fn new() -> Self {
        let isa_builder = cranelift_native::builder().expect("Failed to create ISA builder");
        let isa = isa_builder
        .finish(settings::Flags::new(settings::builder()))
        .expect("Failed to build ISA");

        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        // Register the print_i64 symbol
        let print_i64_ptr = super::print_i64 as *const u8;
        builder.symbol("print_i64", print_i64_ptr);

        let mut module = JITModule::new(builder);

        // Declare the external `print_i64` function.
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I64));
        let print_func_id = module
        .declare_function("print_i64", Linkage::Import, &sig)
        .expect("Failed to declare print_i64");

        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
            print_func_id,
        }
    }

    /// Compiles the program to machine code and executes it, returning the result.
    pub fn compile_and_run(&mut self, prog: &Program) -> i64 {
        let mut sig = self.module.make_signature();
        sig.returns.push(AbiParam::new(types::I64));

        let func_id = self
        .module
        .declare_anonymous_function(&sig)
        .expect("Failed to declare anonymous function");

        self.ctx.func.signature = sig;
        self.ctx.func.name = UserFuncName::user(0, func_id.as_u32()).into();

        let mut variables: HashMap<String, StackSlot> = HashMap::new();

        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
            let entry_block = builder.create_block();
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            // Generate Cranelift IR for each statement.
            let mut last_value = None;
            for stmt in &prog.stmts {
                last_value = Self::codegen_stmt(
                    &mut builder,
                    &mut variables,
                    &mut self.module,
                    self.print_func_id,
                    stmt,
                );
            }

            // Return the last value or 0 if none.
            let return_value = last_value
            .unwrap_or_else(|| builder.ins().iconst(types::I64, 0));
            builder.ins().return_(&[return_value]);

            builder.finalize();
        }

        self.module
        .define_function(func_id, &mut self.ctx)
        .expect("Failed to define function");

        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions().expect("Failed to finalize definitions");

        let func_ptr = self.module.get_finalized_function(func_id);
        let compiled_fn: extern "C" fn() -> i64 = unsafe { mem::transmute(func_ptr) };
        compiled_fn()
    }

    fn codegen_stmt(
        builder: &mut FunctionBuilder,
        variables: &mut HashMap<String, StackSlot>,
        module: &mut JITModule,
        print_func_id: FuncId,
        stmt: &Stmt,
    ) -> Option<Value> {
        match stmt {
            Stmt::ExprStmt(expr) => Some(Self::codegen_expr(
                builder,
                variables,
                module,
                print_func_id,
                expr,
            )),
            Stmt::LetDecl { name, value } => {
                let initial_value =
                Self::codegen_expr(builder, variables, module, print_func_id, value);
                let stack_slot = builder
                .create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8));
                builder.ins().stack_store(initial_value, stack_slot, 0);
                variables.insert(name.clone(), stack_slot);
                Some(initial_value)
            }
        }
    }

    fn codegen_expr(
        builder: &mut FunctionBuilder,
        variables: &mut HashMap<String, StackSlot>,
        module: &mut JITModule,
        print_func_id: FuncId,
        expr: &Expr,
    ) -> Value {
        match expr {
            Expr::Number(n) => builder.ins().iconst(types::I64, *n),
            Expr::Bool(b) => builder.ins().iconst(types::I64, if *b { 1 } else { 0 }),
            Expr::Variable(name) => {
                if let Some(stack_slot) = variables.get(name) {
                    builder.ins().stack_load(types::I64, *stack_slot, 0)
                } else {
                    panic!("Undefined variable: {}", name);
                }
            }
            Expr::FunctionCall { name, args } => {
                if name == "print" {
                    let arg_values: Vec<Value> = args
                    .iter()
                    .map(|arg| Self::codegen_expr(builder, variables, module, print_func_id, arg))
                    .collect();
                    if arg_values.len() != 1 {
                        panic!("'print' expects exactly one argument");
                    }

                    let callee = module.declare_func_in_func(print_func_id, builder.func);

                    let call_inst = builder.ins().call(callee, &arg_values);
                    builder.inst_results(call_inst)[0]
                } else {
                    panic!("Unknown function: {}", name);
                }
            }
            Expr::BinaryOp { op, left, right } => {
                let l = Self::codegen_expr(builder, variables, module, print_func_id, left);
                let r = Self::codegen_expr(builder, variables, module, print_func_id, right);
                match op {
                    Token::Plus => builder.ins().iadd(l, r),
                    Token::Minus => builder.ins().isub(l, r),
                    Token::Star => builder.ins().imul(l, r),
                    Token::Slash => builder.ins().sdiv(l, r),
                    Token::Percent => builder.ins().srem(l, r),
                    Token::EqualEqual => {
                        let val = builder.ins().icmp(IntCC::Equal, l, r);
                        builder.ins().uextend(types::I64, val)
                    }
                    Token::NotEqual => {
                        let val = builder.ins().icmp(IntCC::NotEqual, l, r);
                        builder.ins().uextend(types::I64, val)
                    }
                    Token::Less => {
                        let val = builder.ins().icmp(IntCC::SignedLessThan, l, r);
                        builder.ins().uextend(types::I64, val)
                    }
                    Token::LessEqual => {
                        let val = builder.ins().icmp(IntCC::SignedLessThanOrEqual, l, r);
                        builder.ins().uextend(types::I64, val)
                    }
                    Token::Greater => {
                        let val = builder.ins().icmp(IntCC::SignedGreaterThan, l, r);
                        builder.ins().uextend(types::I64, val)
                    }
                    Token::GreaterEqual => {
                        let val = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, l, r);
                        builder.ins().uextend(types::I64, val)
                    }
                    Token::AndAnd => {
                        let zero_val = builder.ins().iconst(types::I64, 0);
                        let l_bool = builder.ins().icmp(IntCC::NotEqual, l, zero_val);
                        let r_bool = builder.ins().icmp(IntCC::NotEqual, r, zero_val);
                        let val = builder.ins().band(l_bool, r_bool);
                        builder.ins().uextend(types::I64, val)
                    }
                    Token::OrOr => {
                        let zero_val = builder.ins().iconst(types::I64, 0);
                        let l_bool = builder.ins().icmp(IntCC::NotEqual, l, zero_val);
                        let r_bool = builder.ins().icmp(IntCC::NotEqual, r, zero_val);
                        let val = builder.ins().bor(l_bool, r_bool);
                        builder.ins().uextend(types::I64, val)
                    }
                    _ => unreachable!("Unsupported binary operator"),
                }
            }
            Expr::UnaryOp { op, expr } => {
                let val = Self::codegen_expr(builder, variables, module, print_func_id, expr);
                match op {
                    Token::Minus => {
                        let minus_one = builder.ins().iconst(types::I64, -1);
                        builder.ins().imul(val, minus_one)
                    }
                    Token::Not => {
                        let zero = builder.ins().iconst(types::I64, 0);
                        let bool_val = builder.ins().icmp(IntCC::Equal, val, zero);
                        builder.ins().uextend(types::I64, bool_val)
                    }
                    _ => unreachable!("Unsupported unary operator"),
                }
            }
        }
    }
}
