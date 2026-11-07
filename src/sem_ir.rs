// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation Language Toolchain
// Licensed under the MIT License.

//! Carbon Semantic Intermediate Representation (SemIR)
//! Matches and extends `carbon-lang/toolchain/sem_ir` with typed instructions,
//! basic blocks, constant folding, and Control Flow Graphs (CFG).

use crate::ast::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct InstId(pub usize);

impl fmt::Display for InstId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemInst {
    // Constants & Literals
    IntConst {
        val: i64,
        ty: Type,
    },
    FloatConst {
        val: f64,
        ty: Type,
    },
    BoolConst {
        val: bool,
    },
    StringConst {
        val: String,
    },

    // Memory & Variables
    VarStorage {
        name: String,
        ty: Type,
    },
    Load {
        var_name: String,
        ty: Type,
    },
    Store {
        var_name: String,
        val: InstId,
    },
    LoadIndex {
        array_name: String,
        index: InstId,
        ty: Type,
    },
    StoreIndex {
        array_name: String,
        index: InstId,
        val: InstId,
    },

    // Arithmetic & Bitwise
    Binary {
        op: BinaryOp,
        lhs: InstId,
        rhs: InstId,
        ty: Type,
    },
    Unary {
        op: UnaryOp,
        operand: InstId,
        ty: Type,
    },

    // Control Flow
    Branch {
        target_block: usize,
    },
    BranchIf {
        cond: InstId,
        then_block: usize,
        else_block: usize,
    },
    Return {
        val: Option<InstId>,
    },
    Call {
        callee: String,
        args: Vec<InstId>,
        ty: Type,
    },

    // Hardware-Assisted & Hypervisor Extensions
    VmcsTransition {
        exit_reason: u32,
        guest_rip: u64,
    },
    EptWalk {
        gpa: u64,
        hpa: u64,
    },
    LogUpLookup {
        table_id: u32,
        key: InstId,
        val: InstId,
    },
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub instructions: Vec<InstId>,
}

#[derive(Debug, Clone)]
pub struct SemFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub is_extern: bool,
    pub blocks: Vec<BasicBlock>,
    pub instructions: Vec<SemInst>,
}

#[derive(Debug, Clone)]
pub struct SemProgram {
    pub functions: Vec<SemFunction>,
    pub globals: Vec<(String, Type, Option<i64>)>,
}

impl SemProgram {
    pub fn from_ast(program: &Program) -> Self {
        let mut sem_prog = SemProgram {
            functions: Vec::new(),
            globals: Vec::new(),
        };

        for global in &program.globals {
            if let Stmt::VarDecl { name, ty, init, .. } = global {
                let t = ty.clone().unwrap_or(Type::I32);
                let init_val = if let Some(Expr::Integer(n, _)) = init {
                    Some(*n)
                } else {
                    None
                };
                sem_prog.globals.push((name.clone(), t, init_val));
            }
        }

        for func in &program.functions {
            sem_prog.functions.push(SemFunction::from_ast(func));
        }

        sem_prog
    }
}

impl SemFunction {
    pub fn from_ast(func: &Function) -> Self {
        let mut sem_fn = SemFunction {
            name: func.name.clone(),
            params: func
                .params
                .iter()
                .map(|p| (p.name.clone(), p.ty.clone()))
                .collect(),
            return_type: func.return_type.clone(),
            is_extern: func.is_extern,
            blocks: Vec::new(),
            instructions: Vec::new(),
        };

        if func.is_extern {
            return sem_fn;
        }

        let mut builder = CfgBuilder::new();
        for stmt in &func.body {
            builder.lower_stmt(stmt);
        }

        // Ensure every block has a terminator
        for i in 0..builder.blocks.len() {
            let is_term = builder.blocks[i].instructions.last().is_some_and(|&id| {
                matches!(
                    builder.instructions.get(id.0),
                    Some(SemInst::Return { .. })
                        | Some(SemInst::Branch { .. })
                        | Some(SemInst::BranchIf { .. })
                )
            });
            if !is_term {
                if i + 1 < builder.blocks.len() {
                    let jmp_id = InstId(builder.instructions.len());
                    builder.instructions.push(SemInst::Branch {
                        target_block: i + 1,
                    });
                    builder.blocks[i].instructions.push(jmp_id);
                } else {
                    let ret_id = InstId(builder.instructions.len());
                    builder.instructions.push(SemInst::Return { val: None });
                    builder.blocks[i].instructions.push(ret_id);
                }
            }
        }

        sem_fn.blocks = builder.blocks;
        sem_fn.instructions = builder.instructions;
        sem_fn
    }
}

