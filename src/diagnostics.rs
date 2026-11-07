// Copyright (c) 2026 jk9576. All rights reserved.
// Hexane Language Toolchain
// Licensed under the MIT License.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(line: usize, col: usize, start: usize, end: usize) -> Self {
        Self {
            line,
            col,
            start,
            end,
        }
    }

    pub fn dummy() -> Self {
        Self {
            line: 1,
            col: 1,
            start: 0,
            end: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticLevel::Error => write!(f, "error"),
            DiagnosticLevel::Warning => write!(f, "warning"),
            DiagnosticLevel::Note => write!(f, "note"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub filename: String,
    pub span: Span,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn error(filename: impl Into<String>, span: Span, message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: message.into(),
            filename: filename.into(),
            span,
            hint: None,
        }
    }

    pub fn warning(filename: impl Into<String>, span: Span, message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message: message.into(),
            filename: filename.into(),
            span,
            hint: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn render_standard(&self) -> String {
        format!(
            "{}:{}:{}: {}: {}",
            self.filename, self.span.line, self.span.col, self.level, self.message
        )
    }

    pub fn render_json(&self) -> String {
        let escaped_msg = self.message.replace('\\', "\\\\").replace('\"', "\\\"");
        let escaped_file = self.filename.replace('\\', "\\\\").replace('\"', "\\\"");
        format!(
            "{{\"level\":\"{}\",\"file\":\"{}\",\"line\":{},\"col\":{},\"message\":\"{}\"}}",
            self.level, escaped_file, self.span.line, self.span.col, escaped_msg
        )
    }

    pub fn render(&self, source: &str) -> String {
        let mut out = String::new();
        let color_prefix = match self.level {
            DiagnosticLevel::Error => "\x1b[1;31m",
            DiagnosticLevel::Warning => "\x1b[1;33m",
            DiagnosticLevel::Note => "\x1b[1;36m",
        };
        let reset = "\x1b[0m";
        let bold = "\x1b[1m";

        out.push_str(&format!(
            "{}{}{}: {}{}\n",
            color_prefix, self.level, reset, bold, self.message
        ));
        out.push_str(&format!(
            "  --> {}:{}:{}\n",
            self.filename, self.span.line, self.span.col
        ));

        let lines: Vec<&str> = source.lines().collect();
        if self.span.line > 0 && self.span.line <= lines.len() {
            let line_num = self.span.line;
            let line_text = lines[line_num - 1];
            let line_prefix = format!("{:4} | ", line_num);
            let indent = "     | ";

            out.push_str(indent);
            out.push('\n');
            out.push_str(&line_prefix);
            out.push_str(line_text);
            out.push('\n');
            out.push_str(indent);

            let col_offset = if self.span.col > 0 {
                self.span.col - 1
            } else {
                0
            };
            for _ in 0..col_offset {
                out.push(' ');
            }
            let len = if self.span.end > self.span.start {
                (self.span.end - self.span.start).max(1)
            } else {
                1
            };
            out.push_str(color_prefix);
            for _ in 0..len {
                out.push('^');
            }
            out.push_str(reset);
            out.push('\n');
        }

        if let Some(hint) = &self.hint {
            out.push_str(&format!("  = hint: {}\n", hint));
        }

        out
    }
}
