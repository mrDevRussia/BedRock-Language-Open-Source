// ================================================================
// BedRock Language Compiler — Type Inference Engine
// type_inference.rs
// ================================================================
//
// الوظيفة:
//   يستقبل الـ AST بعد الـ parsing (فيه TypeKind::Unknown)
//   ويحلل كل variable وexpression ويحدد نوعها الحقيقي
//   ويتحقق من صحة الأنواع قبل ما نبعت لـ codegen
//
// المراحل:
//   1. Pre-scan   — تسجيل كل الـ functions وأنواعها
//   2. Inference  — تحديد نوع كل variable من السياق
//   3. Checking   — التحقق من صحة الأنواع (overflow, mismatch)
//
// ================================================================

use crate::ast::{Statement, Expression, TypeKind};
use std::collections::HashMap;

// ================================================================
// FuncSignature — توقيع الـ function
// ================================================================

#[derive(Debug, Clone)]
struct FuncSignature {
    /// أنواع الـ parameters بالترتيب
    params: Vec<TypeKind>,
    /// نوع القيمة المرجعة
    return_type: TypeKind,
}

// ================================================================
// TypeEnv — بيئة الأنواع في الـ scope الحالي
// ================================================================

#[derive(Debug, Clone)]
struct TypeEnv {
    /// نوع كل variable: اسمها → نوعها
    vars: HashMap<String, TypeKind>,
    /// توقيع كل function
    funcs: HashMap<String, FuncSignature>,
}

impl TypeEnv {
    fn new() -> Self {
        TypeEnv {
            vars: HashMap::new(),
            funcs: HashMap::new(),
        }
    }

    /// بييجي الـ child scope يرث كل حاجة من الـ parent
    fn child(&self) -> Self {
        TypeEnv {
            vars: self.vars.clone(),
            funcs: self.funcs.clone(),
        }
    }

    fn set_var(&mut self, name: &str, kind: TypeKind) {
        self.vars.insert(name.to_string(), kind);
    }

    fn get_var(&self, name: &str) -> Option<&TypeKind> {
        self.vars.get(name)
    }

    fn set_func(&mut self, name: &str, sig: FuncSignature) {
        self.funcs.insert(name.to_string(), sig);
    }

    fn get_func(&self, name: &str) -> Option<&FuncSignature> {
        self.funcs.get(name)
    }
}

// ================================================================
// TypeInferencer — المحرك الرئيسي
// ================================================================

pub struct TypeInferencer {
    env: TypeEnv,
    /// اسم الـ function الحالية (لفحص الـ return type)
    current_func: Option<String>,
    /// عدد الـ warnings
    warning_count: usize,
    /// عدد الـ errors
    error_count: usize,
}

impl TypeInferencer {
    pub fn new() -> Self {
        TypeInferencer {
            env: TypeEnv::new(),
            current_func: None,
            warning_count: 0,
            error_count: 0,
        }
    }

    // ============================================================
    // نقطة الدخول الرئيسية
    // ============================================================

    /// يستقبل الـ AST ويرجعه بعد الـ inference
    /// لو فيه errors بيوقف الـ compile
    pub fn run(&mut self, stmts: Vec<Statement>) -> Vec<Statement> {
        // المرحلة 1: Pre-scan لتسجيل كل الـ functions
        // عشان نحل مشكلة forward references
        self.prescan_functions(&stmts);

        // المرحلة 2: Inference على كل الـ statements
        let result: Vec<Statement> = stmts
            .into_iter()
            .map(|s| self.infer_stmt(s))
            .collect();

        // المرحلة 3: تقرير النتيجة
        self.report_summary();

        result
    }

    // ============================================================
    // المرحلة 1: Pre-scan
    // ============================================================