struct CfgBuilder {
    instructions: Vec<SemInst>,
    blocks: Vec<BasicBlock>,
    current_block: usize,
    loop_stack: Vec<(usize, usize)>, // (continue_target, break_target)
}

impl CfgBuilder {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            blocks: vec![BasicBlock {
                id: 0,
                instructions: Vec::new(),
            }],
            current_block: 0,
            loop_stack: Vec::new(),
        }
    }

    fn alloc_block(&mut self) -> usize {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            id,
            instructions: Vec::new(),
        });
        id
    }

    fn emit(&mut self, inst: SemInst) -> InstId {
        let id = InstId(self.instructions.len());
        self.instructions.push(inst);
        self.blocks[self.current_block].instructions.push(id);
        id
    }

    fn is_current_terminated(&self) -> bool {
        self.blocks[self.current_block]
            .instructions
            .last()
            .is_some_and(|&id| {
                matches!(
                    self.instructions.get(id.0),
                    Some(SemInst::Return { .. })
                        | Some(SemInst::Branch { .. })
                        | Some(SemInst::BranchIf { .. })
                )
            })
    }

    fn lower_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, ty, init, .. } => {
                let t = ty.clone().unwrap_or(Type::I32);
                self.emit(SemInst::VarStorage {
                    name: name.clone(),
                    ty: t,
                });

                if let Some(init_expr) = init {
                    let init_id = self.lower_expr(init_expr);
                    self.emit(SemInst::Store {
                        var_name: name.clone(),
                        val: init_id,
                    });
                }
            }
            Stmt::Expr(expr) => {
                self.lower_expr(expr);
            }
            Stmt::Return { value, .. } => {
                let val_id = value.as_ref().map(|v| self.lower_expr(v));
                self.emit(SemInst::Return { val: val_id });
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_id = self.lower_expr(cond);
                let then_block = self.alloc_block();
                let merge_block = self.alloc_block();
                let else_block = if else_branch.is_some() {
                    self.alloc_block()
                } else {
                    merge_block
                };

                self.emit(SemInst::BranchIf {
                    cond: cond_id,
                    then_block,
                    else_block,
                });

                // Lower then branch
                self.current_block = then_block;
                for s in then_branch {
                    self.lower_stmt(s);
                }
                if !self.is_current_terminated() {
                    self.emit(SemInst::Branch {
                        target_block: merge_block,
                    });
                }

                // Lower else branch
                if let Some(else_stmts) = else_branch {
                    self.current_block = else_block;
                    for s in else_stmts {
                        self.lower_stmt(s);
                    }
                    if !self.is_current_terminated() {
                        self.emit(SemInst::Branch {
                            target_block: merge_block,
                        });
                    }
                }

                self.current_block = merge_block;
            }
            Stmt::While { cond, body, .. } => {
                let cond_block = self.alloc_block();
                let body_block = self.alloc_block();
                let exit_block = self.alloc_block();

                if !self.is_current_terminated() {
                    self.emit(SemInst::Branch {
                        target_block: cond_block,
                    });
                }

                self.current_block = cond_block;
                let cond_id = self.lower_expr(cond);
                self.emit(SemInst::BranchIf {
                    cond: cond_id,
                    then_block: body_block,
                    else_block: exit_block,
                });

                self.loop_stack.push((cond_block, exit_block));

                self.current_block = body_block;
                for s in body {
                    self.lower_stmt(s);
                }
                if !self.is_current_terminated() {
                    self.emit(SemInst::Branch {
                        target_block: cond_block,
                    });
                }

                self.loop_stack.pop();
                self.current_block = exit_block;
            }
            Stmt::For {
                var_name,
                var_ty,
                iterable,
                body,
                ..
            } => {
                let t = var_ty.clone().unwrap_or(Type::I32);
                self.emit(SemInst::VarStorage {
                    name: var_name.clone(),
                    ty: t.clone(),
                });

                if let Expr::Range {
                    start,
                    end,
                    inclusive,
                    ..
                } = iterable
                {
                    let start_id = self.lower_expr(start);
                    self.emit(SemInst::Store {
                        var_name: var_name.clone(),
                        val: start_id,
                    });

                    let cond_block = self.alloc_block();
                    let body_block = self.alloc_block();
                    let step_block = self.alloc_block();
                    let exit_block = self.alloc_block();

                    if !self.is_current_terminated() {
                        self.emit(SemInst::Branch {
                            target_block: cond_block,
                        });
                    }

                    // Cond block: check var < end (or <= end)
                    self.current_block = cond_block;
                    let cur_val = self.emit(SemInst::Load {
                        var_name: var_name.clone(),
                        ty: t.clone(),
                    });
                    let end_id = self.lower_expr(end);
                    let cmp_op = if *inclusive {
                        BinaryOp::LtEq
                    } else {
                        BinaryOp::Lt
                    };
                    let cmp_id = self.emit(SemInst::Binary {
                        op: cmp_op,
                        lhs: cur_val,
                        rhs: end_id,
                        ty: Type::Bool,
                    });
                    self.emit(SemInst::BranchIf {
                        cond: cmp_id,
                        then_block: body_block,
                        else_block: exit_block,
                    });

                    self.loop_stack.push((step_block, exit_block));

                    // Body block
                    self.current_block = body_block;
                    for s in body {
                        self.lower_stmt(s);
                    }
                    if !self.is_current_terminated() {
                        self.emit(SemInst::Branch {
                            target_block: step_block,
                        });
                    }

                    // Step block: increment var_name
                    self.current_block = step_block;
                    let old_v = self.emit(SemInst::Load {
                        var_name: var_name.clone(),
                        ty: t.clone(),
                    });
                    let one_id = self.emit(SemInst::IntConst {
                        val: 1,
                        ty: t.clone(),
                    });
                    let next_v = self.emit(SemInst::Binary {
                        op: BinaryOp::Add,
                        lhs: old_v,
                        rhs: one_id,
                        ty: t.clone(),
                    });
                    self.emit(SemInst::Store {
                        var_name: var_name.clone(),
                        val: next_v,
                    });
                    self.emit(SemInst::Branch {
                        target_block: cond_block,
                    });

                    self.loop_stack.pop();
                    self.current_block = exit_block;
                }
            }
            Stmt::Break(_) => {
                if let Some(&(_, break_target)) = self.loop_stack.last() {
                    self.emit(SemInst::Branch {
                        target_block: break_target,
                    });
                }
            }
            Stmt::Continue(_) => {
                if let Some(&(cont_target, _)) = self.loop_stack.last() {
                    self.emit(SemInst::Branch {
                        target_block: cont_target,
                    });
                }
            }
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.lower_stmt(s);
                }
            }
        }
    }

    fn lower_expr(&mut self, expr: &Expr) -> InstId {
        match expr {
            Expr::Integer(val, _) => self.emit(SemInst::IntConst {
                val: *val,
                ty: Type::I32,
            }),
            Expr::Float(val, _) => self.emit(SemInst::FloatConst {
                val: *val,
                ty: Type::F64,
            }),
            Expr::Bool(val, _) => self.emit(SemInst::BoolConst { val: *val }),
            Expr::StringLit(val, _) => self.emit(SemInst::StringConst { val: val.clone() }),
            Expr::Ident(name, _) => self.emit(SemInst::Load {
                var_name: name.clone(),
                ty: Type::I32,
            }),
            Expr::Binary {
                op, left, right, ..
            } => {
                let left_id = self.lower_expr(left);
                let right_id = self.lower_expr(right);
                self.emit(SemInst::Binary {
                    op: *op,
                    lhs: left_id,
                    rhs: right_id,
                    ty: Type::I32,
                })
            }
            Expr::Index { array, index, .. } => {
                let idx_id = self.lower_expr(index);
                let arr_name = match &**array {
                    Expr::Ident(name, _) => name.clone(),
                    _ => "anonymous_array".to_string(),
                };
                self.emit(SemInst::LoadIndex {
                    array_name: arr_name,
                    index: idx_id,
                    ty: Type::I32,
                })
            }
            Expr::Unary {
                op, expr: inner, ..
            } => {
                let inner_id = self.lower_expr(inner);
                self.emit(SemInst::Unary {
                    op: *op,
                    operand: inner_id,
                    ty: Type::I32,
                })
            }
            Expr::Assign {
                target, op, value, ..
            } => {
                let val_id = self.lower_expr(value);
                match &**target {
                    Expr::Ident(name, _) => {
                        let bin_op = match op {
                            AssignOp::Assign => None,
                            AssignOp::AddAssign => Some(BinaryOp::Add),
                            AssignOp::SubAssign => Some(BinaryOp::Sub),
                            AssignOp::MulAssign => Some(BinaryOp::Mul),
                            AssignOp::DivAssign => Some(BinaryOp::Div),
                            AssignOp::RemAssign => Some(BinaryOp::Rem),
                            AssignOp::BitAndAssign => Some(BinaryOp::BitAnd),
                            AssignOp::BitOrAssign => Some(BinaryOp::BitOr),
                            AssignOp::BitXorAssign => Some(BinaryOp::BitXor),
                            AssignOp::ShlAssign => Some(BinaryOp::Shl),
                            AssignOp::ShrAssign => Some(BinaryOp::Shr),
                        };

                        let final_val = if let Some(b_op) = bin_op {
                            let load_id = self.emit(SemInst::Load {
                                var_name: name.clone(),
                                ty: Type::I32,
                            });
                            self.emit(SemInst::Binary {
                                op: b_op,
                                lhs: load_id,
                                rhs: val_id,
                                ty: Type::I32,
                            })
                        } else {
                            val_id
                        };

                        self.emit(SemInst::Store {
                            var_name: name.clone(),
                            val: final_val,
                        })
                    }
                    Expr::Index { array, index, .. } => {
                        let idx_id = self.lower_expr(index);
                        let arr_name = match &**array {
                            Expr::Ident(name, _) => name.clone(),
                            _ => "anonymous_array".to_string(),
                        };
                        let bin_op = match op {
                            AssignOp::Assign => None,
                            AssignOp::AddAssign => Some(BinaryOp::Add),
                            AssignOp::SubAssign => Some(BinaryOp::Sub),
                            AssignOp::MulAssign => Some(BinaryOp::Mul),
                            AssignOp::DivAssign => Some(BinaryOp::Div),
                            AssignOp::RemAssign => Some(BinaryOp::Rem),
                            AssignOp::BitAndAssign => Some(BinaryOp::BitAnd),
                            AssignOp::BitOrAssign => Some(BinaryOp::BitOr),
                            AssignOp::BitXorAssign => Some(BinaryOp::BitXor),
                            AssignOp::ShlAssign => Some(BinaryOp::Shl),
                            AssignOp::ShrAssign => Some(BinaryOp::Shr),
                        };

                        let final_val = if let Some(b_op) = bin_op {
                            let load_id = self.emit(SemInst::LoadIndex {
                                array_name: arr_name.clone(),
                                index: idx_id,
                                ty: Type::I32,
                            });
                            self.emit(SemInst::Binary {
                                op: b_op,
                                lhs: load_id,
                                rhs: val_id,
                                ty: Type::I32,
                            })
                        } else {
                            val_id
                        };

                        self.emit(SemInst::StoreIndex {
                            array_name: arr_name,
                            index: idx_id,
                            val: final_val,
                        })
                    }
                    _ => val_id,
                }
            }
            Expr::Call { callee, args, .. } => {
                let mut arg_ids = Vec::new();
                for a in args {
                    arg_ids.push(self.lower_expr(a));
                }
                let name = match &**callee {
                    Expr::Ident(n, _) => n.clone(),
                    _ => "indirect_call".to_string(),
                };
                self.emit(SemInst::Call {
                    callee: name,
                    args: arg_ids,
                    ty: Type::I32,
                })
            }
            _ => self.emit(SemInst::IntConst {
                val: 0,
                ty: Type::I32,
            }),
        }
    }
}

