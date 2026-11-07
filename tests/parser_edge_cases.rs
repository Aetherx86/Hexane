// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation Language Toolchain
// Licensed under the MIT License.

use carbonation::ast::*;
use carbonation::lexer::Lexer;
use carbonation::parser::Parser;

#[test]
fn test_precedence_and_associativity() {
    let source = "var x: bool = 1 + 2 * 3 == 7 && !false;";
    let mut lexer = Lexer::new(source, "precedence.carbon");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(&tokens, "precedence.carbon");
    let program = parser.parse_program().unwrap();

    assert_eq!(program.globals.len(), 1);
    match &program.globals[0] {
        Stmt::VarDecl { name, init, .. } => {
            assert_eq!(name, "x");
            let init_expr = init.as_ref().unwrap();
            match init_expr {
                Expr::Binary {
                    op: BinaryOp::And,
                    left,
                    right,
                    ..
                } => {
                    // Left is (1 + (2 * 3)) == 7
                    match &**left {
                        Expr::Binary {
                            op: BinaryOp::Eq,
                            left: arith,
                            right: seven,
                            ..
                        } => {
                            assert!(matches!(**seven, Expr::Integer(7, _)));
                            match &**arith {
                                Expr::Binary {
                                    op: BinaryOp::Add,
                                    left: one,
                                    right: mul,
                                    ..
                                } => {
                                    assert!(matches!(**one, Expr::Integer(1, _)));
                                    match &**mul {
                                        Expr::Binary {
                                            op: BinaryOp::Mul, ..
                                        } => {}
                                        _ => panic!("expected mul on right of add"),
                                    }
                                }
                                _ => panic!("expected add on left of eq"),
                            }
                        }
                        _ => panic!("expected eq on left of and"),
                    }
                    // Right is !false
                    match &**right {
                        Expr::Unary {
                            op: UnaryOp::Not, ..
                        } => {}
                        _ => panic!("expected not on right of and"),
                    }
                }
                _ => panic!("expected and at root"),
            }
        }
        _ => panic!("expected var decl"),
    }
}

#[test]
fn test_complex_indexing_and_calls() {
    let source = r#"
fn Process(data: array(i32, 10)) -> i32 {
    return data[0] + data[1] * data[2];
}
"#;
    let mut lexer = Lexer::new(source, "index.carbon");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(&tokens, "index.carbon");
    let program = parser.parse_program().unwrap();

    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "Process");
    assert_eq!(program.functions[0].params.len(), 1);
    assert_eq!(
        program.functions[0].params[0].ty,
        Type::Array(Box::new(Type::I32), 10)
    );
}

#[test]
fn test_class_with_fields_and_methods() {
    let source = r#"
class Vector2D {
    var x: f64;
    var y: f64;

    fn Magnitude(ref self) -> f64 {
        return self.x * self.x + self.y * self.y;
    }
}
"#;
    let mut lexer = Lexer::new(source, "vec.carbon");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(&tokens, "vec.carbon");
    let program = parser.parse_program().unwrap();

    assert_eq!(program.classes.len(), 1);
    let class = &program.classes[0];
    assert_eq!(class.name, "Vector2D");
    assert_eq!(class.fields.len(), 2);
    assert_eq!(class.fields[0].0, "x");
    assert_eq!(class.fields[0].1, Type::F64);
    assert_eq!(class.methods.len(), 1);
    assert_eq!(class.methods[0].name, "Magnitude");
}

#[test]
fn test_loop_control_statements() {
    let source = r#"
fn Run() {
    var i: i32 = 0;
    while (i < 100) {
        i += 1;
        if (i == 50) {
            continue;
        }
        if (i == 80) {
            break;
        }
    }
}
"#;
    let mut lexer = Lexer::new(source, "loops.carbon");
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(&tokens, "loops.carbon");
    let program = parser.parse_program().unwrap();

    assert_eq!(program.functions.len(), 1);
    let body = &program.functions[0].body;
    assert_eq!(body.len(), 2);
    match &body[1] {
        Stmt::While {
            body: while_body, ..
        } => {
            assert_eq!(while_body.len(), 3);
            assert!(matches!(while_body[1], Stmt::If { .. }));
            assert!(matches!(while_body[2], Stmt::If { .. }));
        }
        _ => panic!("expected while"),
    }
}