    fn prescan_functions(&mut self, stmts: &[Statement]) {
        for stmt in stmts {
            if let Statement::FunctionDefine(name, params, _, return_type) = stmt {
                let sig = FuncSignature {
                    params: params.iter().map(|(_, k)| {
                        if *k == TypeKind::Unknown { TypeKind::U32 } else { k.clone() }
                    }).collect(),
                    return_type: if *return_type == TypeKind::Unknown {
                        TypeKind::U32
                    } else {
                        return_type.clone()
                    },
                };
                self.env.set_func(name, sig);
            }
        }
    }

    // ============================================================
    // المرحلة 2: Statement Inference
    // ============================================================

    fn infer_stmt(&mut self, stmt: Statement) -> Statement {
        match stmt {

            // --------------------------------------------------
            // let x@u32 = expr;
            // --------------------------------------------------
            Statement::Let(name, expr, kind) => {
                let inferred_expr = self.infer_expr(expr);
                let expr_type = self.type_of_expr(&inferred_expr);

                let final_kind = if kind == TypeKind::Unknown {
                    // مفيش annotation — نستنتج من الـ expression
                    if expr_type == TypeKind::Unknown {
                        self.warn(&format!(
                            "variable '{}' has no type annotation, defaulting to u32", name
                        ));
                        TypeKind::U32
                    } else {
                        expr_type.clone()
                    }
                } else {
                    // في annotation — نتحقق من التوافق
                    self.check_assignable(&expr_type, &kind, &name);
                    kind
                };

                self.env.set_var(&name, final_kind.clone());
                Statement::Let(name, inferred_expr, final_kind)
            }

            // --------------------------------------------------
            // root BASE@u32 = 0x80000000;
            // --------------------------------------------------
            Statement::Root(name, expr, kind) => {
                let inferred_expr = self.infer_expr(expr);
                let expr_type = self.type_of_expr(&inferred_expr);

                let final_kind = if kind == TypeKind::Unknown {
                    TypeKind::U32
                } else {
                    self.check_assignable(&expr_type, &kind, &name);
                    kind
                };

                self.env.set_var(&name, final_kind.clone());
                Statement::Root(name, inferred_expr, final_kind)
            }

            // --------------------------------------------------
            // x = expr;
            // --------------------------------------------------
            Statement::Assignment(name, expr) => {
                let inferred_expr = self.infer_expr(expr);
                let expr_type = self.type_of_expr(&inferred_expr);

                if let Some(var_type) = self.env.get_var(&name).cloned() {
                    if var_type != TypeKind::Unknown
                        && expr_type != TypeKind::Unknown
                        && !self.types_compatible(&expr_type, &var_type)
                    {
                        self.error(&format!(
                            "cannot assign '{}' to variable '{}' of type '{}'",
                            expr_type.name(), name, var_type.name()
                        ));
                    }
                }

                Statement::Assignment(name, inferred_expr)
            }

            // --------------------------------------------------
            // fn add(a@u32, b@u32)@u32 { ... }
            // --------------------------------------------------
            Statement::FunctionDefine(name, params, body, return_type) => {
                let final_return = if return_type == TypeKind::Unknown {
                    TypeKind::U32
                } else {
                    return_type
                };

                // resolve param types
                let resolved_params: Vec<(String, TypeKind)> = params
                    .into_iter()
                    .map(|(pname, pkind)| {
                        let resolved = if pkind == TypeKind::Unknown {
                            self.warn(&format!(
                                "parameter '{}' in fn '{}' has no type, defaulting to u32",
                                pname, name
                            ));
                            TypeKind::U32
                        } else {
                            pkind
                        };
                        (pname, resolved)
                    })
                    .collect();

                // child scope للـ function body
                let mut child_env = self.env.child();
                for (pname, pkind) in &resolved_params {
                    child_env.set_var(pname, pkind.clone());
                }

                // احفظ الـ state وادخل الـ function scope
                let saved_env = std::mem::replace(&mut self.env, child_env);
                let saved_func = self.current_func.replace(name.clone());

                // infer الـ body
                let inferred_body: Vec<Statement> = body
                    .into_iter()
                    .map(|s| self.infer_stmt(s))
                    .collect();

                // رجّع الـ state
                self.env = saved_env;
                self.current_func = saved_func;

                Statement::FunctionDefine(name, resolved_params, inferred_body, final_return)
            }

            // --------------------------------------------------
            // return expr;
            // --------------------------------------------------
            Statement::Return(maybe_expr) => {
                let inferred = maybe_expr.map(|e| {
                    let ie = self.infer_expr(e);

                    if let Some(func_name) = &self.current_func.clone() {
                        if let Some(sig) = self.env.get_func(func_name).cloned() {
                            let ret_type = self.type_of_expr(&ie);
                            if sig.return_type != TypeKind::Unknown
                                && ret_type != TypeKind::Unknown
                                && !self.types_compatible(&ret_type, &sig.return_type)
                            {
                                self.error(&format!(
                                    "fn '{}' return type is '{}' but got '{}'",
                                    func_name,
                                    sig.return_type.name(),
                                    ret_type.name()
                                ));
                            }
                        }
                    }

                    ie
                });
                Statement::Return(inferred)
            }

            // --------------------------------------------------
            // if (cond) { ... } else { ... }
            // --------------------------------------------------
            Statement::If(cond, then_body, else_body) => {
                let inferred_cond = self.infer_expr(cond);
                let inferred_then: Vec<Statement> = then_body
                    .into_iter()
                    .map(|s| self.infer_stmt(s))
                    .collect();
                let inferred_else = else_body.map(|stmts| {
                    stmts.into_iter().map(|s| self.infer_stmt(s)).collect()
                });
                Statement::If(inferred_cond, inferred_then, inferred_else)
            }

            // --------------------------------------------------
            // while (cond) { ... }
            // --------------------------------------------------
            Statement::While(cond, body) => {
                let inferred_cond = self.infer_expr(cond);
                let inferred_body: Vec<Statement> = body
                    .into_iter()
                    .map(|s| self.infer_stmt(s))
                    .collect();
                Statement::While(inferred_cond, inferred_body)
            }

            // --------------------------------------------------
            // loop { ... }
            // --------------------------------------------------
            Statement::Loop(body) => {
                let inferred_body: Vec<Statement> = body
                    .into_iter()
                    .map(|s| self.infer_stmt(s))
                    .collect();
                Statement::Loop(inferred_body)
            }

            // --------------------------------------------------
            // func_call(args);
            // --------------------------------------------------
            Statement::Call(name, args) => {
                let inferred_args = self.infer_call_args(&name, args);
                Statement::Call(name, inferred_args)
            }

            // --------------------------------------------------
            // poke(addr, val);
            // --------------------------------------------------
            Statement::Poke(addr, val) => {
                Statement::Poke(self.infer_expr(addr), self.infer_expr(val))
            }

            // --------------------------------------------------
            // outb(port, val);
            // --------------------------------------------------
            Statement::Outb(port, val) => {
                Statement::Outb(self.infer_expr(port), self.infer_expr(val))
            }

            // --------------------------------------------------
            // let buf@u8 = [0, 1, 2];
            // --------------------------------------------------
            Statement::ArrayDefine(name, vals, kind) => {
                let final_kind = if kind == TypeKind::Unknown {
                    self.warn(&format!(
                        "array '{}' has no element type, defaulting to u32", name
                    ));
                    TypeKind::U32
                } else {
                    kind.clone()
                };

                // تحقق من overflow لكل عنصر
                for (i, &v) in vals.iter().enumerate() {
                    if v > final_kind.max_value() {
                        self.error(&format!(
                            "array '{}' element [{}] = {} exceeds max for '{}' (max: {})",
                            name, i, v, final_kind.name(), final_kind.max_value()
                        ));
                    }
                }

                self.env.set_var(&name, final_kind.clone());
                Statement::ArrayDefine(name, vals, final_kind)
            }

            // --------------------------------------------------
            // array[idx] = val;
            // --------------------------------------------------
            Statement::ArrayAssign(name, idx, val) => {
                Statement::ArrayAssign(
                    name,
                    self.infer_expr(idx),
                    self.infer_expr(val),
                )
            }

            // باقي الـ statements مش محتاجة inference
            Statement::StructDefine(_, _) => stmt,
            Statement::StructInstance(_, _) => stmt,
            Statement::Bnw(_) => stmt,
            other => other,
        }
    }

