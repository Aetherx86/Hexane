// Copyright (c) 2026 jk9576. All rights reserved.
// Graphene Language Toolchain
// Licensed under the MIT License.

pub mod assembler;
pub mod ast;
pub mod checker;
pub mod compiler;
pub mod diagnostics;
pub mod formatter;
pub mod interpreter;
pub mod lexer;
pub mod license;
pub mod parser;
pub mod repl;
pub mod sem_ir;
pub mod token;
pub mod translator;
pub mod value;
pub mod vmp_revirt;
pub mod zkhypervm;

use assembler::Assembler;
use checker::Checker;
use compiler::Compiler;
use formatter::Formatter;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use repl::Repl;
use sem_ir::SemProgram;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use translator::Translator;
use value::Value;
use vmp_revirt::VmpRevirtualizer;
use zkhypervm::ZkHyperVm;

pub const VERSION: &str = "0.1.0";
pub const BANNER: &str = r#"
   ______           __                     __  _           
  / ____/___ ______/ /_  ____  ____  ____ _/ /_(_)___  ____  
 / /   / __ `/ ___/ __ \/ __ \/ __ \/ __ `/ __/ / __ \/ __ \ 
/ /___/ /_/ / /  / /_/ / /_/ / / / / /_/ / /_/ / /_/ / / / / 
\____/\__,_/_/  /_.___/\____/_/ /_/\__,_/\__/_/\____/_/ /_/  
                                                              
  Carbonation Toolchain v0.1.0 (alias: graphene, hexane, hexene)
  Copyright (c) 2026 jk9576. All rights reserved.
  Ultra-Fast, Memory-Safe Carbon-Compatible Compiler & Runtime
"#;

pub fn is_foreign_source(path_str: &str) -> bool {
    if path_str == "-" {
        return false;
    }
    let p = Path::new(path_str);
    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "c" | "h" | "cpp" | "cc" | "cxx" | "hpp" | "s" | "asm" | "o" | "a"
        )
    } else {
        false
    }
}

pub fn print_help() {
    println!("{}", BANNER);
    println!("USAGE:");
    println!("    carbonation <COMMAND> [OPTIONS] [FILE...]");
    println!("    (or: graphene, hexane, hexene)\n");
    println!("COMMANDS:");
    println!("    run, interpret <FILES...>        Directly interpret and execute source file(s)");
    println!(
        "    compile <FILES...> [-o <OUT>]    Compile file(s) to optimized standalone native binary"
    );
    println!("    translate <FILES...> [-o <OUT>]  Translate file(s) to standard C99/C++");
    println!(
        "    assemble <FILES...> [-o <OUT.s>] Generate native machine assembly (.s) or object (.o)"
    );
    println!(
        "    combine <FILES...> [-o <OUT>]    Combine multiple Carbon source files into a unified program"
    );
    println!(
        "    check <FILES...>                 Check syntax and semantic types without executing"
    );
    println!(
        "    revirt <FILES...> [-o <OUT>]     Re-virtualize into hardened VM bytecode / CFF runner"
    );
    println!(
        "    zkhypervm, zk <FILES...>         Execute in zk-HyperVM with verifiable execution trace"
    );
    println!(
        "    sem-ir <FILES...>                Dump Semantic Intermediate Representation (SemIR)"
    );
    println!(
        "    format, fmt <FILES...>           Format Carbon source code according to style guide"
    );
    println!("    repl                             Launch interactive Carbonation REPL shell");
    println!("    help, --help, -h                 Print this help message");
    println!("    version, --version, -v           Print version information\n");
    println!("OPTIONS:");
    println!("    -o <PATH>                        Specify output file path");
    println!("    -O0, -O1, -O2, -O3               Optimization level (default: -O3)");
    println!(
        "    -E                               Preprocess / translate to stdout (gcc/clang compatible)"
    );
    println!("    -S                               Compile to assembly .s (gcc/clang compatible)");
    println!(
        "    -c                               Compile to object file .o (gcc/clang compatible)"
    );
    println!("    --dump-ast                       Dump parsed Abstract Syntax Tree");
    println!("    --dump-tokens                    Dump token stream from lexer");
    println!("\nEXAMPLES:");
    println!("    carbonation run hello.carbon");
    println!("    carbonation compile math.carbon main.carbon -o app");
    println!("    carbonation translate sieve.carbon -o sieve.c");
    println!("    carbonation assemble sieve.carbon -o sieve.s");
    println!("    echo 'fn Run() -> i32 {{ return 42; }}' | carbonation run -");
}