impl SemProgram {
    pub fn dump_sem_ir(&self) -> String {
        let mut out = String::new();
        out.push_str("// Carbon SemIR Representation (Generated by Hexene / Carbonation)\n");
        out.push_str("// Copyright (c) 2026 jk9576. All rights reserved.\n\n");

        for (gname, gty, init) in &self.globals {
            if let Some(v) = init {
                out.push_str(&format!("global var {}: {} = {};\n", gname, gty, v));
            } else {
                out.push_str(&format!("global var {}: {};\n", gname, gty));
            }
        }
        if !self.globals.is_empty() {
            out.push('\n');
        }

        for func in &self.functions {
            let params_str: Vec<String> = func
                .params
                .iter()
                .map(|(n, t)| format!("{}: {}", n, t))
                .collect();
            let ext = if func.is_extern { "extern " } else { "" };
            out.push_str(&format!(
                "{}fn @{}({}) -> {} {{\n",
                ext,
                func.name,
                params_str.join(", "),
                func.return_type
            ));

            if !func.is_extern {
                for (b_idx, block) in func.blocks.iter().enumerate() {
                    out.push_str(&format!("  block{}:\n", b_idx));
                    for &inst_id in &block.instructions {
                        if let Some(inst) = func.instructions.get(inst_id.0) {
                            out.push_str(&format!("    {} = {}\n", inst_id, format_inst(inst)));
                        }
                    }
                }
            }
            out.push_str("}\n\n");
        }

        out
    }
}