    // ============================================================
    // Expression Inference
    // ============================================================

    fn infer_expr(&mut self, expr: Expression) -> Expression {
        match expr {

            // رقم ثابت — نحدد نوعه من حجمه أو من الـ annotation
            Expression::Number(n, kind) => {
                let resolved = if kind == TypeKind::Unknown {
                    infer_number_type(n)
                } else {
                    if n > kind.max_value() {
                        self.error(&format!(
                            "value {} exceeds max value for type '{}' (max: {})",
                            n, kind.name(), kind.max_value()
                        ));
                    }
                    kind
                };
                Expression::Number(n, resolved)
            }

            // variable — النوع من الـ env
            Expression::Variable(name) => {
                Expression::Variable(name)
            }

            // عملية حسابية
            Expression::BinaryOp(left, op, right) => {
                let inferred_left  = self.infer_expr(*left);
                let inferred_right = self.infer_expr(*right);
                let left_type  = self.type_of_expr(&inferred_left);
                let right_type = self.type_of_expr(&inferred_right);

                // تحذير لو الأنواع مختلفة في عملية حسابية
                if left_type != TypeKind::Unknown
                    && right_type != TypeKind::Unknown
                    && left_type != right_type
                {
                    self.warn(&format!(
                        "operation '{}' between '{}' and '{}' — types differ",
                        op, left_type.name(), right_type.name()
                    ));
                }

                Expression::BinaryOp(
                    Box::new(inferred_left),
                    op,
                    Box::new(inferred_right),
                )
            }

            // function call كـ expression
            Expression::Call(name, args) => {
                let inferred_args = self.infer_call_args(&name, args);
                Expression::Call(name, inferred_args)
            }

            // peek(addr) — بيرجع u32 دايماً
            Expression::Peek(addr) => {
                Expression::Peek(Box::new(self.infer_expr(*addr)))
            }

            // inb(port) — بيرجع u8
            Expression::Inb(port) => {
                Expression::Inb(Box::new(self.infer_expr(*port)))
            }

            // array[idx]
            Expression::ArrayAccess(name, idx) => {
                Expression::ArrayAccess(name, Box::new(self.infer_expr(*idx)))
            }

            // الحالات الجديدة مكانها الصحيح هنا:
            Expression::WaitKey => Expression::WaitKey,

            Expression::FieldAccess(var, field) => {
                Expression::FieldAccess(var, field)
            }

            Expression::FieldAssign(var, field, val) => {
                Expression::FieldAssign(var, field, Box::new(self.infer_expr(*val)))
            }
        }
    }

