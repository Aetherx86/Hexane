// Copyright (c) 2026 jk9576. All rights reserved.
// Hexane Language Toolchain
// Licensed under the MIT License.

use crate::diagnostics::{Diagnostic, Span};
use crate::token::{Token, TokenKind};

pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<(usize, char)>,
    cursor: usize,
    filename: String,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, filename: impl Into<String>) -> Self {
        let chars: Vec<(usize, char)> = source.char_indices().collect();
        Self {
            source,
            chars,
            cursor: 0,
            filename: filename.into(),
            line: 1,
            col: 1,
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.cursor).map(|&(_, c)| c)
    }

    fn peek_next_char(&self) -> Option<char> {
        self.chars.get(self.cursor + 1).map(|&(_, c)| c)
    }

    fn current_byte_offset(&self) -> usize {
        if self.cursor < self.chars.len() {
            self.chars[self.cursor].0
        } else {
            self.source.len()
        }
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(&(_, c)) = self.chars.get(self.cursor) {
            self.cursor += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(c)
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), Diagnostic> {
        while let Some(c) = self.peek_char() {
            match c {
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                }
                '/' => {
                    if let Some('/') = self.peek_next_char() {
                        // Single-line comment
                        self.advance();
                        self.advance();
                        while let Some(ch) = self.peek_char() {
                            if ch == '\n' {
                                self.advance();
                                break;
                            }
                            self.advance();
                        }
                    } else if let Some('*') = self.peek_next_char() {
                        // Multi-line block comment
                        let start_line = self.line;
                        let start_col = self.col;
                        let start_offset = self.current_byte_offset();
                        self.advance();
                        self.advance();

                        let mut closed = false;
                        while let Some(ch) = self.advance() {
                            if ch == '*' && self.peek_char() == Some('/') {
                                self.advance();
                                closed = true;
                                break;
                            }
                        }
                        if !closed {
                            let end_offset = self.current_byte_offset();
                            return Err(Diagnostic::error(
                                &self.filename,
                                Span::new(start_line, start_col, start_offset, end_offset),
                                "unterminated block comment",
                            ));
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Diagnostic> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments()?;
            let start_line = self.line;
            let start_col = self.col;
            let start_offset = self.current_byte_offset();

            let Some(c) = self.peek_char() else {
                tokens.push(Token::new(
                    TokenKind::Eof,
                    Span::new(start_line, start_col, start_offset, start_offset),
                ));
                break;
            };

            let kind = match c {
                '+' => {
                    self.advance();
                    if self.peek_char() == Some('+') {
                        self.advance();
                        TokenKind::PlusPlus
                    } else if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::PlusEq
                    } else {
                        TokenKind::Plus
                    }
                }
                '-' => {
                    self.advance();
                    if self.peek_char() == Some('>') {
                        self.advance();
                        TokenKind::Arrow
                    } else if self.peek_char() == Some('-') {
                        self.advance();
                        TokenKind::MinusMinus
                    } else if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::MinusEq
                    } else {
                        TokenKind::Minus
                    }
                }
                '*' => {
                    self.advance();
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::StarEq
                    } else {
                        TokenKind::Star
                    }
                }
                '/' => {
                    self.advance();
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::SlashEq
                    } else {
                        TokenKind::Slash
                    }
                }
                '%' => {
                    self.advance();
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::PercentEq
                    } else {
                        TokenKind::Percent
                    }
                }
                '=' => {
                    self.advance();
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::EqEq
                    } else if self.peek_char() == Some('>') {
                        self.advance();
                        TokenKind::FatArrow
                    } else {
                        TokenKind::Eq
                    }
                }
                '!' => {
                    self.advance();
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::NotEq
                    } else {
                        TokenKind::Bang
                    }
                }
                '<' => {
                    self.advance();
                    if self.peek_char() == Some('<') {
                        self.advance();
                        if self.peek_char() == Some('=') {
                            self.advance();
                            TokenKind::ShlEq
                        } else {
                            TokenKind::Shl
                        }
                    } else if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::LtEq
                    } else {
                        TokenKind::Lt
                    }
                }
                '>' => {
                    self.advance();
                    if self.peek_char() == Some('>') {
                        self.advance();
                        if self.peek_char() == Some('=') {
                            self.advance();
                            TokenKind::ShrEq
                        } else {
                            TokenKind::Shr
                        }
                    } else if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::GtEq
                    } else {
                        TokenKind::Gt
                    }
                }
                '&' => {
                    self.advance();
                    if self.peek_char() == Some('&') {
                        self.advance();
                        TokenKind::AndAnd
                    } else if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::AmpEq
                    } else {
                        TokenKind::Amp
                    }
                }
                '|' => {
                    self.advance();
                    if self.peek_char() == Some('|') {
                        self.advance();
                        TokenKind::OrOr
                    } else if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::PipeEq
                    } else {
                        TokenKind::Pipe
                    }
                }
                '^' => {
                    self.advance();
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::CaretEq
                    } else {
                        TokenKind::Caret
                    }
                }
                '~' => {
                    self.advance();
                    TokenKind::Tilde
                }
                '.' => {
                    self.advance();
                    TokenKind::Dot
                }
                ':' => {
                    self.advance();
                    if self.peek_char() == Some(':') {
                        self.advance();
                        TokenKind::ColonColon
                    } else {
                        TokenKind::Colon
                    }
                }
                ';' => {
                    self.advance();
                    TokenKind::Semicolon
                }
                ',' => {
                    self.advance();
                    TokenKind::Comma
                }
                '(' => {
                    self.advance();
                    TokenKind::LParen
                }
                ')' => {
                    self.advance();
                    TokenKind::RParen
                }
                '{' => {
                    self.advance();
                    TokenKind::LBrace
                }
                '}' => {
                    self.advance();
                    TokenKind::RBrace
                }
                '[' => {
                    self.advance();
                    TokenKind::LBracket
                }
                ']' => {
                    self.advance();
                    TokenKind::RBracket
                }
                '"' => self.lex_string(start_line, start_col, start_offset)?,
                '0'..='9' => self.lex_number(start_line, start_col, start_offset)?,
                c if c.is_alphabetic() || c == '_' => {
                    self.lex_identifier(start_line, start_col, start_offset)
                }
                unexpected => {
                    let end_offset = self.current_byte_offset() + unexpected.len_utf8();
                    self.advance();
                    return Err(Diagnostic::error(
                        &self.filename,
                        Span::new(start_line, start_col, start_offset, end_offset),
                        format!("unexpected character '{}'", unexpected),
                    ));
                }
            };

            let end_offset = self.current_byte_offset();
            tokens.push(Token::new(
                kind,
                Span::new(start_line, start_col, start_offset, end_offset),
            ));
        }

        Ok(tokens)
    }

    fn lex_string(
        &mut self,
        start_line: usize,
        start_col: usize,
        start_offset: usize,
    ) -> Result<TokenKind, Diagnostic> {
        self.advance(); // consume opening quote
        let mut string_buf = String::new();

        while let Some(c) = self.advance() {
            match c {
                '"' => return Ok(TokenKind::StringLit(string_buf)),
                '\\' => {
                    let escaped = match self.advance() {
                        Some('n') => '\n',
                        Some('t') => '\t',
                        Some('r') => '\r',
                        Some('\\') => '\\',
                        Some('\'') => '\'',
                        Some('"') => '"',
                        Some('0') => '\0',
                        Some('x') => {
                            let mut hex = String::new();
                            for _ in 0..2 {
                                if let Some(h) = self.advance() {
                                    hex.push(h);
                                }
                            }
                            if let Ok(val) = u8::from_str_radix(&hex, 16) {
                                val as char
                            } else {
                                return Err(Diagnostic::error(
                                    &self.filename,
                                    Span::new(
                                        self.line,
                                        self.col,
                                        start_offset,
                                        self.current_byte_offset(),
                                    ),
                                    format!("invalid hex escape sequence '\\x{}'", hex),
                                ));
                            }
                        }
                        Some(other) => other,
                        None => {
                            return Err(Diagnostic::error(
                                &self.filename,
                                Span::new(
                                    start_line,
                                    start_col,
                                    start_offset,
                                    self.current_byte_offset(),
                                ),
                                "unexpected end of file in string escape",
                            ));
                        }
                    };
                    string_buf.push(escaped);
                }
                ch => string_buf.push(ch),
            }
        }

        Err(Diagnostic::error(
            &self.filename,
            Span::new(
                start_line,
                start_col,
                start_offset,
                self.current_byte_offset(),
            ),
            "unterminated string literal",
        ))
    }

    fn lex_number(
        &mut self,
        start_line: usize,
        start_col: usize,
        start_offset: usize,
    ) -> Result<TokenKind, Diagnostic> {
        let first = self.advance().unwrap();
        let mut raw = String::new();
        raw.push(first);

        if first == '0'
            && let Some(second) = self.peek_char()
        {
            if second == 'x' || second == 'X' {
                self.advance();
                let mut hex_str = String::new();
                while let Some(c) = self.peek_char() {
                    if c.is_ascii_hexdigit() {
                        hex_str.push(c);
                        self.advance();
                    } else if c == '_' {
                        self.advance(); // ignore separators
                    } else {
                        break;
                    }
                }
                if hex_str.is_empty() {
                    return Err(Diagnostic::error(
                        &self.filename,
                        Span::new(
                            start_line,
                            start_col,
                            start_offset,
                            self.current_byte_offset(),
                        ),
                        "hexadecimal literal requires at least one digit",
                    ));
                }
                let val = u64::from_str_radix(&hex_str, 16).map_err(|e| {
                    Diagnostic::error(
                        &self.filename,
                        Span::new(
                            start_line,
                            start_col,
                            start_offset,
                            self.current_byte_offset(),
                        ),
                        format!("hexadecimal literal out of range: {}", e),
                    )
                })? as i64;
                return Ok(TokenKind::Integer(val));
            } else if second == 'b' || second == 'B' {
                self.advance();
                let mut bin_str = String::new();
                while let Some(c) = self.peek_char() {
                    if c == '0' || c == '1' {
                        bin_str.push(c);
                        self.advance();
                    } else if c == '_' {
                        self.advance();
                    } else {
                        break;
                    }
                }
                if bin_str.is_empty() {
                    return Err(Diagnostic::error(
                        &self.filename,
                        Span::new(
                            start_line,
                            start_col,
                            start_offset,
                            self.current_byte_offset(),
                        ),
                        "binary literal requires at least one digit",
                    ));
                }
                let val = u64::from_str_radix(&bin_str, 2).map_err(|e| {
                    Diagnostic::error(
                        &self.filename,
                        Span::new(
                            start_line,
                            start_col,
                            start_offset,
                            self.current_byte_offset(),
                        ),
                        format!("binary literal out of range: {}", e),
                    )
                })? as i64;
                return Ok(TokenKind::Integer(val));
            } else if second == 'o' || second == 'O' {
                self.advance();
                let mut oct_str = String::new();
                while let Some(c) = self.peek_char() {
                    if ('0'..='7').contains(&c) {
                        oct_str.push(c);
                        self.advance();
                    } else if c == '_' {
                        self.advance();
                    } else {
                        break;
                    }
                }
                let val = u64::from_str_radix(&oct_str, 8).map_err(|e| {
                    Diagnostic::error(
                        &self.filename,
                        Span::new(
                            start_line,
                            start_col,
                            start_offset,
                            self.current_byte_offset(),
                        ),
                        format!("octal literal out of range: {}", e),
                    )
                })? as i64;
                return Ok(TokenKind::Integer(val));
            }
        }

        let mut is_float = false;

        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                raw.push(c);
                self.advance();
            } else if c == '_' {
                self.advance();
            } else if c == '.'
                && !is_float
                && self
                    .peek_next_char()
                    .is_some_and(|next| next.is_ascii_digit())
            {
                is_float = true;
                raw.push(c);
                self.advance();
            } else if (c == 'e' || c == 'E') && !raw.ends_with('e') && !raw.ends_with('E') {
                is_float = true;
                raw.push(c);
                self.advance();
                if let Some(sign) = self.peek_char()
                    && (sign == '+' || sign == '-')
                {
                    raw.push(sign);
                    self.advance();
                }
            } else {
                break;
            }
        }

        if is_float {
            let val = raw.parse::<f64>().map_err(|e| {
                Diagnostic::error(
                    &self.filename,
                    Span::new(
                        start_line,
                        start_col,
                        start_offset,
                        self.current_byte_offset(),
                    ),
                    format!("invalid floating point literal: {}", e),
                )
            })?;
            Ok(TokenKind::FloatLit(val))
        } else {
            let val = raw
                .parse::<i64>()
                .or_else(|_| raw.parse::<u64>().map(|v| v as i64))
                .map_err(|e| {
                    Diagnostic::error(
                        &self.filename,
                        Span::new(
                            start_line,
                            start_col,
                            start_offset,
                            self.current_byte_offset(),
                        ),
                        format!("integer literal out of range: {}", e),
                    )
                })?;
            Ok(TokenKind::Integer(val))
        }
    }

    fn lex_identifier(
        &mut self,
        _start_line: usize,
        _start_col: usize,
        _start_offset: usize,
    ) -> TokenKind {
        let mut id = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                id.push(c);
                self.advance();
            } else {
                break;
            }
        }

        match id.as_str() {
            "fn" => TokenKind::Fn,
            "var" => TokenKind::Var,
            "let" => TokenKind::Let,
            "returned" => TokenKind::Returned,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "class" => TokenKind::Class,
            "struct" => TokenKind::Struct,
            "impl" => TokenKind::Impl,
            "as" => TokenKind::As,
            "interface" => TokenKind::Interface,
            "package" => TokenKind::Package,
            "library" => TokenKind::Library,
            "import" => TokenKind::Import,
            "api" => TokenKind::Api,
            "ref" => TokenKind::Ref,
            "self" => TokenKind::SelfKw,
            "extern" => TokenKind::Extern,
            "and" => TokenKind::AndAnd,
            "or" => TokenKind::OrOr,
            "not" => TokenKind::Bang,

            "i8" => TokenKind::I8,
            "i16" => TokenKind::I16,
            "i32" => TokenKind::I32,
            "i64" => TokenKind::I64,
            "u8" => TokenKind::U8,
            "u16" => TokenKind::U16,
            "u32" => TokenKind::U32,
            "u64" => TokenKind::U64,
            "int" => TokenKind::Int,
            "f32" => TokenKind::F32,
            "f64" => TokenKind::F64,
            "float" => TokenKind::Float,
            "bool" => TokenKind::BoolType,
            "String" => TokenKind::StringType,
            "StringView" => TokenKind::StringViewType,
            "auto" => TokenKind::Auto,
            "void" => TokenKind::Void,
            "array" => TokenKind::Array,

            "true" => TokenKind::BoolLit(true),
            "false" => TokenKind::BoolLit(false),

            _ => TokenKind::Ident(id),
        }
    }
}
