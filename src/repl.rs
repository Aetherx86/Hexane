// Copyright (c) 2026 jk9576. All rights reserved.
// Graphene Language Toolchain
// Licensed under the MIT License.

use crate::checker::Checker;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::io::{self, Write};

pub struct Repl {
    interpreter: Interpreter,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new("<repl>"),
        }
    }

    pub fn start(&mut self) {
        println!("\x1b[1;36mGraphene Language REPL (v0.1.0)\x1b[0m");
        println!("Carbon & Graphene compatible interactive shell.");
        println!("Type expressions or declarations. Press Ctrl+D or type 'exit' to quit.\n");

        let stdin = io::stdin();
        let mut line_buffer = String::new();

        loop {
            print!("\x1b[1;32mgraphene>\x1b[0m ");
            let _ = io::stdout().flush();

            line_buffer.clear();
            if stdin.read_line(&mut line_buffer).unwrap_or(0) == 0 {
                // EOF
                println!("\nGoodbye!");
                break;
            }

            let trimmed = line_buffer.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed == "exit" || trimmed == "quit" {
                println!("Goodbye!");
                break;
            }

            self.execute_line(trimmed);
        }
    }

    fn execute_line(&mut self, line: &str) {
        // Try parsing as full program declarations first (functions, classes, globals)
        let mut lexer = Lexer::new(line, "<repl>");
        if let Ok(tokens) = lexer.tokenize() {
            let mut parser = Parser::new(&tokens, "<repl>");
            if let Ok(program) = parser.parse_program()
                && (!program.functions.is_empty()
                    || !program.classes.is_empty()
                    || !program.globals.is_empty())
            {
                let mut checker = Checker::new("<repl>");
                if let Err(errors) = checker.check_program(&program) {
                    for err in errors {
                        eprintln!("{}", err.render(line));
                    }
                    return;
                }

                if let Err(err) = self.interpreter.load_program(&program) {
                    eprintln!("{}", err.render(line));
                    return;
                }
                println!("\x1b[32m[defined]\x1b[0m");
                return;
            }
        }

        // Try parsing as an expression
        let mut lexer = Lexer::new(line, "<repl>");
        if let Ok(tokens) = lexer.tokenize() {
            let mut parser = Parser::new(&tokens, "<repl>");
            if let Ok(expr) = parser.parse_expression() {
                match self.interpreter.eval_repl_expr(&expr) {
                    Ok(val) => {
                        println!("\x1b[1;35m=> {}\x1b[0m", val);
                        return;
                    }
                    Err(diag) => {
                        eprintln!("{}", diag.render(line));
                        return;
                    }
                }
            }
        }

        eprintln!("\x1b[31merror: invalid expression or statement in REPL\x1b[0m");
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}