    // ============================================================
    // Helper — infer args وتحقق من الأنواع
    // ============================================================

    fn infer_call_args(&mut self, name: &str, args: Vec<Expression>) -> Vec<Expression> {
        let inferred: Vec<Expression> = args
            .into_iter()
            .map(|a| self.infer_expr(a))
            .collect();

        if let Some(sig) = self.env.get_func(name).cloned() {
            if inferred.len() != sig.params.len() {
                self.error(&format!(
                    "fn '{}' expects {} argument(s) but got {}",
                    name, sig.params.len(), inferred.len()
                ));
            } else {
                for (i, (arg, expected)) in inferred.iter().zip(sig.params.iter()).enumerate() {
                    let got = self.type_of_expr(arg);
                    if got != TypeKind::Unknown
                        && *expected != TypeKind::Unknown
                        && !self.types_compatible(&got, expected)
                    {
                        self.error(&format!(
                            "fn '{}' argument {} expects '{}' but got '{}'",
                            name, i + 1, expected.name(), got.name()
                        ));
                    }
                }
            }
        }

        inferred
    }

    // ============================================================
    // Helper — نوع الـ expression
    // ============================================================

    fn type_of_expr(&self, expr: &Expression) -> TypeKind {
        match expr {
            Expression::Number(_, kind)     => kind.clone(),
            Expression::Variable(name)      => {
                self.env.get_var(name).cloned().unwrap_or(TypeKind::Unknown)
            }
            Expression::BinaryOp(left, op, right) => {
                match op.as_str() {
                    "==" | "!=" | ">" | "<" | ">=" | "<=" => TypeKind::Bool,
                    _ => {
                        let lt = self.type_of_expr(left);
                        if lt != TypeKind::Unknown { lt }
                        else { self.type_of_expr(right) }
                    }
                }
            }
            Expression::Peek(_)             => TypeKind::Unknown,
            Expression::Inb(_)              => TypeKind::U8,
            Expression::Call(name, _)       => {
                self.env.get_func(name)
                    .map(|sig| sig.return_type.clone())
                    .unwrap_or(TypeKind::U32)
            }
            Expression::ArrayAccess(name, _) => {
                self.env.get_var(name).cloned().unwrap_or(TypeKind::U32)
            }
            _ => TypeKind::Unknown,
        }
    }

