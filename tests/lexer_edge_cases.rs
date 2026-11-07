// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation Language Toolchain
// Licensed under the MIT License.

use carbonation::lexer::Lexer;
use carbonation::token::TokenKind;

#[test]
fn test_number_radixes_and_separators() {
    let source = "0 42 1_000_000 0x0 0xFF 0x1A_2B 0b0 0b1010_0101 0o77 0o12_34";
    let mut lexer = Lexer::new(source, "numbers.carbon");
    let tokens = lexer.tokenize().expect("tokenization failed");

    let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Integer(0),
            TokenKind::Integer(42),
            TokenKind::Integer(1_000_000),
            TokenKind::Integer(0),
            TokenKind::Integer(255),
            TokenKind::Integer(0x1A2B),
            TokenKind::Integer(0),
            TokenKind::Integer(0b10100101),
            TokenKind::Integer(0o77),
            TokenKind::Integer(0o1234),
            TokenKind::Eof,
        ]
    );
}

#[test]
#[allow(clippy::approx_constant)]
fn test_float_literals() {
    let source = "0.0 3.1415926535 1e6 2.5e-3 100.5E+2";
    let mut lexer = Lexer::new(source, "floats.carbon");
    let tokens = lexer.tokenize().expect("tokenization failed");

    match &tokens[0].kind {
        TokenKind::FloatLit(f) => assert!((*f - 0.0).abs() < 1e-9),
        _ => panic!("expected float"),
    }
    match &tokens[1].kind {
        TokenKind::FloatLit(f) => assert!((*f - 3.1415926535).abs() < 1e-9),
        _ => panic!("expected float"),
    }
    match &tokens[2].kind {
        TokenKind::FloatLit(f) => assert!((*f - 1_000_000.0).abs() < 1e-9),
        _ => panic!("expected float"),
    }
    match &tokens[3].kind {
        TokenKind::FloatLit(f) => assert!((*f - 0.0025).abs() < 1e-9),
        _ => panic!("expected float"),
    }
    match &tokens[4].kind {
        TokenKind::FloatLit(f) => assert!((*f - 10050.0).abs() < 1e-9),
        _ => panic!("expected float"),
    }
}

#[test]
fn test_string_escapes() {
    let source =
        r#""hello\nworld" "tab\there" "hex \x41\x42" "quotes \"inside\" and \\ backslash""#;
    let mut lexer = Lexer::new(source, "strings.carbon");
    let tokens = lexer.tokenize().expect("tokenization failed");

    assert_eq!(
        tokens[0].kind,
        TokenKind::StringLit("hello\nworld".to_string())
    );
    assert_eq!(
        tokens[1].kind,
        TokenKind::StringLit("tab\there".to_string())
    );
    assert_eq!(tokens[2].kind, TokenKind::StringLit("hex AB".to_string()));
    assert_eq!(
        tokens[3].kind,
        TokenKind::StringLit("quotes \"inside\" and \\ backslash".to_string())
    );
}

#[test]
fn test_comments_and_whitespace() {
    let source = r#"
    // Single line comment
    /* Multi-line
       block comment */
    var x: i32 = 10; /* inline comment */ // another
    /* comment before eof */
    "#;
    let mut lexer = Lexer::new(source, "comments.carbon");
    let tokens = lexer.tokenize().expect("tokenization failed");

    let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Var,
            TokenKind::Ident("x".to_string()),
            TokenKind::Colon,
            TokenKind::I32,
            TokenKind::Eq,
            TokenKind::Integer(10),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn test_unclosed_block_comment_error() {
    let source = "/* unclosed comment";
    let mut lexer = Lexer::new(source, "err.carbon");
    let res = lexer.tokenize();
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .message
            .contains("unterminated block comment")
    );
}

#[test]
fn test_unclosed_string_error() {
    let source = "\"unterminated string";
    let mut lexer = Lexer::new(source, "err.carbon");
    let res = lexer.tokenize();
    assert!(res.is_err());
    assert!(res.unwrap_err().message.contains("unterminated string"));
}

#[test]
fn test_64bit_masks() {
    let source = "0xFFFFFFFF_FFFFFFFF 0x7FFFFFFF_FFFFFFFF 0x80000000_00000000";
    let mut lexer = Lexer::new(source, "masks.carbon");
    let tokens = lexer.tokenize().expect("tokenization failed");

    assert_eq!(tokens[0].kind, TokenKind::Integer(-1));
    assert_eq!(tokens[1].kind, TokenKind::Integer(i64::MAX));
    assert_eq!(tokens[2].kind, TokenKind::Integer(i64::MIN));
}
