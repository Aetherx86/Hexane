// Copyright (c) 2026 jk9576. All rights reserved.
// Hexane / Carbonation Language Toolchain
// The Full Queen Suite (N-Queens Benchmark across all toolchain subsystems)

use carbonation::parse_file_raw;
use carbonation::sem_ir::SemProgram;
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
fn test_queen_interpreter() {
    let carbon_file = "examples/n_queens.carbon";
    assert!(
        std::path::Path::new(carbon_file).exists(),
        "examples/n_queens.carbon must exist"
    );

    let hexane = get_bin_path("hexane");
    let output = Command::new(&hexane)
        .arg("run")
        .arg(carbon_file)
        .output()
        .expect("failed to run hexane run on n_queens.carbon");

    assert!(
        output.status.success(),
        "Interpreter execution of N-Queens failed: {:?}",
        output
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_queen_translator_and_native_c() {
    let carbon_file = "examples/n_queens.carbon";
    let temp_dir = std::env::temp_dir();
    let c_file = temp_dir.join("queen_translated.c");
    let bin_file = temp_dir.join("queen_c_bin");

    let hexene = get_bin_path("hexene");
    let status = Command::new(&hexene)
        .arg("translate")
        .arg(carbon_file)
        .arg("-o")
        .arg(&c_file)
        .status()
        .expect("failed to execute hexene translate");
    assert!(status.success(), "translate failed");
    assert!(c_file.exists());

    let clang_status = Command::new("clang")
        .arg(&c_file)
        .arg("-o")
        .arg(&bin_file)
        .arg("-O3")
        .status()
        .expect("failed to compile translated C with clang");
    assert!(clang_status.success());

    let run_status = Command::new(&bin_file)
        .status()
        .expect("failed to run compiled C binary");
    assert_eq!(run_status.code(), Some(0));

    let _ = fs::remove_file(&c_file);
    let _ = fs::remove_file(&bin_file);
}

#[test]
fn test_queen_native_compiler() {
    let carbon_file = "examples/n_queens.carbon";
    let temp_dir = std::env::temp_dir();
    let bin_file = temp_dir.join("queen_native_bin");

    let graphene = get_bin_path("graphene");
    let status = Command::new(&graphene)
        .arg("compile")
        .arg(carbon_file)
        .arg("-o")
        .arg(&bin_file)
        .arg("-O3")
        .status()
        .expect("failed to compile n_queens with graphene");

    assert!(status.success(), "compiler failed");
    assert!(bin_file.exists());

    let run_status = Command::new(&bin_file)
        .status()
        .expect("failed to run native n_queens executable");
    assert_eq!(run_status.code(), Some(0));

    let _ = fs::remove_file(&bin_file);
}

#[test]
fn test_queen_assembler_pipeline() {
    let carbon_file = "examples/n_queens.carbon";
    let temp_dir = std::env::temp_dir();
    let s_file = temp_dir.join("queen_asm.s");
    let o_file = temp_dir.join("queen_obj.o");
    let bin_file = temp_dir.join("queen_linked_bin");

    let carbonation = get_bin_path("carbonation");

    // Assemble to .s
    let status_s = Command::new(&carbonation)
        .arg("assemble")
        .arg(carbon_file)
        .arg("-o")
        .arg(&s_file)
        .arg("-O3")
        .status()
        .expect("failed to assemble n_queens to .s");
    assert!(status_s.success());
    assert!(s_file.exists());

    // Assemble to .o
    let status_o = Command::new(&carbonation)
        .arg("assemble")
        .arg(carbon_file)
        .arg("-o")
        .arg(&o_file)
        .arg("-O3")
        .status()
        .expect("failed to assemble n_queens to .o");
    assert!(status_o.success());
    assert!(o_file.exists());

    // Link and execute
    let link_status = Command::new("clang")
        .arg(&o_file)
        .arg("-o")
        .arg(&bin_file)
        .status()
        .expect("failed to link object file");
    assert!(link_status.success());

    let run_status = Command::new(&bin_file)
        .status()
        .expect("failed to execute linked binary");
    assert_eq!(run_status.code(), Some(0));

    let _ = fs::remove_file(&s_file);
    let _ = fs::remove_file(&o_file);
    let _ = fs::remove_file(&bin_file);
}

#[test]
fn test_queen_sem_ir_lowering() {
    let carbon_file = "examples/n_queens.carbon";
    let (_, program) = parse_file_raw(carbon_file, false, false).unwrap();
    let sem_prog = SemProgram::from_ast(&program);

    assert_eq!(sem_prog.functions.len(), 3);
    let dump = sem_prog.dump_sem_ir();
    assert!(dump.contains("fn @Abs"));
    assert!(dump.contains("fn @SolveQueens"));
    assert!(dump.contains("fn @Main"));
    assert!(dump.contains("array.load"));
    assert!(dump.contains("array.store"));
    assert!(dump.contains("br.cond"));
    assert!(dump.contains("ret"));
}

#[test]
fn test_queen_vmp_revirt_cff_runner() {
    let carbon_file = "examples/n_queens.carbon";
    let temp_dir = std::env::temp_dir();
    let runner_bin = temp_dir.join("queen_vmp_cff_bin");

    let hexane = get_bin_path("hexane");
    let status = Command::new(&hexane)
        .arg("revirt")
        .arg(carbon_file)
        .arg("-o")
        .arg(&runner_bin)
        .arg("-O3")
        .status()
        .expect("failed to revirt n_queens");

    assert!(status.success(), "VMP re-virtualization compilation failed");
    assert!(runner_bin.exists());

    let run_status = Command::new(&runner_bin)
        .status()
        .expect("failed to run VMP CFF runner");
    assert_eq!(run_status.code(), Some(0));

    let _ = fs::remove_file(&runner_bin);
}

#[test]
fn test_queen_formatter() {
    let carbon_file = "examples/n_queens.carbon";
    let temp_dir = std::env::temp_dir();
    let formatted_file = temp_dir.join("queen_fmt.carbon");

    let hexene = get_bin_path("hexene");
    let status = Command::new(&hexene)
        .arg("format")
        .arg(carbon_file)
        .arg("-o")
        .arg(&formatted_file)
        .status()
        .expect("failed to format n_queens");

    assert!(status.success());
    let formatted = fs::read_to_string(&formatted_file).unwrap();
    assert!(formatted.contains("fn Abs(x: i32) -> i32 {"));
    assert!(formatted.contains("fn SolveQueens(n: i32) -> i32 {"));
    assert!(formatted.contains("fn Main() -> i32 {"));

    let _ = fs::remove_file(&formatted_file);
}