fn format_inst(inst: &SemInst) -> String {
    match inst {
        SemInst::IntConst { val, ty } => format!("const.int {} : {}", val, ty),
        SemInst::FloatConst { val, ty } => format!("const.float {} : {}", val, ty),
        SemInst::BoolConst { val } => format!("const.bool {}", val),
        SemInst::StringConst { val } => format!("const.str {:?}", val),
        SemInst::VarStorage { name, ty } => format!("var.storage {} : {}", name, ty),
        SemInst::Load { var_name, ty } => format!("var.load {} : {}", var_name, ty),
        SemInst::Store { var_name, val } => format!("var.store {}, {}", var_name, val),
        SemInst::LoadIndex {
            array_name,
            index,
            ty,
        } => format!("array.load {}[{}] : {}", array_name, index, ty),
        SemInst::StoreIndex {
            array_name,
            index,
            val,
        } => format!("array.store {}[{}], {}", array_name, index, val),
        SemInst::Binary { op, lhs, rhs, ty } => format!("arith.{:?} {}, {} : {}", op, lhs, rhs, ty),
        SemInst::Unary { op, operand, ty } => format!("unary.{:?} {} : {}", op, operand, ty),
        SemInst::Branch { target_block } => format!("br block{}", target_block),
        SemInst::BranchIf {
            cond,
            then_block,
            else_block,
        } => {
            format!("br.cond {}, block{}, block{}", cond, then_block, else_block)
        }
        SemInst::Return { val } => match val {
            Some(v) => format!("ret {}", v),
            None => "ret.void".to_string(),
        },
        SemInst::Call { callee, args, ty } => {
            let args_s: Vec<String> = args.iter().map(|a| a.to_string()).collect();
            format!("call @{}({}) : {}", callee, args_s.join(", "), ty)
        }
        SemInst::VmcsTransition {
            exit_reason,
            guest_rip,
        } => {
            format!(
                "vmcs.transition exit_reason=0x{:x}, rip=0x{:x}",
                exit_reason, guest_rip
            )
        }
        SemInst::EptWalk { gpa, hpa } => {
            format!("ept.walk gpa=0x{:x} -> hpa=0x{:x}", gpa, hpa)
        }
        SemInst::LogUpLookup { table_id, key, val } => {
            format!("lookup.logup table={}, key={}, val={}", table_id, key, val)
        }
    }
}
