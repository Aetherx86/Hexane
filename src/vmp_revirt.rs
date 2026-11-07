// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation / Hexene Language Toolchain
// Licensed under the MIT License.

//! VMP Re-Virtualizer (`vmp_revirt`)
//! Re-virtualizes Carbon code into a hardened, encrypted bytecode machine
//! with an affine state-machine dispatcher, rolling opcode decryption,
//! and Control Flow Flattening (CFF).

use crate::sem_ir::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VmpOp {
    Nop = 0x10,
    PushImm = 0x11,
    Pop = 0x12,
    LoadVar = 0x13,
    StoreVar = 0x14,
    LoadIndex = 0x15,
    StoreIndex = 0x16,
    Add = 0x20,
    Sub = 0x21,
    Mul = 0x22,
    Div = 0x23,
    Mod = 0x24,
    Xor = 0x25,
    And = 0x26,
    Or = 0x27,
    Shl = 0x28,
    Shr = 0x29,
    Neg = 0x2A,
    CmpEq = 0x30,
    CmpLt = 0x31,
    CmpGt = 0x32,
    CmpLe = 0x33,
    CmpGe = 0x34,
    CmpNe = 0x35,
    Jmp = 0x40,
    Jz = 0x41,
    Jnz = 0x42,
    Call = 0x50,
    Ret = 0x51,
    Halt = 0xFF,
}

#[derive(Debug, Clone)]
pub struct VmpInstruction {
    pub op: VmpOp,
    pub operand: i64,
}

#[derive(Debug, Clone)]
pub struct VmpFunction {
    pub name: String,
    pub param_count: usize,
    pub bytecode: Vec<VmpInstruction>,
}

#[derive(Debug, Clone)]
pub struct VmpPackage {
    pub seed_key: u32,
    pub functions: Vec<VmpFunction>,
    pub var_map: HashMap<String, usize>,
}

pub struct VmpRevirtualizer {
    seed_key: u32,
}

fn get_inst_key(seed: u32, pc: usize) -> u8 {
    let step = (pc as u32).wrapping_mul(214013).wrapping_add(2531011);
    ((seed ^ step) & 0xFF) as u8
}

impl VmpRevirtualizer {
    pub fn new(seed_key: u32) -> Self {
        Self { seed_key }
    }

