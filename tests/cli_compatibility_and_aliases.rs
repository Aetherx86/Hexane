// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation Language Toolchain
// Licensed under the MIT License.

use std::io::Write;
use std::process::{Command, Stdio};

fn get_bin_path(name: &str) -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    if path.ends_with("deps") {
        path.pop();
    }
    path.join(name)
}

#[test]
fn test_all_four_binary_aliases_exist_and_run() {
    for bin_name in &["carbonation", "graphene", "hexane", "hexene"] {
        let bin = get_bin_path(bin_name);
        let output = Command::new(&bin)
            .arg("--version")
            .output()
            .unwrap_or_else(|_| panic!("failed to execute binary '{}' at {:?}", bin_name, bin));

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Carbonation Toolchain"));
        assert!(stdout.contains("jk9576"));
    }
}

#[test]
fn test_pipe_stdin_support() {
    let bin = get_bin_path("carbonation");
    let mut child = Command::new(&bin)
        .arg("run")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let input = r#"
import Core library "io";

fn Run() -> i32 {
    Core.PrintStr("Piped execution works!\n");
    return 123;
}
"#;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(123));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Piped execution works!"));
}

#[test]
fn test_compiler_dash_e_flag_pipeline() {
    let bin = get_bin_path("carbonation");
    let mut child = Command::new(&bin)
        .arg("-E")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let input = "fn Run() -> i32 { return 99; }";
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("#include <stdio.h>"));
    assert!(stdout.contains("int32_t Run(void)"));
}

#[test]
fn test_error_exit_code_on_syntax_error() {
    let bin = get_bin_path("carbonation");
    let mut child = Command::new(&bin)
        .arg("run")
        .arg("-")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let bad_input = "fn Run( { return 0; }";
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(bad_input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"));
    assert!(stderr.contains("expected"));
}
