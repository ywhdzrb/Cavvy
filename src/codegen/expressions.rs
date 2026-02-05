//! 表达式代码生成
use crate::codegen::context::IRGenerator;
use crate::ast::*;
use crate::error::{EolResult, codegen_error};

impl IRGenerator {
    /// 生成表达式代码
    pub fn generate_expression(&mut self, expr: &Expr) -> EolResult<String> {
        match expr {
            Expr::Literal(lit) => self.generate_literal(lit),
            Expr::Identifier(name) => {
                // 加载变量值
                let temp = self.new_temp();
                // 从 var_types 中获取变量类型，默认为 i64（为了向后兼容）
                let var_type = self.var_types.get(name).cloned().unwrap_or_else(|| "i64".to_string());
                self.emit_line(&format!("  {} = load {}, {}* %{}, align 8", temp, var_type, var_type, name));
                Ok(format!("{} {}", var_type, temp))
            }
            Expr::Binary(bin) => self.generate_binary_expression(bin),
            Expr::Unary(unary) => self.generate_unary_expression(unary),
            Expr::Call(call) => self.generate_call_expression(call),
            Expr::Assignment(assign) => self.generate_assignment(assign),
            Expr::Cast(cast) => self.generate_cast_expression(cast),
            Expr::MemberAccess(member) => self.generate_member_access(member),
            Expr::New(new_expr) => self.generate_new_expression(new_expr),
        }
    }

