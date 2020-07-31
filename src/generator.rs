use std::collections::HashMap;

use crate::ast::*;
use crate::scanner::*;

type VariableMap = HashMap<String, isize>;

fn generate_expression(expression: &Expr, variables: &mut VariableMap, stack_index: &mut isize) -> String {
    match expression {
        Expr::Const(num) => return format!("  movl ${}, %eax\n", num),
        Expr::BinOp(op, lhs, rhs) => {
            let mut generated: String;
            match op {
                Operator::Plus | Operator::Minus | Operator::Star | Operator::Slash | Operator::Modulo => {
                    // We reverse who is in ecx register because subtraction is dst - src -> dst.
                    // Otherwise we'd have to `movl %ecx, %eax`. This is an optimization.
                    if *op == Operator::Minus || *op == Operator::Slash {
                        generated = generate_expression(rhs, variables, stack_index);
                        generated.push_str("  movl %eax, %ecx\n"); // rhs is now in ecx register
                        generated.push_str(&generate_expression(lhs, variables, stack_index));
                    } else {
                        generated = generate_expression(lhs, variables, stack_index);
                        generated.push_str("  movl %eax, %ecx\n"); // lhs is now in ecx register
                        generated.push_str(&generate_expression(rhs, variables, stack_index));
                    }

                    match op {
                        Operator::Plus => generated.push_str("  addl %ecx, %eax\n"),
                        Operator::Minus => generated.push_str("  subl %ecx, %eax\n"),
                        Operator::Star => generated.push_str("  imul %ecx, %eax\n"),
                        Operator::Slash => generated.push_str("  idivl %ecx\n  movl %ecx, %eax\n"),
                        Operator::Modulo => generated.push_str("  idivl %ecx\n  movl %edx, %eax\n"),
                        _ => unimplemented!()
                    }
                }
                Operator::EqualEqual  | Operator::NotEqual  | // Equality and comparison
                Operator::LessThan    | Operator::LessEqual |
                Operator::GreaterThan | Operator::GreaterEqual => {
                    generated = generate_expression(rhs, variables, stack_index);
                    generated.push_str("  push %eax\n");
                    generated.push_str(&generate_expression(lhs, variables, stack_index)); // lhs is now in eax register
                    generated.push_str("  pop %ecx\n"); // rhs is now in ecx register
                    generated.push_str("  cmpl %eax, %ecx\n  movl $0, %eax\n");
                    generated.push_str(match op {
                        Operator::EqualEqual   => "  sete %al\n",
                        Operator::NotEqual     => "  setne %al\n",
                        Operator::LessThan     => "  setl %al\n",
                        Operator::LessEqual    => "  setle %al\n",
                        Operator::GreaterThan  => "  setg %al\n",
                        Operator::GreaterEqual => "  setge %al\n",
                        _ => unimplemented!()
                    });
                }
                Operator::Or | Operator::And => {
                    generated = generate_expression(lhs, variables, stack_index);
                    generated.push_str("  push %eax\n");
                    generated.push_str(&generate_expression(rhs, variables, stack_index));
                    generated.push_str("  pop %ecx\n");
                    generated.push_str(match op {
                        Operator::Or  => "  orl %ecx, %eax\n  movl $0, %eax\n  setne %al\n",
                        /* 1. Set `cl` to 1 if lhs != 0
                         * 2. Set `al` to 1 if rhs != 0
                         * 3. Store `al` and `cl` in `al`
                         */
                        Operator::And => "  cmpl $0, %eax\n  setne %cl\n  cmpl $0, %eax\n  movl $0, %eax\n  setne %al\n  andb %cl, %al\n",
                        _ => unsafe { ::std::hint::unreachable_unchecked() },
                    });
                }
                _ if op.is_bitwise() => {
                    generated = generate_expression(lhs, variables, stack_index);
                    generated.push_str("  push %eax\n");
                    generated.push_str(&generate_expression(rhs, variables, stack_index));
                    generated.push_str("  pop %ebx\n");
                    match op {
                        Operator::BitwiseAND => generated.push_str("  and %ebx, %eax\n"),
                        Operator::BitwiseOR => generated.push_str("  or %ebx, %eax\n"),
                        Operator::BitwiseXOR => generated.push_str("  xor %ebx, %eax\n"),
                        Operator::BitwiseShiftLeft => generated.push_str("  shl %ebx, %eax\n"),
                        Operator::BitwiseShiftRight => generated.push_str("  shr %ebx, %eax\n"),
                        _ => unimplemented!(), // should be impossible
                    }
                }
                _ => unimplemented!()
            }
            return generated;
        }
        Expr::UnaryOp(op, expr) => {
            let mut generated_expr = generate_expression(expr, variables, stack_index);
            match op {
                Operator::LogicalNegation => {
                    generated_expr.push_str("  cmpl $0, %eax\n  sete %al\n");
                }
                Operator::Minus => {
                    generated_expr.push_str("  neg %eax\n");
                }
                Operator::BitwiseComplement => {
                    generated_expr.push_str("  not %eax\n");
                }
                _ => unimplemented!(),
            }
            return generated_expr;
        }
        Expr::Assign(op, name, expr) => { // `op` is guaranteed valid assignment operator by parser
            let mut output = generate_expression(expr, variables, stack_index);

            if !variables.contains_key(name) {
                panic!("Attempting to assign to an undeclared variable");
            }

            let offset: isize = *variables.get(name).unwrap();
            match op {
                Operator::Assignment => output.push_str(&format!("  movl %eax, {}(%ebp)\n", offset)),
                Operator::PlusAssign => output.push_str(&format!("  addl %eax, {}(%ebp)\n", offset)),
                Operator::MinusAssign => output.push_str(&format!("  subl %eax, {}(%ebp)\n", offset)),
                Operator::StarAssign => output.push_str(&format!("  imul %eax, {}(%ebp)\n", offset)),
                Operator::SlashAssign => output.push_str(&format!("  movl %eax, %ecx\n  movl {0}(%ebp), %eax\n  idivl %ecx\n  movl %ecx, %eax\n  movl %eax, {0}(%ebp)\n", offset)),
                Operator::ModAssign => output.push_str(&format!("  movl %eax, %ecx\n  movl {0}(%ebp), %eax\n  idivl %ecx\n  movl %edx, %eax\n  movl %eax, {0}(%ebp)\n", offset)),
                // NOTE: Operators LeftShiftAssign, RightShiftAssign, ANDAssign, ORAssign, and XORAssign are all omitted until further development.
                _ => unimplemented!(),
            }

            return output;
        }
        Expr::Var(name) => {
            if !variables.contains_key(name) {
                panic!("Attempting to reference an undeclared variable");
            }

            let offset: isize = *variables.get(name).unwrap();
            return format!("  movl {}(%ebp), %eax\n", offset);
        }
    }
}

