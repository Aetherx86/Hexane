// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation Language Toolchain
// Licensed under the MIT License.

use carbonation::assembler::Assembler;
use carbonation::compiler::Compiler;
use carbonation::lexer::Lexer;
use carbonation::parser::Parser;
use carbonation::translator::Translator;
use std::fs;
use std::process::Command;

fn parse(source: &str, name: &str) -> carbonation::ast::Program {
    let mut lexer = Lexer::new(source, name);
    let tokens = lexer.tokenize().expect("lexing failed");
    let mut parser = Parser::new(&tokens, name);
    parser.parse_program().expect("parsing failed")
}

#[test]
fn test_collatz_conjecture_compiled() {
    let source = r#"
fn CollatzLength(var n: i32) -> i32 {
    var steps: i32 = 0;
    while (n != 1) {
        if (n % 2 == 0) {
            n = n / 2;
        } else {
            n = 3 * n + 1;
        }
        steps += 1;
    }
    return steps;
}

fn Main() -> i32 {
    // Collatz length of 27 is 111 steps
    return CollatzLength(27);
}
"#;
    let program = parse(source, "collatz.carbon");
    let temp_bin = std::env::temp_dir().join(format!("carbon_test_collatz_{}", std::process::id()));

    let compiler = Compiler::new("collatz.carbon");
    compiler
        .compile_to_binary(&program, &temp_bin, "O3")
        .unwrap();

    let output = Command::new(&temp_bin).output().unwrap();
    let _ = fs::remove_file(&temp_bin);

    assert_eq!(output.status.code(), Some(111));
}

#[test]
fn test_binary_search_compiled() {
    let source = r#"
class Searcher {
    var arr: array(i32, 8);

    fn Init(ref self) {
        self.arr[0] = 3;
        self.arr[1] = 9;
        self.arr[2] = 14;
        self.arr[3] = 19;
        self.arr[4] = 27;
        self.arr[5] = 35;
        self.arr[6] = 42;
        self.arr[7] = 88;
    }

    fn BinarySearch(ref self, target: i32) -> i32 {
        var low: i32 = 0;
        var high: i32 = 7;
        while (low <= high) {
            var mid: i32 = (low + high) / 2;
            if (self.arr[mid] == target) {
                return mid;
            }
            if (self.arr[mid] < target) {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }
        return -1;
    }
}

fn Main() -> i32 {
    var s: Searcher;
    s.Init();
    var idx: i32 = s.BinarySearch(35); // Index should be 5
    return idx;
}
"#;
    let program = parse(source, "bsearch.carbon");
    let temp_bin = std::env::temp_dir().join(format!("carbon_test_bsearch_{}", std::process::id()));

    let compiler = Compiler::new("bsearch.carbon");
    compiler
        .compile_to_binary(&program, &temp_bin, "O2")
        .unwrap();

    let output = Command::new(&temp_bin).output().unwrap();
    let _ = fs::remove_file(&temp_bin);

    assert_eq!(output.status.code(), Some(5));
}

#[test]
fn test_assemble_to_s_and_compile_with_host_assembler() {
    let source = r#"
fn Main() -> i32 {
    var x: i32 = 20;
    var y: i32 = 22;
    return x + y; // 42
}
"#;
    let program = parse(source, "asm_test.carbon");
    let temp_s = std::env::temp_dir().join(format!("carbon_test_asm_s_{}.s", std::process::id()));
    let temp_bin =
        std::env::temp_dir().join(format!("carbon_test_asm_s_{}_bin", std::process::id()));

    let assembler = Assembler::new("asm_test.carbon");
    assembler.assemble_to_file(&program, &temp_s, "O3").unwrap();

    // Verify .s file exists and contains assembly instructions
    let s_content = fs::read_to_string(&temp_s).unwrap();
    assert!(s_content.contains("Main") || s_content.contains("_Main"));

    // Compile assembly to executable using clang
    let clang_status = Command::new("clang")
        .arg(&temp_s)
        .arg("-o")
        .arg(&temp_bin)
        .status()
        .unwrap();
    assert!(clang_status.success());

    let output = Command::new(&temp_bin).output().unwrap();
    let _ = fs::remove_file(&temp_s);
    let _ = fs::remove_file(&temp_bin);

    assert_eq!(output.status.code(), Some(42));
}

#[test]
fn test_assemble_to_object_file_and_link() {
    let source = r#"
fn Main() -> i32 {
    return 77;
}
"#;
    let program = parse(source, "obj_test.carbon");
    let temp_o = std::env::temp_dir().join(format!("carbon_test_obj_o_{}.o", std::process::id()));
    let temp_bin =
        std::env::temp_dir().join(format!("carbon_test_obj_o_{}_bin", std::process::id()));

    let assembler = Assembler::new("obj_test.carbon");
    assembler.assemble_to_file(&program, &temp_o, "O3").unwrap();

    // Link object file into binary
    let link_status = Command::new("clang")
        .arg(&temp_o)
        .arg("-o")
        .arg(&temp_bin)
        .status()
        .unwrap();
    assert!(link_status.success());

    let output = Command::new(&temp_bin).output().unwrap();
    let _ = fs::remove_file(&temp_o);
    let _ = fs::remove_file(&temp_bin);

    assert_eq!(output.status.code(), Some(77));
}

#[test]
fn test_all_optimization_levels() {
    let source = r#"
fn Main() -> i32 {
    var acc: i32 = 0;
    var i: i32 = 0;
    while (i < 10) {
        acc += i;
        i += 1;
    }
    return acc; // 45
}
"#;
    let program = parse(source, "opt.carbon");
    let compiler = Compiler::new("opt.carbon");

    for opt in &["O0", "O1", "O2", "O3"] {
        let temp_bin = std::env::temp_dir().join(format!(
            "carbon_test_opt_level_{}_{}_bin",
            opt,
            std::process::id()
        ));
        compiler
            .compile_to_binary(&program, &temp_bin, opt)
            .unwrap();

        let output = Command::new(&temp_bin).output().unwrap();
        let _ = fs::remove_file(&temp_bin);

        assert_eq!(output.status.code(), Some(45));
    }
}

#[test]
fn test_translator_and_c_compilation() {
    let source = r#"
import Core library "io";

fn Main() -> i32 {
    var msg: String = "Transpiled C Success\n";
    Core.PrintStr(msg);
    return 0;
}
"#;
    let program = parse(source, "trans.carbon");
    let mut translator = Translator::new();
    let c_code = translator.translate(&program).unwrap();

    let temp_c = std::env::temp_dir().join(format!("carbon_test_trans_{}.c", std::process::id()));
    let temp_bin =
        std::env::temp_dir().join(format!("carbon_test_trans_{}_bin", std::process::id()));

    fs::write(&temp_c, &c_code).unwrap();

    let status = Command::new("clang")
        .arg("-std=c99")
        .arg("-Wall")
        .arg("-Werror")
        .arg(&temp_c)
        .arg("-o")
        .arg(&temp_bin)
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new(&temp_bin).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _ = fs::remove_file(&temp_c);
    let _ = fs::remove_file(&temp_bin);

    assert!(stdout.contains("Transpiled C Success"));
}