    /// 生成字面量代码
    fn generate_literal(&mut self, lit: &LiteralValue) -> EolResult<String> {
        match lit {
            LiteralValue::Int(val) => Ok(format!("i64 {}", val)),
            LiteralValue::Float(val) => Ok(format!("double {}", val)),
            LiteralValue::Bool(val) => Ok(format!("i1 {}", if *val { 1 } else { 0 })),
            LiteralValue::String(s) => {
                let global_name = self.get_or_create_string_constant(s);
                let temp = self.new_temp();
                let len = s.len() + 1;
                self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0", 
                    temp, len, len, global_name));
                Ok(format!("i8* {}", temp))
            }
            LiteralValue::Char(c) => Ok(format!("i8 {}", *c as u8)),
            LiteralValue::Null => Ok("i64 0".to_string()),
        }
    }

    /// 生成二元表达式代码
    fn generate_binary_expression(&mut self, bin: &BinaryExpr) -> EolResult<String> {
        let left = self.generate_expression(&bin.left)?;
        let right = self.generate_expression(&bin.right)?;
        
        // 解析类型和值
        let (left_type, left_val) = self.parse_typed_value(&left);
        let (right_type, right_val) = self.parse_typed_value(&right);
        
        let temp = self.new_temp();
        
        match bin.op {
            BinaryOp::Add => {
                // 字符串拼接处理
                if left_type == "i8*" && right_type == "i8*" {
                    // 调用内建的字符串拼接函数
                    self.emit_line(&format!("  {} = call i8* @__eol_string_concat(i8* {}, i8* {})",
                        temp, left_val, right_val));
                } else if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = add {} {}, {}",
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fadd {} {}, {}",
                        temp, left_type, left_val, right_val));
                }
            }
            BinaryOp::Sub => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = sub {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fsub {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
            }
            BinaryOp::Mul => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = mul {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fmul {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
            }
            BinaryOp::Div => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = sdiv {} {}, {}",
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fdiv {} {}, {}",
                        temp, left_type, left_val, right_val));
                }
            }
            BinaryOp::Mod => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = srem {} {}, {}",
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = frem {} {}, {}",
                        temp, left_type, left_val, right_val));
                }
            }
            BinaryOp::Eq => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = icmp eq {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fcmp oeq {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::Ne => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = icmp ne {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fcmp one {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::Lt => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = icmp slt {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fcmp olt {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::Le => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = icmp sle {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fcmp ole {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::Gt => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = icmp sgt {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fcmp ogt {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::Ge => {
                if left_type.starts_with("i") {
                    self.emit_line(&format!("  {} = icmp sge {} {}, {}", 
                        temp, left_type, left_val, right_val));
                } else {
                    self.emit_line(&format!("  {} = fcmp oge {} {}, {}", 
                        temp, left_type, left_val, right_val));
                }
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::And => {
                self.emit_line(&format!("  {} = and {} {}, {}", 
                    temp, left_type, left_val, right_val));
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::Or => {
                self.emit_line(&format!("  {} = or {} {}, {}",
                    temp, left_type, left_val, right_val));
                return Ok(format!("i1 {}", temp));
            }
            BinaryOp::BitAnd => {
                self.emit_line(&format!("  {} = and {} {}, {}",
                    temp, left_type, left_val, right_val));
            }
            BinaryOp::BitOr => {
                self.emit_line(&format!("  {} = or {} {}, {}",
                    temp, left_type, left_val, right_val));
            }
            BinaryOp::BitXor => {
                self.emit_line(&format!("  {} = xor {} {}, {}",
                    temp, left_type, left_val, right_val));
            }
            BinaryOp::Shl => {
                self.emit_line(&format!("  {} = shl {} {}, {}",
                    temp, left_type, left_val, right_val));
            }
            BinaryOp::Shr => {
                self.emit_line(&format!("  {} = ashr {} {}, {}",
                    temp, left_type, left_val, right_val));
            }
            BinaryOp::UnsignedShr => {
                self.emit_line(&format!("  {} = lshr {} {}, {}",
                    temp, left_type, left_val, right_val));
            }
        }
        
        Ok(format!("{} {}", left_type, temp))
    }

    /// 生成一元表达式代码
    fn generate_unary_expression(&mut self, unary: &UnaryExpr) -> EolResult<String> {
        let operand = self.generate_expression(&unary.operand)?;
        let (op_type, op_val) = self.parse_typed_value(&operand);
        let temp = self.new_temp();
        
        match unary.op {
            UnaryOp::Neg => {
                if op_type.starts_with("i") {
                    self.emit_line(&format!("  {} = sub {} 0, {}",
                        temp, op_type, op_val));
                } else {
                    self.emit_line(&format!("  {} = fneg {} {}",
                        temp, op_type, op_val));
                }
            }
            UnaryOp::Not => {
                self.emit_line(&format!("  {} = xor {} {}, 1",
                    temp, op_type, op_val));
                return Ok(format!("i1 {}", temp));
            }
            UnaryOp::BitNot => {
                // 位取反：xor 操作数与 -1
                if op_type.starts_with("i") {
                    self.emit_line(&format!("  {} = xor {} {}, -1",
                        temp, op_type, op_val));
                } else {
                    // 浮点数不支持位取反，但类型系统应该已经阻止了这种情况
                    return Err(codegen_error("Bitwise NOT not supported for floating point".to_string()));
                }
            }
            UnaryOp::PreInc | UnaryOp::PostInc => {
                // i++ 或 ++i
                let one = if op_type.starts_with("i") { "1" } else { "1.0" };
                if op_type.starts_with("i") {
                    self.emit_line(&format!("  {} = add {} {}, {}",
                        temp, op_type, op_val, one));
                } else {
                    self.emit_line(&format!("  {} = fadd {} {}, {}",
                        temp, op_type, op_val, one));
                }
                // 存储回变量
                if let Expr::Identifier(name) = unary.operand.as_ref() {
                    self.emit_line(&format!("  store {} {}, {}* %{}",
                        op_type, temp, op_type, name));
                }
                // 前置返回新值，后置返回旧值
                if unary.op == UnaryOp::PreInc {
                    return Ok(format!("{} {}", op_type, temp));
                } else {
                    return Ok(format!("{} {}", op_type, op_val));
                }
            }
            UnaryOp::PreDec | UnaryOp::PostDec => {
                // i-- 或 --i
                let one = if op_type.starts_with("i") { "1" } else { "1.0" };
                if op_type.starts_with("i") {
                    self.emit_line(&format!("  {} = sub {} {}, {}",
                        temp, op_type, op_val, one));
                } else {
                    self.emit_line(&format!("  {} = fsub {} {}, {}",
                        temp, op_type, op_val, one));
                }
                // 存储回变量
                if let Expr::Identifier(name) = unary.operand.as_ref() {
                    self.emit_line(&format!("  store {} {}, {}* %{}",
                        op_type, temp, op_type, name));
                }
                // 前置返回新值，后置返回旧值
                if unary.op == UnaryOp::PreDec {
                    return Ok(format!("{} {}", op_type, temp));
                } else {
                    return Ok(format!("{} {}", op_type, op_val));
                }
            }
        }
        
        Ok(format!("{} {}", op_type, temp))
    }

    /// 生成函数调用表达式代码
    fn generate_call_expression(&mut self, call: &CallExpr) -> EolResult<String> {
        // 处理 print 和 println 函数
        if let Expr::Identifier(name) = call.callee.as_ref() {
            if name == "print" {
                return self.generate_print_call(&call.args, false);
            }
            if name == "println" {
                return self.generate_print_call(&call.args, true);
            }
        }
        
        // 处理普通函数调用
        let fn_name = match call.callee.as_ref() {
            Expr::Identifier(name) => name.clone(),
            Expr::MemberAccess(member) => {
                if let Expr::Identifier(class_name) = member.object.as_ref() {
                    format!("{}. {}", class_name, member.member)
                } else {
                    return Err(codegen_error("Invalid method call".to_string()));
                }
            }
            _ => return Err(codegen_error("Invalid function call".to_string())),
        };
        
        let args: Vec<String> = call.args.iter()
            .map(|arg| self.generate_expression(arg))
            .collect::<EolResult<Vec<_>>>()?;
        
        let temp = self.new_temp();
        self.emit_line(&format!("  {} = call i64 @{}({})", 
            temp, fn_name, args.join(", ")));
        
        Ok(format!("i64 {}", temp))
    }

    /// 生成 print/println 调用代码
    fn generate_print_call(&mut self, args: &[Expr], newline: bool) -> EolResult<String> {
        if args.is_empty() {
            return Err(codegen_error("print requires at least one argument".to_string()));
        }
        
        let first_arg = &args[0];
        
        match first_arg {
            Expr::Literal(LiteralValue::String(s)) => {
                let global_name = self.get_or_create_string_constant(s);
                let fmt_str = if newline { "%s\n" } else { "%s" };
                let fmt_name = self.get_or_create_string_constant(fmt_str);
                let len = s.len() + 1;
                let fmt_len = fmt_str.len() + 1; // 加上null终止符
                
                let str_ptr = self.new_temp();
                let fmt_ptr = self.new_temp();
                
                self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                    str_ptr, len, len, global_name));
                self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                    fmt_ptr, fmt_len, fmt_len, fmt_name));
                
                self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, i8* {})",
                    fmt_ptr, str_ptr));
            }
            Expr::Literal(LiteralValue::Int(_)) => {
                let value = self.generate_expression(first_arg)?;
                let (_, val) = self.parse_typed_value(&value);
                let fmt_str = if newline { "%ld\n" } else { "%ld" };
                let fmt_name = self.get_or_create_string_constant(fmt_str);
                let fmt_len = fmt_str.len() + 1;
                
                let fmt_ptr = self.new_temp();
                self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                    fmt_ptr, fmt_len, fmt_len, fmt_name));
                
                self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, i64 {})",
                    fmt_ptr, val));
            }
            _ => {
                // 根据类型决定格式字符串
                let value = self.generate_expression(first_arg)?;
                let (type_str, val) = self.parse_typed_value(&value);
                
                if type_str == "i8*" {
                    // 字符串指针类型
                    let fmt_str = if newline { "%s\n" } else { "%s" };
                    let fmt_name = self.get_or_create_string_constant(fmt_str);
                    let fmt_len = fmt_str.len() + 1;
                    let fmt_ptr = self.new_temp();
                    self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                        fmt_ptr, fmt_len, fmt_len, fmt_name));
                    self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, i8* {})",
                        fmt_ptr, val));
                } else if type_str.starts_with("i") && type_str != "i8*" {
                    // 整数类型（排除i8*）
                    // 需要将整数扩展为 i64 以匹配 %ld 格式
                    let fmt_str = if newline { "%ld\n" } else { "%ld" };
                    let fmt_name = self.get_or_create_string_constant(fmt_str);
                    let fmt_len = fmt_str.len() + 1;
                    let fmt_ptr = self.new_temp();
                    self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                        fmt_ptr, fmt_len, fmt_len, fmt_name));
                    
                    // 如果类型不是 i64，需要扩展
                    let final_val = if type_str != "i64" {
                        let ext_temp = self.new_temp();
                        self.emit_line(&format!("  {} = sext {} {} to i64", ext_temp, type_str, val));
                        ext_temp
                    } else {
                        val.to_string()
                    };
                    
                    self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, i64 {})",
                        fmt_ptr, final_val));
                } else if type_str == "double" || type_str == "float" {
                    // 浮点数类型
                    let fmt_str = if newline { "%f\n" } else { "%f" };
                    let fmt_name = self.get_or_create_string_constant(fmt_str);
                    let fmt_len = fmt_str.len() + 1;
                    let fmt_ptr = self.new_temp();
                    self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                        fmt_ptr, fmt_len, fmt_len, fmt_name));
                    self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, double {})",
                        fmt_ptr, val));
                } else {
                    // 默认作为字符串处理
                    let fmt_str = if newline { "%s\n" } else { "%s" };
                    let fmt_name = self.get_or_create_string_constant(fmt_str);
                    let fmt_len = fmt_str.len() + 1;
                    let fmt_ptr = self.new_temp();
                    self.emit_line(&format!("  {} = getelementptr [{} x i8], [{} x i8]* {}, i64 0, i64 0",
                        fmt_ptr, fmt_len, fmt_len, fmt_name));
                    self.emit_line(&format!("  call i32 (i8*, ...) @printf(i8* {}, {})",
                        fmt_ptr, value));
                }
            }
        }
        
        Ok("i64 0".to_string())
    }

    /// 生成赋值表达式代码
    fn generate_assignment(&mut self, assign: &AssignmentExpr) -> EolResult<String> {
        let value = self.generate_expression(&assign.value)?;
        let (_, val) = self.parse_typed_value(&value);
        
        match assign.target.as_ref() {
            Expr::Identifier(name) => {
                // 简化处理，假设都是i64类型
                self.emit_line(&format!("  store i64 {}, i64* %{}", val, name));
                Ok(value)
            }
            _ => Err(codegen_error("Invalid assignment target".to_string()))
        }
    }

    /// 生成类型转换表达式代码
    fn generate_cast_expression(&mut self, cast: &CastExpr) -> EolResult<String> {
        let expr_value = self.generate_expression(&cast.expr)?;
        let (from_type, val) = self.parse_typed_value(&expr_value);
        let to_type = self.type_to_llvm(&cast.target_type);
        
        let temp = self.new_temp();
        
        // 相同类型无需转换
        if from_type == to_type {
            return Ok(format!("{} {}", to_type, val));
        }
        
        // 整数到整数
        if from_type.starts_with("i") && to_type.starts_with("i") && !from_type.ends_with("*") && !to_type.ends_with("*") {
            let from_bits: u32 = from_type.trim_start_matches('i').parse().unwrap_or(64);
            let to_bits: u32 = to_type.trim_start_matches('i').parse().unwrap_or(64);
            
            if to_bits > from_bits {
                // 符号扩展
                self.emit_line(&format!("  {} = sext {} {} to {}",
                    temp, from_type, val, to_type));
            } else {
                // 截断
                self.emit_line(&format!("  {} = trunc {} {} to {}",
                    temp, from_type, val, to_type));
            }
            return Ok(format!("{} {}", to_type, temp));
        }
        
        // 整数到浮点
        if from_type.starts_with("i") && !from_type.ends_with("*") && 
           (to_type == "float" || to_type == "double") {
            self.emit_line(&format!("  {} = sitofp {} {} to {}",
                temp, from_type, val, to_type));
            return Ok(format!("{} {}", to_type, temp));
        }
        
        // 浮点到整数
        if (from_type == "float" || from_type == "double") && 
           to_type.starts_with("i") && !to_type.ends_with("*") {
            self.emit_line(&format!("  {} = fptosi {} {} to {}",
                temp, from_type, val, to_type));
            return Ok(format!("{} {}", to_type, temp));
        }
        
        // 浮点到浮点
        if (from_type == "float" || from_type == "double") && 
           (to_type == "float" || to_type == "double") {
            if to_type == "double" {
                self.emit_line(&format!("  {} = fpext {} {} to {}",
                    temp, from_type, val, to_type));
            } else {
                self.emit_line(&format!("  {} = fptrunc {} {} to {}",
                    temp, from_type, val, to_type));
            }
            return Ok(format!("{} {}", to_type, temp));
        }
        
        Err(codegen_error(format!("Unsupported cast from {} to {}", from_type, to_type)))
    }

    /// 生成成员访问表达式代码
    fn generate_member_access(&mut self, member: &MemberAccessExpr) -> EolResult<String> {
        // 暂不实现，返回占位符
        Err(codegen_error("Member access not yet implemented".to_string()))
    }

    /// 生成 new 表达式代码
    fn generate_new_expression(&mut self, new_expr: &NewExpr) -> EolResult<String> {
        // 暂不实现，返回占位符
        Err(codegen_error("New expression not yet implemented".to_string()))
    }
}
