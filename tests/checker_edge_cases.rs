// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation Language Toolchain
// Licensed under the MIT License.

use carbonation::checker::Checker;
use carbonation::lexer::Lexer;
use carbonation::parser::Parser;

fn check_source(source: &str) -> Result<(), Vec<String>> {
    let mut lexer = Lexer::new(source, "test.carbon");
    let tokens = lexer.tokenize().map_err(|e| vec![e.message])?;
    let mut parser = Parser::new(&tokens, "test.carbon");
    let program = parser.parse_program().map_err(|e| vec![e.message])?;
    let mut checker = Checker::new("test.carbon");
    checker
        .check_program(&program)
        .map_err(|errs| errs.into_iter().map(|e| e.message).collect())
}

#[test]
fn test_undeclared_variable() {
    let source = r#"
fn Run() {
    foo = 42;
}
"#;
    let res = check_source(source);
    assert!(res.is_err());
    assert!(res.unwrap_err()[0].contains("undefined identifier 'foo'"));
}

#[test]
fn test_duplicate_functions() {
    let source = r#"
fn Test() -> i32 { return 1; }
fn Test() -> i32 { return 2; }
"#;
    let res = check_source(source);
    assert!(res.is_err());
    assert!(res.unwrap_err()[0].contains("duplicate function 'Test'"));
}

#[test]
fn test_return_type_mismatch() {
    let source = r#"
fn GetNumber() -> i32 {
    return "not a number";
}
"#;
    let res = check_source(source);
    assert!(res.is_err());
    assert!(res.unwrap_err()[0].contains("return type mismatch"));
}

#[test]
fn test_return_var_without_returned_var() {
    let source = r#"
fn Invalid() -> i32 {
    var x: i32 = 10;
    return var;
}
"#;
    let res = check_source(source);
    assert!(res.is_err());
    assert!(res.unwrap_err()[0].contains("'return var;' used without 'returned var' declaration"));
}

#[test]
fn test_block_scoping() {
    let source = r#"
fn ScopeTest() -> i32 {
    var x: i32 = 10;
    if (x > 5) {
        var x: i32 = 20; // shadowed inside block
    }
    return x; // outer x still 10
}
"#;
    assert!(check_source(source).is_ok());
}