fn generate_statement(statement: &Statement, function_name: &str, variables: &mut VariableMap, stack_index: &mut isize) -> String {
    let mut output = String::new();
    match statement {
        Statement::Return(expr) => {
            output.push_str(&generate_expression(expr, variables, stack_index));
            output.push_str(&format!("  goto _{}_epilogue\n", function_name));
        }
        Statement::Declare(name, value) => {
            if variables.contains_key(name) {
                panic!("Can't declare a variable twice in the same scope");
            }

            if let Some(expr) = value {
                output.push_str(&generate_expression(expr, variables, stack_index));
                output.push_str("  pushl %eax\n");
            } else {
                output.push_str("  pushl $0\n");
            }

            variables.insert(name.clone(), *stack_index);
            *stack_index -= 4;
        }
        Statement::Expr(expr) => output.push_str(&generate_expression(expr, variables, stack_index)),
    }
    output
}

pub fn generate(ast: &Program) -> String {
    let mut output = String::new();
    match ast {
        Program::Func(name, statements) => { // Code generated when a function is made
            let mut variable_map = VariableMap::new();
            let mut stack_index = -4isize; // ESP - 4
            output.push_str(&format!("  .globl _{0}\n_{0}:\n", name));

            // Function prologue
            output.push_str("  push %ebp\n  movl %esp, %ebp\n");

            for statement in statements {
                output.push_str(&generate_statement(statement, &name, &mut variable_map, &mut stack_index));
            }

            if !output.ends_with(&format!("goto _{}_epilogue\n", name)) {
                // No return issued, so we return zero by default
                output.push_str("  movl $0, %eax\n");
            } else {
                // Output ends with "goto _{}_epilogue", which is dumb because we define the epilogue immediately after
                output = output[0..output.len() - format!("  goto _{}_epilogue\n", name).len()].to_owned();
            }

            // Function epilogue
            output.push_str(&format!("_{}_epilogue:\n  movl %ebp, %esp\n  pop %ebp\n  ret\n", name));
        }
    }
    output
}
