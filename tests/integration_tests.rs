// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation Language Toolchain
// Licensed under the MIT License.

use carbonation::checker::Checker;
use carbonation::compiler::Compiler;
use carbonation::interpreter::Interpreter;
use carbonation::lexer::Lexer;
use carbonation::parser::Parser;
use carbonation::translator::Translator;
use carbonation::value::Value;
use std::fs;
use std::process::Command;

#[test]
fn test_lexer_tokens() {
    let source = "fn Add(x: i32, y: i32) -> i32 { return x + y; }";
    let mut lexer = Lexer::new(source, "test.carbon");
    let tokens = lexer.tokenize().expect("tokenization should succeed");
    assert!(!tokens.is_empty());
}

#[test]
fn test_hex_and_bin_literals() {
    let source = r#"
fn Run() -> i32 {
    var a: i32 = 0x2A; // 42
    var b: i32 = 0b1010; // 10
    return a + b; // 52
}
"#;
    let mut lexer = Lexer::new(source, "test.carbon");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(&tokens, "test.carbon");
    let program = parser.parse_program().unwrap();

    let mut interpreter = Interpreter::new("test.carbon");
    interpreter.load_program(&program).unwrap();
    let res = interpreter.run_entry().unwrap();
    assert_eq!(res, Value::Int(52));
}

#[test]
fn test_fibonacci_recursion() {
    let source = r#"
fn Fib(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return Fib(n - 1) + Fib(n - 2);
}

fn Main() -> i32 {
    return Fib(10); // 55
}
"#;
    let mut lexer = Lexer::new(source, "fib.carbon");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(&tokens, "fib.carbon");
    let program = parser.parse_program().unwrap();

    let mut checker = Checker::new("fib.carbon");
    checker.check_program(&program).unwrap();

    let mut interpreter = Interpreter::new("fib.carbon");
    interpreter.load_program(&program).unwrap();
    let res = interpreter.run_entry().unwrap();
    assert_eq!(res, Value::Int(55));
}

#[test]
fn test_compiler_standalone_executable() {
    let source = r#"
import Core library "io";

fn Main() -> i32 {
    var sum: i32 = 0;
    for (i: i32 in Core.InclusiveRange(1, 100)) {
        sum += i;
    }
    return sum; // 5050
}
"#;
    let mut lexer = Lexer::new(source, "sum.carbon");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(&tokens, "sum.carbon");
    let program = parser.parse_program().unwrap();

    let temp_out =
        std::env::temp_dir().join(format!("carbonation_test_sum_{}", std::process::id()));
    let compiler = Compiler::new("sum.carbon");
    compiler
        .compile_to_binary(&program, &temp_out, "O3")
        .expect("compilation should succeed");

    let output = Command::new(&temp_out).output().expect("should run binary");
    let _ = fs::remove_file(&temp_out);

    // Exit code modulo 256 for 5050 is 5050 % 256 = 186
    assert_eq!(output.status.code(), Some(5050 % 256));
}

#[test]
fn test_immutable_let_binding_error() {
    let source = r#"
fn Run() {
    let x: i32 = 10;
    x = 20;
}
"#;
    let mut lexer = Lexer::new(source, "let_err.carbon");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(&tokens, "let_err.carbon");
    let program = parser.parse_program().unwrap();

    let mut checker = Checker::new("let_err.carbon");
    let err = checker.check_program(&program);
    assert!(err.is_err());
    let err_msg = err.unwrap_err()[0].message.clone();
    assert!(err_msg.contains("cannot assign to immutable 'let' binding"));
}

#[test]
fn test_translator_c_output() {
    let source = r#"
class Point {
    var x: i32;
    var y: i32;
}

fn Run() -> i32 {
    var p: Point;
    p.x = 10;
    p.y = 20;
    return p.x + p.y;
}
"#;
    let mut lexer = Lexer::new(source, "point.carbon");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(&tokens, "point.carbon");
    let program = parser.parse_program().unwrap();

    let mut translator = Translator::new();
    let c_code = translator.translate(&program).unwrap();
    assert!(c_code.contains("struct Point"));
    assert!(c_code.contains("int32_t x;"));
    assert!(c_code.contains("int32_t y;"));
    assert!(c_code.contains("int main("));
}
