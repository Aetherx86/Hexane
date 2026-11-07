// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation / Hexene Language Toolchain
// Licensed under the MIT License.

use carbonation::parse_file_raw;
use carbonation::sem_ir::SemProgram;
use carbonation::vmp_revirt::VmpRevirtualizer;
use carbonation::zkhypervm::{ZkHyperVm, ZkOpcode};
use std::fs;
use std::process::Command;

fn get_bin_path(bin_name: &str) -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push(bin_name);
    path
}

#[test]
fn test_vmp_revirtualization_in_memory() {
    let src = r#"
package RevirtTest api;

fn Main() -> i32 {
  var a: i32 = 25;
  var b: i32 = 17;
  var c: i32 = a + b; // 42
  return c;
}
"#;

    let temp_dir = std::env::temp_dir();
    let carbon_path = temp_dir.join("test_revirt_mem.carbon");
    fs::write(&carbon_path, src).unwrap();

    let (_, program) = parse_file_raw(carbon_path.to_str().unwrap(), false, false).unwrap();
    let sem_prog = SemProgram::from_ast(&program);

    let revirt = VmpRevirtualizer::new(0x9A8B7C6D);
    let pkg = revirt.revirtualize(&sem_prog);

    assert_eq!(pkg.functions.len(), 1);
    let main_fn = &pkg.functions[0];
    assert_eq!(main_fn.name, "Main");

    let result = revirt.execute_in_memory(main_fn);
    assert_eq!(
        result, 42,
        "Revirtualized VM in-memory execution returned incorrect result"
    );

    let _ = fs::remove_file(&carbon_path);
}

#[test]
fn test_vmp_revirtualization_compiled_cff_runner() {
    let src = r#"
package RevirtCff api;

fn Main() -> i32 {
  var x: i32 = 100;
  var y: i32 = 42;
  var diff: i32 = x - y; // 58
  if (diff == 58) {
    return 0;
  } else {
    return 1;
  }
}
"#;

    let temp_dir = std::env::temp_dir();
    let carbon_path = temp_dir.join("test_revirt_cff.carbon");
    let runner_bin = temp_dir.join("test_revirt_runner_app");
    fs::write(&carbon_path, src).unwrap();

    let hexene = get_bin_path("hexene");
    let status = Command::new(&hexene)
        .arg("revirt")
        .arg(&carbon_path)
        .arg("-o")
        .arg(&runner_bin)
        .arg("-O3")
        .status()
        .expect("failed to execute hexene revirt command");

    assert!(status.success(), "hexene revirt compilation failed");

    let run_status = Command::new(&runner_bin)
        .status()
        .expect("failed to run revirtualized CFF runner");

    assert_eq!(
        run_status.code(),
        Some(0),
        "revirtualized binary exited with failure code"
    );

    let _ = fs::remove_file(&carbon_path);
    let _ = fs::remove_file(&runner_bin);
}

#[test]
fn test_vmp_revirtualization_loop_in_memory_and_cff() {
    let src = r#"
package RevirtLoop api;

fn Main() -> i32 {
  var sum: i32 = 0;
  var i: i32 = 1;
  while (i <= 10) {
    sum += i;
    i += 1;
  }
  return sum; // 55
}
"#;

    let temp_dir = std::env::temp_dir();
    let carbon_path = temp_dir.join("test_revirt_loop.carbon");
    let runner_bin = temp_dir.join("test_revirt_loop_bin");
    fs::write(&carbon_path, src).unwrap();

    let (_, program) = parse_file_raw(carbon_path.to_str().unwrap(), false, false).unwrap();
    let sem_prog = SemProgram::from_ast(&program);

    let revirt = VmpRevirtualizer::new(0x1337BEEF);
    let pkg = revirt.revirtualize(&sem_prog);
    let main_fn = &pkg.functions[0];

    // In-memory execution
    let result = revirt.execute_in_memory(main_fn);
    assert_eq!(result, 55, "VMP loop in-memory execution failed");

    // Compiled CFF runner execution
    let hexane = get_bin_path("hexane");
    let status = Command::new(&hexane)
        .arg("revirt")
        .arg(&carbon_path)
        .arg("-o")
        .arg(&runner_bin)
        .arg("-O2")
        .status()
        .expect("failed to execute hexane revirt command");

    assert!(status.success(), "hexane revirt compilation failed");

    let run_status = Command::new(&runner_bin)
        .status()
        .expect("failed to run revirtualized CFF runner");

    assert_eq!(
        run_status.code(),
        Some(55),
        "revirtualized loop exited with wrong code"
    );

    let _ = fs::remove_file(&carbon_path);
    let _ = fs::remove_file(&runner_bin);
}