pub fn print_version() {
    println!(
        "Carbonation Toolchain v{} (alias: graphene, hexane, hexene)",
        VERSION
    );
    println!("Copyright (c) 2026 jk9576. All rights reserved.");
    println!("Target: {}", env::consts::ARCH);
    println!("Built with safe Rust: memory-safe, zero leaks, zero segfaults.");
}

pub fn parse_file_raw(
    path_str: &str,
    dump_tokens: bool,
    dump_ast: bool,
) -> Result<(String, ast::Program), i32> {
    use std::io::Read;

    let source = if path_str == "-" {
        let mut s = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut s) {
            eprintln!("\x1b[1;31merror\x1b[0m: failed to read from stdin: {}", e);
            return Err(1);
        }
        s
    } else {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("\x1b[1;31merror\x1b[0m: file not found: '{}'", path_str);
            return Err(1);
        }
        match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "\x1b[1;31merror\x1b[0m: failed to read file '{}': {}",
                    path_str, e
                );
                return Err(1);
            }
        }
    };

    let mut lexer = Lexer::new(&source, path_str);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(diag) => {
            eprintln!("{}", diag.render(&source));
            return Err(1);
        }
    };

    if dump_tokens {
        println!("=== TOKENS [{}] ===", path_str);
        for t in &tokens {
            println!(
                "{:?} ({}:{}:{})",
                t.kind, t.span.line, t.span.col, t.span.start
            );
        }
    }

    let mut parser = Parser::new(&tokens, path_str);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(diag) => {
            eprintln!("{}", diag.render(&source));
            return Err(1);
        }
    };

    if dump_ast {
        println!("=== AST [{}] ===", path_str);
        println!("{:#?}", program);
    }

    Ok((source, program))
}

pub fn parse_file(
    path_str: &str,
    dump_tokens: bool,
    dump_ast: bool,
) -> Result<(String, ast::Program), i32> {
    let (source, program) = parse_file_raw(path_str, dump_tokens, dump_ast)?;
    let mut checker = Checker::new(path_str);
    if let Err(errors) = checker.check_program(&program) {
        for err in errors {
            eprintln!("{}", err.render(&source));
        }
        return Err(1);
    }
    Ok((source, program))
}

pub fn parse_and_combine_files(
    files: &[String],
    dump_tokens: bool,
    dump_ast: bool,
) -> Result<(String, ast::Program), i32> {
    if files.is_empty() {
        eprintln!("\x1b[1;31merror\x1b[0m: no input files provided");
        return Err(1);
    }

    let mut combined_source = String::new();
    let mut combined_program = ast::Program::new();

    for file in files {
        let (src, prog) = parse_file_raw(file, dump_tokens, dump_ast)?;
        combined_source.push_str(&src);
        combined_source.push('\n');
        combined_program.merge(prog);
    }

    let report_name = if files.len() == 1 {
        files[0].clone()
    } else {
        files.join(" + ")
    };

    let mut checker = Checker::new(&report_name);
    if let Err(errors) = checker.check_program(&combined_program) {
        for err in errors {
            eprintln!("{}", err.render(&combined_source));
        }
        return Err(1);
    }

    Ok((combined_source, combined_program))
}

fn parse_cli_args(args: &[String]) -> (Vec<String>, Option<String>, String, bool, bool) {
    let mut files = Vec::new();
    let mut output = None;
    let mut opt = "O3".to_string();
    let mut dump_tokens = false;
    let mut dump_ast = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" if i + 1 < args.len() => {
                output = Some(args[i + 1].clone());
                i += 2;
            }
            opt_flag if opt_flag.starts_with("-O") => {
                opt = opt_flag[1..].to_string();
                i += 1;
            }
            "--dump-tokens" => {
                dump_tokens = true;
                i += 1;
            }
            "--dump-ast" => {
                dump_ast = true;
                i += 1;
            }
            file if !file.starts_with('-') || file == "-" => {
                files.push(file.to_string());
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    (files, output, opt, dump_tokens, dump_ast)
}