    // ============================================================
    // Helper — فحص توافق نوعين
    // ============================================================

    fn types_compatible(&self, from: &TypeKind, to: &TypeKind) -> bool {
        if from == to                           { return true; }
        if *from == TypeKind::Unknown
        || *to   == TypeKind::Unknown           { return true; }

        match (from, to) {
            // unsigned widening
            (TypeKind::U8,  TypeKind::U16) |
            (TypeKind::U8,  TypeKind::U32) |
            (TypeKind::U8,  TypeKind::U64) |
            (TypeKind::U16, TypeKind::U32) |
            (TypeKind::U16, TypeKind::U64) |
            (TypeKind::U32, TypeKind::U64) => true,
            // signed widening
            (TypeKind::I8,  TypeKind::I16) |
            (TypeKind::I8,  TypeKind::I32) |
            (TypeKind::I8,  TypeKind::I64) |
            (TypeKind::I16, TypeKind::I32) |
            (TypeKind::I16, TypeKind::I64) |
            (TypeKind::I32, TypeKind::I64) => true,
            // bool → unsigned
            (TypeKind::Bool, TypeKind::U8)  |
            (TypeKind::Bool, TypeKind::U32) => true,
            _ => false,
        }
    }

    // ============================================================
    // Helper — فحص assignment
    // ============================================================

    fn check_assignable(&mut self, from: &TypeKind, to: &TypeKind, context: &str) {
        if !self.types_compatible(from, to) {
            self.error(&format!(
                "type mismatch for '{}': cannot use '{}' as '{}'",
                context, from.name(), to.name()
            ));
        }
    }

    // ============================================================
    // Reporting
    // ============================================================

    fn warn(&mut self, msg: &str) {
        self.warning_count += 1;
        eprintln!("[TYPE WARNING] {}", msg);
    }

    fn error(&mut self, msg: &str) {
        self.error_count += 1;
        eprintln!("[TYPE ERROR] {}", msg);
        std::process::exit(1);
    }

    fn report_summary(&self) {
        eprintln!("-------------------------------------------");
        if self.warning_count > 0 {
            eprintln!("[TYPE] {} warning(s)", self.warning_count);
        }
        if self.error_count == 0 {
            eprintln!("[TYPE] All types resolved — OK");
        }
        eprintln!("-------------------------------------------");
    }
}

// ================================================================
// Helper — استنتاج نوع رقم من قيمته
// ================================================================

fn infer_number_type(_n: u64) -> TypeKind {
    TypeKind::Unknown
}