#[test]
fn test_zkhypervm_control_flow_and_loop_verification() {
    let src = r#"
package ZkLoopTest api;

fn Main() -> i32 {
  var total: i32 = 0;
  var count: i32 = 1;
  while (count <= 5) {
    total += count;
    count += 1;
  }
  return total; // 15
}
"#;

    let temp_dir = std::env::temp_dir();
    let carbon_path = temp_dir.join("test_zk_loop.carbon");
    fs::write(&carbon_path, src).unwrap();

    let (_, program) = parse_file_raw(carbon_path.to_str().unwrap(), false, false).unwrap();
    let sem_prog = SemProgram::from_ast(&program);

    let mut zk_vm = ZkHyperVm::new();
    zk_vm.compile_from_sem_ir(&sem_prog);

    let res = zk_vm.execute_and_trace();
    assert_eq!(
        res.final_return, 15,
        "zk-HyperVM loop execution returned wrong value"
    );
    assert!(
        res.is_verified,
        "zk-HyperVM loop trace constraint verification failed"
    );
    assert!(!res.state_commitment.is_empty());
    assert!(res.total_cycles > 20);

    let _ = fs::remove_file(&carbon_path);
}

#[test]
fn test_zkhypervm_execution_and_constraint_verification() {
    let mut zk_vm = ZkHyperVm::new();

    // R0 = 15
    zk_vm.push_op(ZkOpcode::LoadImm(0, 15));
    // R1 = 27
    zk_vm.push_op(ZkOpcode::LoadImm(1, 27));
    // R0 = R0 + R1 = 42
    zk_vm.push_op(ZkOpcode::Add(0, 0, 1));
    // Store R0 to memory address 20
    zk_vm.push_op(ZkOpcode::LoadImm(2, 20));
    zk_vm.push_op(ZkOpcode::StoreMem(2, 0));
    // Load from memory address 20 to R3
    zk_vm.push_op(ZkOpcode::LoadMem(3, 2));
    // Hardware virtualization transition: VMExit (reason 0x28, guest_rip 0x8000)
    zk_vm.push_op(ZkOpcode::VmcsExit(0x28, 0x8000));
    // 2D-EPT translation: GPA 0x1000 -> HPA 0xF000
    zk_vm.push_op(ZkOpcode::EptTranslate(0x1000, 0xF000));
    // LogUp lookup table argument
    zk_vm.push_op(ZkOpcode::LogUpLookup(1, 42, 100));
    // Halt
    zk_vm.push_op(ZkOpcode::Halt);

    let res = zk_vm.execute_and_trace();

    assert_eq!(res.final_return, 42);
    assert!(res.is_verified, "zk-HyperVM constraint verification failed");
    assert!(!res.state_commitment.is_empty());
    assert!(res.trace.len() >= 10);

    // Verify trace properties
    let last = res.trace.last().unwrap();
    assert_eq!(last.opcode, "HALT");
}