pub fn run_cli() {
    license::enforce_license_or_exit("hexane");
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        process::exit(0);
    }

    let command = args[1].as_str();

    match command {
        "-h" | "--help" | "help" => {
            print_help();
        }
        "-v" | "--version" | "version" => {
            print_version();
        }
        "-E" => {
            if args.len() < 3 {
                eprintln!("\x1b[1;31merror\x1b[0m: missing source file for '-E'");
                process::exit(1);
            }
            let file = &args[2];
            let (_, program) = match parse_file(file, false, false) {
                Ok(res) => res,
                Err(code) => process::exit(code),
            };
            let mut translator = Translator::new();
            match translator.translate(&program) {
                Ok(c_code) => print!("{}", c_code),
                Err(diag) => {
                    eprintln!("{}", diag.render_standard());
                    process::exit(1);
                }
            }
        }
        "-S" => {
            if args.len() < 3 {
                eprintln!("\x1b[1;31merror\x1b[0m: missing source file for '-S'");
                process::exit(1);
            }
            let file = &args[2];
            let out_file = Path::new(file)
                .with_extension("s")
                .to_string_lossy()
                .to_string();
            let assembler = Assembler::new(file);
            if is_foreign_source(file) {
                if let Err(diag) =
                    assembler.assemble_source(Path::new(file), Path::new(&out_file), "O3")
                {
                    eprintln!("{}", diag.render_standard());
                    process::exit(1);
                }
            } else {
                let (source, program) = match parse_file(file, false, false) {
                    Ok(res) => res,
                    Err(code) => process::exit(code),
                };
                if let Err(diag) = assembler.assemble_to_file(&program, Path::new(&out_file), "O3")
                {
                    eprintln!("{}", diag.render(&source));
                    process::exit(1);
                }
            }
            println!("\x1b[1;32m[OK]\x1b[0m Generated assembly '{}'", out_file);
        }
        "-c" => {
            if args.len() < 3 {
                eprintln!("\x1b[1;31merror\x1b[0m: missing source file for '-c'");
                process::exit(1);
            }
            let file = &args[2];
            let out_file = Path::new(file)
                .with_extension("o")
                .to_string_lossy()
                .to_string();
            let assembler = Assembler::new(file);
            if is_foreign_source(file) {
                if let Err(diag) =
                    assembler.assemble_source(Path::new(file), Path::new(&out_file), "O3")
                {
                    eprintln!("{}", diag.render_standard());
                    process::exit(1);
                }
            } else {
                let (source, program) = match parse_file(file, false, false) {
                    Ok(res) => res,
                    Err(code) => process::exit(code),
                };
                if let Err(diag) = assembler.assemble_to_file(&program, Path::new(&out_file), "O3")
                {
                    eprintln!("{}", diag.render(&source));
                    process::exit(1);
                }
            }
            println!("\x1b[1;32m[OK]\x1b[0m Generated object file '{}'", out_file);
        }
        "repl" => {
            let mut repl = Repl::new();
            repl.start();
        }
        "check" => {
            let (files, _, _, _, _) = parse_cli_args(&args[2..]);
            if files.is_empty() {
                eprintln!("\x1b[1;31merror\x1b[0m: missing source file(s) for 'check'");
                process::exit(1);
            }
            match parse_and_combine_files(&files, false, false) {
                Ok(_) => {
                    println!(
                        "\x1b[1;32m[OK]\x1b[0m [{}] syntax and semantics are valid.",
                        files.join(", ")
                    );
                }
                Err(code) => process::exit(code),
            }
        }
        "run" | "interpret" => {
            let (files, _, _, dump_tokens, dump_ast) = parse_cli_args(&args[2..]);
            if files.is_empty() {
                eprintln!(
                    "\x1b[1;31merror\x1b[0m: missing source file(s) for '{}'",
                    command
                );
                process::exit(1);
            }

            let (source, program) = match parse_and_combine_files(&files, dump_tokens, dump_ast) {
                Ok(res) => res,
                Err(code) => process::exit(code),
            };

            let mut interpreter = Interpreter::new(files.join(", "));
            if let Err(diag) = interpreter.load_program(&program) {
                eprintln!("{}", diag.render(&source));
                process::exit(1);
            }

            match interpreter.run_entry() {
                Ok(val) => {
                    if let Value::Int(code) = val
                        && code != 0
                    {
                        process::exit(code as i32);
                    }
                }
                Err(diag) => {
                    eprintln!("{}", diag.render(&source));
                    process::exit(1);
                }
            }
        }
        "translate" => {
            let (files, output_path, _, _, _) = parse_cli_args(&args[2..]);
            if files.is_empty() {
                eprintln!("\x1b[1;31merror\x1b[0m: missing source file(s) for 'translate'");
                process::exit(1);
            }

            let (source, program) = match parse_and_combine_files(&files, false, false) {
                Ok(res) => res,
                Err(code) => process::exit(code),
            };

            let mut translator = Translator::new();
            match translator.translate(&program) {
                Ok(c_code) => {
                    if let Some(out) = output_path {
                        if let Err(e) = fs::write(&out, &c_code) {
                            eprintln!(
                                "\x1b[1;31merror\x1b[0m: failed to write translated code to '{}': {}",
                                out, e
                            );
                            process::exit(1);
                        }
                        println!(
                            "\x1b[1;32m[OK]\x1b[0m Translated [{}] -> '{}'",
                            files.join(", "),
                            out
                        );
                    } else {
                        print!("{}", c_code);
                    }
                }
                Err(diag) => {
                    eprintln!("{}", diag.render(&source));
                    process::exit(1);
                }
            }
        }
        "combine" => {
            let (files, output_path, _, _, _) = parse_cli_args(&args[2..]);
            if files.is_empty() {
                eprintln!("\x1b[1;31merror\x1b[0m: missing source file(s) for 'combine'");
                process::exit(1);
            }

            let (source, program) = match parse_and_combine_files(&files, false, false) {
                Ok(res) => res,
                Err(code) => process::exit(code),
            };

            let mut translator = Translator::new();
            match translator.translate(&program) {
                Ok(c_code) => {
                    if let Some(out) = output_path {
                        if let Err(e) = fs::write(&out, &c_code) {
                            eprintln!(
                                "\x1b[1;31merror\x1b[0m: failed to write combined output to '{}': {}",
                                out, e
                            );
                            process::exit(1);
                        }
                        println!(
                            "\x1b[1;32m[OK]\x1b[0m Combined [{}] -> '{}'",
                            files.join(", "),
                            out
                        );
                    } else {
                        print!("{}", c_code);
                    }
                }
                Err(diag) => {
                    eprintln!("{}", diag.render(&source));
                    process::exit(1);
                }
            }
        }
        "assemble" => {
            let (files, output_path, opt_level, _, _) = parse_cli_args(&args[2..]);
            if files.is_empty() {
                eprintln!("\x1b[1;31merror\x1b[0m: missing source file(s) for 'assemble'");
                process::exit(1);
            }

            let default_out = Path::new(&files[0])
                .with_extension("s")
                .to_string_lossy()
                .to_string();
            let target_out = output_path.unwrap_or(default_out);

            let assembler = Assembler::new(files.join(", "));

            if files.len() == 1 && is_foreign_source(&files[0]) {
                if let Err(diag) = assembler.assemble_source(
                    Path::new(&files[0]),
                    Path::new(&target_out),
                    &opt_level,
                ) {
                    eprintln!("{}", diag.render_standard());
                    process::exit(1);
                }
            } else {
                let (source, program) = match parse_and_combine_files(&files, false, false) {
                    Ok(res) => res,
                    Err(code) => process::exit(code),
                };

                if let Err(diag) =
                    assembler.assemble_to_file(&program, Path::new(&target_out), &opt_level)
                {
                    eprintln!("{}", diag.render(&source));
                    process::exit(1);
                }
            }

            println!(
                "\x1b[1;32m[OK]\x1b[0m Assembled [{}] -> '{}' (-{})",
                files.join(", "),
                target_out,
                opt_level
            );
        }
        "compile" => {
            let (files, output_path, opt_level, _, _) = parse_cli_args(&args[2..]);
            if files.is_empty() {
                eprintln!("\x1b[1;31merror\x1b[0m: missing source file(s) for 'compile'");
                process::exit(1);
            }

            let default_out = Path::new(&files[0])
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("a.out")
                .to_string();
            let target_out = output_path.unwrap_or(default_out);

            let mut carbon_files = Vec::new();
            let mut foreign_files = Vec::new();

            for f in &files {
                if is_foreign_source(f) {
                    foreign_files.push(f.clone());
                } else {
                    carbon_files.push(f.clone());
                }
            }

            let compiler = Compiler::new(files.join(", "));
            let foreign_paths: Vec<&Path> = foreign_files
                .iter()
                .map(|s| Path::new(s.as_str()))
                .collect();

            if carbon_files.is_empty() {
                // Pure foreign source compilation (e.g. C, C++, Assembly)
                if let Err(diag) = compiler.compile_with_sources(
                    None,
                    &foreign_paths,
                    Path::new(&target_out),
                    &opt_level,
                ) {
                    eprintln!("{}", diag.render_standard());
                    process::exit(1);
                }
            } else {
                let (source, program) = match parse_and_combine_files(&carbon_files, false, false) {
                    Ok(res) => res,
                    Err(code) => process::exit(code),
                };

                if let Err(diag) = compiler.compile_with_sources(
                    Some(&program),
                    &foreign_paths,
                    Path::new(&target_out),
                    &opt_level,
                ) {
                    eprintln!("{}", diag.render(&source));
                    process::exit(1);
                }
            }

            println!(
                "\x1b[1;32m[OK]\x1b[0m Compiled [{}] -> '{}' (-{})",
                files.join(", "),
                target_out,
                opt_level
            );
        }
        "revirt" | "vmp-revirt" => {
            let (files, output_path, opt_level, _, _) = parse_cli_args(&args[2..]);
            if files.is_empty() {
                eprintln!("\x1b[1;31merror\x1b[0m: missing source file(s) for 'revirt'");
                process::exit(1);
            }

            let (_, program) = match parse_and_combine_files(&files, false, false) {
                Ok(res) => res,
                Err(code) => process::exit(code),
            };

            let sem_prog = SemProgram::from_ast(&program);
            let revirt = VmpRevirtualizer::new(0x3A5C8E9D);
            let package = revirt.revirtualize(&sem_prog);
            let c_runner = revirt.emit_c_vm_runner(&package);

            if let Some(out) = output_path {
                if out.ends_with(".c") {
                    if let Err(e) = fs::write(&out, &c_runner) {
                        eprintln!(
                            "\x1b[1;31merror\x1b[0m: failed to write revirtualized C code: {}",
                            e
                        );
                        process::exit(1);
                    }
                    println!(
                        "\x1b[1;32m[OK]\x1b[0m Re-virtualized [{}] -> '{}'",
                        files.join(", "),
                        out
                    );
                } else {
                    let temp_dir = std::env::temp_dir();
                    let temp_c = temp_dir.join(format!("vmp_revirt_{}.c", std::process::id()));
                    if let Err(e) = fs::write(&temp_c, &c_runner) {
                        eprintln!(
                            "\x1b[1;31merror\x1b[0m: failed to write temporary file: {}",
                            e
                        );
                        process::exit(1);
                    }
                    let compiler = Compiler::new(files.join(", "));
                    let extra = [temp_c.as_path()];
                    if let Err(diag) =
                        compiler.compile_with_sources(None, &extra, Path::new(&out), &opt_level)
                    {
                        let _ = fs::remove_file(&temp_c);
                        eprintln!("{}", diag.render_standard());
                        process::exit(1);
                    }
                    let _ = fs::remove_file(&temp_c);
                    println!(
                        "\x1b[1;32m[OK]\x1b[0m Re-virtualized & Compiled [{}] -> '{}' (-{})",
                        files.join(", "),
                        out,
                        opt_level
                    );
                }
            } else {
                // Execute in-memory revirtualized bytecode
                if let Some(entry_fn) = package
                    .functions
                    .iter()
                    .find(|f| f.name == "Main" || f.name == "Run")
                {
                    let res = revirt.execute_in_memory(entry_fn);
                    println!(
                        "\x1b[1;32m[OK]\x1b[0m Re-virtualized VM Execution Result: {}",
                        res
                    );
                    if res != 0 {
                        process::exit(res as i32);
                    }
                }
            }
        }
        "zkhypervm" | "zk" => {
            let (files, output_path, _, _, _) = parse_cli_args(&args[2..]);
            if files.is_empty() {
                eprintln!("\x1b[1;31merror\x1b[0m: missing source file(s) for 'zkhypervm'");
                process::exit(1);
            }

            let (_, program) = match parse_and_combine_files(&files, false, false) {
                Ok(res) => res,
                Err(code) => process::exit(code),
            };

            let sem_prog = SemProgram::from_ast(&program);
            let mut zk_vm = ZkHyperVm::new();
            zk_vm.compile_from_sem_ir(&sem_prog);

            let result = zk_vm.execute_and_trace();
            println!("{}", result);

            if let Some(out) = output_path {
                let mut trace_json = String::from("[\n");
                for (i, row) in result.trace.iter().enumerate() {
                    let comma = if i + 1 < result.trace.len() { "," } else { "" };
                    trace_json.push_str(&format!(
                        "  {{\"cycle\": {}, \"pc\": {}, \"opcode\": {:?}, \"r0\": {}, \"r1\": {}, \"verified\": {}}}{}\n",
                        row.cycle, row.pc, row.opcode, row.registers[0], row.registers[1], result.is_verified, comma
                    ));
                }
                trace_json.push_str("]\n");

                if let Err(e) = fs::write(&out, &trace_json) {
                    eprintln!(
                        "\x1b[1;31merror\x1b[0m: failed to write ZK trace file: {}",
                        e
                    );
                    process::exit(1);
                }
                println!(
                    "\x1b[1;32m[OK]\x1b[0m Exported verifiable execution trace to '{}'",
                    out
                );
            }

            if !result.is_verified {
                eprintln!("\x1b[1;31merror\x1b[0m: zk-HyperVM constraint verification failed!");
                process::exit(1);
            }
        }
        "sem-ir" => {
            let (files, output_path, _, _, _) = parse_cli_args(&args[2..]);
            if files.is_empty() {
                eprintln!("\x1b[1;31merror\x1b[0m: missing source file(s) for 'sem-ir'");
                process::exit(1);
            }

            let (_, program) = match parse_and_combine_files(&files, false, false) {
                Ok(res) => res,
                Err(code) => process::exit(code),
            };

            let sem_prog = SemProgram::from_ast(&program);
            let ir_dump = sem_prog.dump_sem_ir();

            if let Some(out) = output_path {
                if let Err(e) = fs::write(&out, &ir_dump) {
                    eprintln!("\x1b[1;31merror\x1b[0m: failed to write SemIR dump: {}", e);
                    process::exit(1);
                }
                println!(
                    "\x1b[1;32m[OK]\x1b[0m Generated SemIR [{}] -> '{}'",
                    files.join(", "),
                    out
                );
            } else {
                print!("{}", ir_dump);
            }
        }
        "format" | "fmt" => {
            let (files, output_path, _, _, _) = parse_cli_args(&args[2..]);
            if files.is_empty() {
                eprintln!("\x1b[1;31merror\x1b[0m: missing source file(s) for 'format'");
                process::exit(1);
            }

            for file in &files {
                let (_, program) = match parse_file_raw(file, false, false) {
                    Ok(res) => res,
                    Err(code) => process::exit(code),
                };

                let mut fmt = Formatter::new();
                let formatted = fmt.format_program(&program);

                if let Some(ref out) = output_path {
                    if let Err(e) = fs::write(out, &formatted) {
                        eprintln!(
                            "\x1b[1;31merror\x1b[0m: failed to write formatted file '{}': {}",
                            out, e
                        );
                        process::exit(1);
                    }
                    println!("\x1b[1;32m[OK]\x1b[0m Formatted [{}] -> '{}'", file, out);
                } else {
                    print!("{}", formatted);
                }
            }
        }
        file_arg if Path::new(file_arg).exists() => {
            let (source, program) = match parse_file(file_arg, false, false) {
                Ok(res) => res,
                Err(code) => process::exit(code),
            };

            let mut interpreter = Interpreter::new(file_arg);
            if let Err(diag) = interpreter.load_program(&program) {
                eprintln!("{}", diag.render(&source));
                process::exit(1);
            }

            match interpreter.run_entry() {
                Ok(val) => {
                    if let Value::Int(code) = val
                        && code != 0
                    {
                        process::exit(code as i32);
                    }
                }
                Err(diag) => {
                    eprintln!("{}", diag.render(&source));
                    process::exit(1);
                }
            }
        }
        unknown => {
            eprintln!("\x1b[1;31merror\x1b[0m: unknown command '{}'", unknown);
            println!("Run 'graphene help' for available commands.");
            process::exit(1);
        }
    }
}