    pub fn revirtualize(&self, sem_prog: &SemProgram) -> VmpPackage {
        let mut package = VmpPackage {
            seed_key: self.seed_key,
            functions: Vec::new(),
            var_map: HashMap::new(),
        };

        let mut fn_indices: HashMap<String, usize> = HashMap::new();
        for (i, f) in sem_prog
            .functions
            .iter()
            .filter(|f| !f.is_extern)
            .enumerate()
        {
            fn_indices.insert(f.name.clone(), i);
        }

        for func in &sem_prog.functions {
            if func.is_extern {
                continue;
            }
            let mut vmp_fn = VmpFunction {
                name: func.name.clone(),
                param_count: func.params.len(),
                bytecode: Vec::new(),
            };

            let mut var_slots: HashMap<String, usize> = HashMap::new();
            for (idx, (pname, _)) in func.params.iter().enumerate() {
                var_slots.insert(pname.clone(), idx * 16);
            }

            let mut block_start_pc: HashMap<usize, usize> = HashMap::new();
            let mut branch_fixups: Vec<(usize, usize)> = Vec::new(); // (bytecode_idx, target_block)

            for block in &func.blocks {
                block_start_pc.insert(block.id, vmp_fn.bytecode.len());
                for &inst_id in &block.instructions {
                    if let Some(inst) = func.instructions.get(inst_id.0) {
                        match inst {
                            SemInst::IntConst { val, .. } => {
                                vmp_fn.bytecode.push(VmpInstruction {
                                    op: VmpOp::PushImm,
                                    operand: *val,
                                });
                            }
                            SemInst::BoolConst { val } => {
                                vmp_fn.bytecode.push(VmpInstruction {
                                    op: VmpOp::PushImm,
                                    operand: if *val { 1 } else { 0 },
                                });
                            }
                            SemInst::VarStorage { name, .. } => {
                                if !var_slots.contains_key(name) {
                                    let slot = var_slots.len() * 16;
                                    var_slots.insert(name.clone(), slot);
                                }
                            }
                            SemInst::Store { var_name, .. } => {
                                let next_slot = var_slots.len() * 16;
                                let slot = *var_slots.entry(var_name.clone()).or_insert(next_slot);
                                vmp_fn.bytecode.push(VmpInstruction {
                                    op: VmpOp::StoreVar,
                                    operand: slot as i64,
                                });
                            }
                            SemInst::Load { var_name, .. } => {
                                let next_slot = var_slots.len() * 16;
                                let slot = *var_slots.entry(var_name.clone()).or_insert(next_slot);
                                vmp_fn.bytecode.push(VmpInstruction {
                                    op: VmpOp::LoadVar,
                                    operand: slot as i64,
                                });
                            }
                            SemInst::LoadIndex { array_name, .. } => {
                                let next_slot = var_slots.len() * 16;
                                let slot =
                                    *var_slots.entry(array_name.clone()).or_insert(next_slot);
                                vmp_fn.bytecode.push(VmpInstruction {
                                    op: VmpOp::LoadIndex,
                                    operand: slot as i64,
                                });
                            }
                            SemInst::StoreIndex { array_name, .. } => {
                                let next_slot = var_slots.len() * 16;
                                let slot =
                                    *var_slots.entry(array_name.clone()).or_insert(next_slot);
                                vmp_fn.bytecode.push(VmpInstruction {
                                    op: VmpOp::StoreIndex,
                                    operand: slot as i64,
                                });
                            }
                            SemInst::Unary { op, .. } => {
                                if op == &crate::ast::UnaryOp::Neg {
                                    vmp_fn.bytecode.push(VmpInstruction {
                                        op: VmpOp::Neg,
                                        operand: 0,
                                    });
                                }
                            }
                            SemInst::Call { callee, .. } => {
                                let callee_idx = *fn_indices.get(callee).unwrap_or(&0);
                                vmp_fn.bytecode.push(VmpInstruction {
                                    op: VmpOp::Call,
                                    operand: callee_idx as i64,
                                });
                            }
                            SemInst::Binary { op, .. } => {
                                let v_op = match op {
                                    crate::ast::BinaryOp::Add => VmpOp::Add,
                                    crate::ast::BinaryOp::Sub => VmpOp::Sub,
                                    crate::ast::BinaryOp::Mul => VmpOp::Mul,
                                    crate::ast::BinaryOp::Div => VmpOp::Div,
                                    crate::ast::BinaryOp::Rem => VmpOp::Mod,
                                    crate::ast::BinaryOp::BitXor => VmpOp::Xor,
                                    crate::ast::BinaryOp::BitAnd => VmpOp::And,
                                    crate::ast::BinaryOp::BitOr => VmpOp::Or,
                                    crate::ast::BinaryOp::Shl => VmpOp::Shl,
                                    crate::ast::BinaryOp::Shr => VmpOp::Shr,
                                    crate::ast::BinaryOp::Eq => VmpOp::CmpEq,
                                    crate::ast::BinaryOp::Lt => VmpOp::CmpLt,
                                    crate::ast::BinaryOp::Gt => VmpOp::CmpGt,
                                    crate::ast::BinaryOp::LtEq => VmpOp::CmpLe,
                                    crate::ast::BinaryOp::GtEq => VmpOp::CmpGe,
                                    crate::ast::BinaryOp::NotEq => VmpOp::CmpNe,
                                    _ => VmpOp::Nop,
                                };
                                vmp_fn.bytecode.push(VmpInstruction {
                                    op: v_op,
                                    operand: 0,
                                });
                            }
                            SemInst::Branch { target_block } => {
                                let idx = vmp_fn.bytecode.len();
                                vmp_fn.bytecode.push(VmpInstruction {
                                    op: VmpOp::Jmp,
                                    operand: 0,
                                });
                                branch_fixups.push((idx, *target_block));
                            }
                            SemInst::BranchIf {
                                then_block,
                                else_block,
                                ..
                            } => {
                                let jz_idx = vmp_fn.bytecode.len();
                                vmp_fn.bytecode.push(VmpInstruction {
                                    op: VmpOp::Jz,
                                    operand: 0,
                                });
                                branch_fixups.push((jz_idx, *else_block));

                                let jmp_idx = vmp_fn.bytecode.len();
                                vmp_fn.bytecode.push(VmpInstruction {
                                    op: VmpOp::Jmp,
                                    operand: 0,
                                });
                                branch_fixups.push((jmp_idx, *then_block));
                            }
                            SemInst::Return { .. } => {
                                vmp_fn.bytecode.push(VmpInstruction {
                                    op: VmpOp::Ret,
                                    operand: 0,
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }

            let halt_pc = vmp_fn.bytecode.len();
            vmp_fn.bytecode.push(VmpInstruction {
                op: VmpOp::Halt,
                operand: 0,
            });

            for (idx, target_block) in branch_fixups {
                let target_pc = *block_start_pc.get(&target_block).unwrap_or(&halt_pc);
                vmp_fn.bytecode[idx].operand = target_pc as i64;
            }

            package.functions.push(vmp_fn);
        }

        package
    }

    /// Emits a self-contained, Control-Flow-Flattened C source code
    /// running the virtual machine with rolling opcode decryption.
    pub fn emit_c_vm_runner(&self, package: &VmpPackage) -> String {
        let mut c = String::new();
        c.push_str("// VMP Re-Virtualization Runner (Generated by Hexene / Carbonation)\n");
        c.push_str("// Copyright (c) 2026 jk9576. All rights reserved.\n");
        c.push_str("#include <stdio.h>\n#include <stdint.h>\n#include <stdlib.h>\n\n");

        c.push_str(&format!(
            "static uint32_t VMP_ROLLING_KEY = 0x{:08X};\n\n",
            package.seed_key
        ));

        // Bytecode definition
        for func in &package.functions {
            c.push_str(&format!(
                "static const int64_t VMP_CODE_{}[] = {{\n",
                func.name
            ));
            for (pc, inst) in func.bytecode.iter().enumerate() {
                let enc_op = (inst.op as u32) ^ (get_inst_key(package.seed_key, pc) as u32);
                c.push_str(&format!("    0x{:02X}, {}, \n", enc_op, inst.operand));
            }
            c.push_str("};\n\n");
        }

        // Forward declarations
        c.push_str("static int64_t vmp_execute(const int64_t* code, int64_t code_len, const int64_t* args, int num_args);\n");
        c.push_str(
            "static int64_t vmp_dispatch_call(int func_idx, const int64_t* args, int num_args);\n",
        );
        c.push_str("static int vmp_get_arg_count(int func_idx);\n\n");

        // Helper: vmp_get_arg_count
        c.push_str("static int vmp_get_arg_count(int func_idx) {\n    switch (func_idx) {\n");
        for (i, func) in package.functions.iter().enumerate() {
            c.push_str(&format!(
                "        case {}: return {};\n",
                i, func.param_count
            ));
        }
        c.push_str("        default: return 0;\n    }\n}\n\n");

        // Helper: vmp_dispatch_call
        c.push_str("static int64_t vmp_dispatch_call(int func_idx, const int64_t* args, int num_args) {\n    switch (func_idx) {\n");
        for (i, func) in package.functions.iter().enumerate() {
            c.push_str(&format!(
                "        case {}: return vmp_execute(VMP_CODE_{}, sizeof(VMP_CODE_{}) / sizeof(int64_t), args, num_args);\n",
                i, func.name, func.name
            ));
        }
        c.push_str("        default: return 0;\n    }\n}\n\n");

        // Virtual Machine Dispatcher Engine
        c.push_str(r#"
int64_t vmp_execute(const int64_t* code, int64_t code_len, const int64_t* args, int num_args) {
    int64_t stack[1024];
    int64_t vars[1024] = {0};
    int sp = 0;
    int pc = 0;
    int64_t ret_val = 0;

    if (args != NULL && num_args > 0) {
        for (int i = 0; i < num_args && i < 64; i++) {
            vars[i * 16] = args[i];
        }
    }

    // Control Flow Flattening State Variable
    uint32_t state = 0x1000;

    while (state != 0x9999) {
        // Opaque Invariant Predicate: ((VMP_ROLLING_KEY | 1) > 0) is universally true
        if (!((VMP_ROLLING_KEY | 1) > 0)) {
            return 0xDEADBEEF;
        }

        switch (state) {
            case 0x1000: { // Fetch & Decrypt Opcode
                if (pc * 2 >= code_len) {
                    state = 0x9999;
                    break;
                }
                uint8_t inst_key = (uint8_t)(((VMP_ROLLING_KEY ^ (((uint32_t)pc * 214013 + 2531011)))) & 0xFF);
                uint8_t raw_op = (uint8_t)(code[pc * 2] ^ inst_key);
                int64_t operand = code[pc * 2 + 1];
                pc++;

                switch (raw_op) {
                    case 0x11: // PushImm
                        stack[sp++] = operand;
                        break;
                    case 0x12: // Pop
                        if (sp > 0) sp--;
                        break;
                    case 0x13: // LoadVar
                        stack[sp++] = vars[operand];
                        break;
                    case 0x14: // StoreVar
                        if (sp > 0) vars[operand] = stack[--sp];
                        break;
                    case 0x15: // LoadIndex
                        if (sp > 0) {
                            int64_t idx = stack[--sp];
                            int slot = (int)(operand + idx);
                            stack[sp++] = (slot >= 0 && slot < 1024) ? vars[slot] : 0;
                        }
                        break;
                    case 0x16: // StoreIndex
                        if (sp >= 2) {
                            int64_t idx = stack[--sp];
                            int64_t val = stack[--sp];
                            int slot = (int)(operand + idx);
                            if (slot >= 0 && slot < 1024) vars[slot] = val;
                        }
                        break;
                    case 0x20: // Add
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = a + b; }
                        break;
                    case 0x21: // Sub
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = a - b; }
                        break;
                    case 0x22: // Mul
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = a * b; }
                        break;
                    case 0x23: // Div
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = (b != 0 ? a / b : 0); }
                        break;
                    case 0x24: // Mod
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = (b != 0 ? a % b : 0); }
                        break;
                    case 0x25: // Xor
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = a ^ b; }
                        break;
                    case 0x26: // And
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = a & b; }
                        break;
                    case 0x27: // Or
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = a | b; }
                        break;
                    case 0x28: // Shl
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = a << (b & 63); }
                        break;
                    case 0x29: // Shr
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = a >> (b & 63); }
                        break;
                    case 0x2A: // Neg
                        if (sp > 0) stack[sp - 1] = -stack[sp - 1];
                        break;
                    case 0x30: // CmpEq
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = (a == b); }
                        break;
                    case 0x31: // CmpLt
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = (a < b); }
                        break;
                    case 0x32: // CmpGt
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = (a > b); }
                        break;
                    case 0x33: // CmpLe
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = (a <= b); }
                        break;
                    case 0x34: // CmpGe
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = (a >= b); }
                        break;
                    case 0x35: // CmpNe
                        if (sp >= 2) { int64_t b = stack[--sp]; int64_t a = stack[--sp]; stack[sp++] = (a != b); }
                        break;
                    case 0x40: // Jmp
                        pc = (int)operand;
                        break;
                    case 0x41: // Jz
                        if (sp > 0) {
                            int64_t cond = stack[--sp];
                            if (cond == 0) {
                                pc = (int)operand;
                            }
                        }
                        break;
                    case 0x42: // Jnz
                        if (sp > 0) {
                            int64_t cond = stack[--sp];
                            if (cond != 0) {
                                pc = (int)operand;
                            }
                        }
                        break;
                    case 0x50: { // Call
                        int callee_idx = (int)operand;
                        int arg_count = vmp_get_arg_count(callee_idx);
                        int64_t call_args[16] = {0};
                        for (int a = arg_count - 1; a >= 0; a--) {
                            call_args[a] = (sp > 0) ? stack[--sp] : 0;
                        }
                        int64_t res = vmp_dispatch_call(callee_idx, call_args, arg_count);
                        stack[sp++] = res;
                        break;
                    }
                    case 0x51: // Ret
                        if (sp > 0) ret_val = stack[--sp];
                        state = 0x9999;
                        break;
                    case 0xFF: // Halt
                        if (sp > 0) ret_val = stack[--sp];
                        state = 0x9999;
                        break;
                    default:
                        break;
                }
                break;
            }
            default:
                state = 0x9999;
                break;
        }
    }

    return ret_val;
}
"#);

        // Main wrapper
        c.push_str(
            r#"
int main(int argc, char** argv) {
    (void)argc; (void)argv;
"#,
        );
        let entry_idx = package
            .functions
            .iter()
            .position(|f| f.name == "Main" || f.name == "Run")
            .unwrap_or(0);
        c.push_str(&format!(
            "    int64_t res = vmp_dispatch_call({}, NULL, 0);\n",
            entry_idx
        ));
        c.push_str("    return (int)res;\n");
        c.push_str("}\n");

        c
    }

    /// Fast memory-safe in-process emulator of revirtualized bytecode
    pub fn execute_in_memory(&self, vmp_fn: &VmpFunction) -> i64 {
        let pkg = VmpPackage {
            seed_key: self.seed_key,
            functions: vec![vmp_fn.clone()],
            var_map: HashMap::new(),
        };
        self.execute_package(&pkg, &vmp_fn.name)
    }

    pub fn execute_package(&self, package: &VmpPackage, entry_name: &str) -> i64 {
        let entry_idx = package
            .functions
            .iter()
            .position(|f| f.name == entry_name)
            .or_else(|| {
                package
                    .functions
                    .iter()
                    .position(|f| f.name == "Main" || f.name == "Run")
            })
            .unwrap_or(0);
        self.execute_internal(package, entry_idx, &[])
    }

    fn execute_internal(&self, package: &VmpPackage, fn_idx: usize, args: &[i64]) -> i64 {
        let vmp_fn = match package.functions.get(fn_idx) {
            Some(f) => f,
            None => return 0,
        };

        let mut stack: Vec<i64> = Vec::new();
        let mut vars: HashMap<usize, i64> = HashMap::new();
        for (i, &arg) in args.iter().enumerate() {
            vars.insert(i * 16, arg);
        }

        let mut pc = 0;
        let mut cycles = 0;
        let mut ret_val = 0;

        while pc < vmp_fn.bytecode.len() && cycles < 10_000_000 {
            let inst = &vmp_fn.bytecode[pc];
            pc += 1;
            cycles += 1;

            match inst.op {
                VmpOp::PushImm => stack.push(inst.operand),
                VmpOp::Pop => {
                    let _ = stack.pop();
                }
                VmpOp::LoadVar => {
                    let val = *vars.get(&(inst.operand as usize)).unwrap_or(&0);
                    stack.push(val);
                }
                VmpOp::StoreVar => {
                    if let Some(v) = stack.pop() {
                        vars.insert(inst.operand as usize, v);
                    }
                }
                VmpOp::LoadIndex => {
                    let idx = stack.pop().unwrap_or(0);
                    let slot = (inst.operand + idx) as usize;
                    let val = *vars.get(&slot).unwrap_or(&0);
                    stack.push(val);
                }
                VmpOp::StoreIndex => {
                    let idx = stack.pop().unwrap_or(0);
                    let val = stack.pop().unwrap_or(0);
                    let slot = (inst.operand + idx) as usize;
                    vars.insert(slot, val);
                }
                VmpOp::Add => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a.wrapping_add(b));
                    }
                }
                VmpOp::Sub => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a.wrapping_sub(b));
                    }
                }
                VmpOp::Mul => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a.wrapping_mul(b));
                    }
                }
                VmpOp::Div => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        if b != 0 {
                            stack.push(a / b);
                        } else {
                            stack.push(0);
                        }
                    }
                }
                VmpOp::Mod => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        if b != 0 {
                            stack.push(a % b);
                        } else {
                            stack.push(0);
                        }
                    }
                }
                VmpOp::Xor => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a ^ b);
                    }
                }
                VmpOp::And => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a & b);
                    }
                }
                VmpOp::Or => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a | b);
                    }
                }
                VmpOp::Shl => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a << (b & 63));
                    }
                }
                VmpOp::Shr => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a >> (b & 63));
                    }
                }
                VmpOp::Neg => {
                    if let Some(v) = stack.last_mut() {
                        *v = -*v;
                    }
                }
                VmpOp::CmpEq => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(if a == b { 1 } else { 0 });
                    }
                }
                VmpOp::CmpLt => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(if a < b { 1 } else { 0 });
                    }
                }
                VmpOp::CmpGt => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(if a > b { 1 } else { 0 });
                    }
                }
                VmpOp::CmpLe => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(if a <= b { 1 } else { 0 });
                    }
                }
                VmpOp::CmpGe => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(if a >= b { 1 } else { 0 });
                    }
                }
                VmpOp::CmpNe => {
                    if stack.len() >= 2 {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(if a != b { 1 } else { 0 });
                    }
                }
                VmpOp::Jmp => {
                    pc = inst.operand as usize;
                }
                VmpOp::Jz => {
                    let cond = stack.pop().unwrap_or(0);
                    if cond == 0 {
                        pc = inst.operand as usize;
                    }
                }
                VmpOp::Jnz => {
                    let cond = stack.pop().unwrap_or(0);
                    if cond != 0 {
                        pc = inst.operand as usize;
                    }
                }
                VmpOp::Call => {
                    let callee_idx = inst.operand as usize;
                    let param_count = package
                        .functions
                        .get(callee_idx)
                        .map(|f| f.param_count)
                        .unwrap_or(0);
                    let mut call_args = Vec::new();
                    for _ in 0..param_count {
                        call_args.push(stack.pop().unwrap_or(0));
                    }
                    call_args.reverse();
                    let res = self.execute_internal(package, callee_idx, &call_args);
                    stack.push(res);
                }
                VmpOp::Ret => {
                    if let Some(v) = stack.pop() {
                        ret_val = v;
                    }
                    break;
                }
                VmpOp::Halt => {
                    if let Some(v) = stack.pop() {
                        ret_val = v;
                    }
                    break;
                }
                _ => {}
            }
        }

        ret_val
    }
}