#[test]
fn test_zkhypervm_cli_with_trace_export() {
    let src = r#"
package ZkCliTest api;

fn Main() -> i32 {
  var p: i32 = 7;
  var q: i32 = 6;
  var prod: i32 = p * q; // 42
  return prod;
}
"#;

    let temp_dir = std::env::temp_dir();
    let carbon_path = temp_dir.join("test_zk_cli.carbon");
    let trace_path = temp_dir.join("test_zk_trace.json");
    fs::write(&carbon_path, src).unwrap();

    let carbonation = get_bin_path("carbonation");
    let status = Command::new(&carbonation)
        .arg("zkhypervm")
        .arg(&carbon_path)
        .arg("-o")
        .arg(&trace_path)
        .status()
        .expect("failed to run carbonation zkhypervm");

    assert!(status.success(), "carbonation zkhypervm failed");
    assert!(trace_path.exists(), "trace json file was not generated");

    let trace_content = fs::read_to_string(&trace_path).unwrap();
    assert!(trace_content.contains("\"verified\": true"));

    let _ = fs::remove_file(&carbon_path);
    let _ = fs::remove_file(&trace_path);
}

#[test]
fn test_sem_ir_dump_and_lowering() {
    let src = r#"
package SemIrTest api;

var GlobalScore: i32 = 100;

fn Compute(x: i32) -> i32 {
  var total: i32 = x + GlobalScore;
  return total;
}
"#;

    let temp_dir = std::env::temp_dir();
    let carbon_path = temp_dir.join("test_sem_ir.carbon");
    fs::write(&carbon_path, src).unwrap();

    let graphene = get_bin_path("graphene");
    let output = Command::new(&graphene)
        .arg("sem-ir")
        .arg(&carbon_path)
        .output()
        .expect("failed to run graphene sem-ir");

    assert!(output.status.success());
    let sem_ir_str = String::from_utf8_lossy(&output.stdout);

    assert!(sem_ir_str.contains("global var GlobalScore: i32 = 100;"));
    assert!(sem_ir_str.contains("fn @Compute"));
    assert!(sem_ir_str.contains("arith.Add"));
    assert!(sem_ir_str.contains("ret"));

    let _ = fs::remove_file(&carbon_path);
}

#[test]
fn test_formatter_cli() {
    let unformatted = r#"
package   MessyCode   api ;
var   a : i32 = 10 ;
fn   Foo ( x : i32 ) -> i32 {
var y:i32=x+a;
return y ;
}
"#;

    let temp_dir = std::env::temp_dir();
    let carbon_path = temp_dir.join("test_unformatted.carbon");
    let formatted_path = temp_dir.join("test_formatted.carbon");
    fs::write(&carbon_path, unformatted).unwrap();

    let hexane = get_bin_path("hexane");
    let status = Command::new(&hexane)
        .arg("format")
        .arg(&carbon_path)
        .arg("-o")
        .arg(&formatted_path)
        .status()
        .expect("failed to run hexane format");

    assert!(status.success());
    let formatted = fs::read_to_string(&formatted_path).unwrap();

    assert!(formatted.contains("package MessyCode api;"));
    assert!(formatted.contains("var a: i32 = 10;"));
    assert!(formatted.contains("fn Foo(x: i32) -> i32 {"));
    assert!(formatted.contains("  var y: i32 = x + a;"));
    assert!(formatted.contains("  return y;"));

    let _ = fs::remove_file(&carbon_path);
    let _ = fs::remove_file(&formatted_path);
}

#[test]
fn test_for_loop_sem_ir_and_revirt() {
    let src = r#"
package ForTest api;

fn Main() -> i32 {
  var acc: i32 = 0;
  for (var i: i32 in Core.Range(1, 5)) {
    acc += i;
  }
  return acc; // 1 + 2 + 3 + 4 = 10
}
"#;

    let temp_dir = std::env::temp_dir();
    let carbon_path = temp_dir.join("test_for_sem_ir.carbon");
    fs::write(&carbon_path, src).unwrap();

    let (_, program) = parse_file_raw(carbon_path.to_str().unwrap(), false, false).unwrap();
    let sem_prog = SemProgram::from_ast(&program);

    let revirt = VmpRevirtualizer::new(0xCAFEBABE);
    let pkg = revirt.revirtualize(&sem_prog);
    let main_fn = &pkg.functions[0];

    let res = revirt.execute_in_memory(main_fn);
    assert_eq!(
        res, 10,
        "For-loop in-memory revirt execution returned wrong result"
    );

    let _ = fs::remove_file(&carbon_path);
}
