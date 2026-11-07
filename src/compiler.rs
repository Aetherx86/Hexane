// Copyright (c) 2026 jk9576. All rights reserved.
// Graphene Language Toolchain
// Licensed under the MIT License.

use crate::ast::Program;
use crate::diagnostics::{Diagnostic, Span};
use crate::translator::Translator;
use std::fs;
use std::path::Path;
use std::process::Command;

use std::sync::atomic::{AtomicUsize, Ordering};
static COMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct Compiler {
    filename: String,
}

impl Compiler {
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
        }
    }

    pub fn compile_with_sources(
        &self,
        program: Option<&Program>,
        extra_sources: &[impl AsRef<Path>],
        output_path: &Path,
        opt_level: &str,
    ) -> Result<(), Diagnostic> {
        let mut temp_c = None;
        if let Some(prog) = program {
            let mut translator = Translator::new();
            let c_code = translator.translate(prog)?;

            let temp_dir = std::env::temp_dir();
            let count = COMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let p = temp_dir.join(format!(
                "carbonation_build_{}_{}.c",
                std::process::id(),
                count
            ));

            fs::write(&p, &c_code).map_err(|e| {
                Diagnostic::error(
                    &self.filename,
                    Span::dummy(),
                    format!("failed to write temporary source file: {}", e),
                )
            })?;
            temp_c = Some(p);
        }

        let has_cpp = extra_sources.iter().any(|s| {
            let path = s.as_ref();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                matches!(ext.to_lowercase().as_str(), "cpp" | "cc" | "cxx" | "hpp")
            } else {
                false
            }
        });

        let compiler_bin = if has_cpp { "clang++" } else { "clang" };
        let mut cmd = Command::new(compiler_bin);
        cmd.arg(format!("-{}", opt_level)).arg("-Wall").arg("-I.");

        if let Some(ref c_file) = temp_c {
            cmd.arg("-x").arg("c").arg(c_file);
        }

        for extra in extra_sources {
            let path = extra.as_ref();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext.to_lowercase().as_str() {
                    "cpp" | "cc" | "cxx" => {
                        cmd.arg("-x").arg("c++").arg(path);
                    }
                    "c" => {
                        cmd.arg("-x").arg("c").arg(path);
                    }
                    "s" | "asm" => {
                        if ext == "S" {
                            cmd.arg("-x").arg("assembler-with-cpp").arg(path);
                        } else {
                            cmd.arg("-x").arg("assembler").arg(path);
                        }
                    }
                    _ => {
                        cmd.arg("-x").arg("none").arg(path);
                    }
                }
            } else {
                cmd.arg("-x").arg("none").arg(path);
            }
        }
        cmd.arg("-x").arg("none");

        cmd.arg("-lm").arg("-o").arg(output_path);

        let output = cmd.output().map_err(|e| {
            if let Some(ref c_file) = temp_c {
                let _ = fs::remove_file(c_file);
            }
            Diagnostic::error(
                &self.filename,
                Span::dummy(),
                format!("failed to invoke compiler ({}): {}", compiler_bin, e),
            )
        })?;

        if let Some(ref c_file) = temp_c {
            let _ = fs::remove_file(c_file);
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Diagnostic::error(
                &self.filename,
                Span::dummy(),
                format!("compilation failed:\n{}", stderr),
            ));
        }

        Ok(())
    }

    pub fn compile_to_binary(
        &self,
        program: &Program,
        output_path: &Path,
        opt_level: &str,
    ) -> Result<(), Diagnostic> {
        let empty: [&Path; 0] = [];
        self.compile_with_sources(Some(program), &empty, output_path, opt_level)
    }
}
