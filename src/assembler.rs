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
static ASM_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct Assembler {
    filename: String,
}

impl Assembler {
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
        }
    }

    pub fn assemble_to_file(
        &self,
        program: &Program,
        output_path: &Path,
        opt_level: &str,
    ) -> Result<(), Diagnostic> {
        let mut translator = Translator::new();
        let c_code = translator.translate(program)?;

        // Write to a uniquely named temporary C file per thread/invocation
        let temp_dir = std::env::temp_dir();
        let count = ASM_COUNTER.fetch_add(1, Ordering::Relaxed);
        let temp_c = temp_dir.join(format!(
            "carbonation_asm_{}_{}.c",
            std::process::id(),
            count
        ));

        fs::write(&temp_c, &c_code).map_err(|e| {
            Diagnostic::error(
                &self.filename,
                Span::dummy(),
                format!(
                    "failed to write temporary file '{}': {}",
                    temp_c.display(),
                    e
                ),
            )
        })?;

        let is_object = output_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext == "o")
            .unwrap_or(false);

        let mut cmd = Command::new("clang");
        if is_object {
            cmd.arg("-c");
        } else {
            cmd.arg("-S"); // emit assembly
        }

        cmd.arg(format!("-{}", opt_level))
            .arg("-std=c99")
            .arg("-Wall")
            .arg(&temp_c)
            .arg("-o")
            .arg(output_path);

        let output = cmd.output().map_err(|e| {
            let _ = fs::remove_file(&temp_c);
            Diagnostic::error(
                &self.filename,
                Span::dummy(),
                format!("failed to execute clang assembler: {}", e),
            )
        })?;

        let _ = fs::remove_file(&temp_c);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Diagnostic::error(
                &self.filename,
                Span::dummy(),
                format!("assembly error:\n{}", stderr),
            ));
        }

        Ok(())
    }

    pub fn assemble_source(
        &self,
        input_source: &Path,
        output_path: &Path,
        opt_level: &str,
    ) -> Result<(), Diagnostic> {
        let is_object = output_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext == "o")
            .unwrap_or(false);

        let has_cpp = input_source
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| matches!(ext.to_lowercase().as_str(), "cpp" | "cc" | "cxx"))
            .unwrap_or(false);

        let compiler_bin = if has_cpp { "clang++" } else { "clang" };
        let mut cmd = Command::new(compiler_bin);
        if is_object {
            cmd.arg("-c");
        } else {
            cmd.arg("-S");
        }

        cmd.arg(format!("-{}", opt_level))
            .arg("-Wall")
            .arg("-I.")
            .arg(input_source)
            .arg("-o")
            .arg(output_path);

        let output = cmd.output().map_err(|e| {
            Diagnostic::error(
                &self.filename,
                Span::dummy(),
                format!("failed to execute clang assembler: {}", e),
            )
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Diagnostic::error(
                &self.filename,
                Span::dummy(),
                format!("assembly error:\n{}", stderr),
            ));
        }

        Ok(())
    }
}
