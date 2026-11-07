// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation Language Toolchain
// Licensed under the MIT License.

use carbonation::compiler::Compiler;
use carbonation::interpreter::Interpreter;
use carbonation::parse_and_combine_files;
use carbonation::translator::Translator;
use carbonation::value::Value;
use std::fs;
use std::process::Command;

#[test]
fn test_multi_file_interpreting_functions() {
    let dir = std::env::temp_dir();
    let f1 = dir.join(format!("math_{}.carbon", std::process::id()));
    let f2 = dir.join(format!("main_{}.carbon", std::process::id()));

    fs::write(
        &f1,
        "fn Multiply(a: i32, b: i32) -> i32 { return a * b; }\n",
    )
    .unwrap();
    fs::write(&f2, "fn Main() -> i32 { return Multiply(6, 7); }\n").unwrap();

    let files = vec![
        f1.to_string_lossy().to_string(),
        f2.to_string_lossy().to_string(),
    ];
    let (_, program) = parse_and_combine_files(&files, false, false).unwrap();

    let mut interp = Interpreter::new("multi_test");
    interp.load_program(&program).unwrap();
    let val = interp.run_entry().unwrap();

    let _ = fs::remove_file(&f1);
    let _ = fs::remove_file(&f2);

    assert_eq!(val, Value::Int(42));
}

#[test]
fn test_multi_file_class_sharing() {
    let dir = std::env::temp_dir();
    let f1 = dir.join(format!("person_{}.carbon", std::process::id()));
    let f2 = dir.join(format!("app_{}.carbon", std::process::id()));

    let class_src = r#"
class Person {
    var age: i32;
    fn Make(initial_age: i32) -> Person {
        var p: Person;
        p.age = initial_age;
        return p;
    }
    fn GetAge(ref self) -> i32 {
        return self.age;
    }
}
"#;
    let app_src = r#"
fn Main() -> i32 {
    var person: Person = Person.Make(25);
    return person.GetAge();
}
"#;
    fs::write(&f1, class_src).unwrap();
    fs::write(&f2, app_src).unwrap();

    let files = vec![
        f1.to_string_lossy().to_string(),
        f2.to_string_lossy().to_string(),
    ];
    let (_, program) = parse_and_combine_files(&files, false, false).unwrap();

    let temp_bin = dir.join(format!("multi_person_bin_{}", std::process::id()));
    let compiler = Compiler::new("multi_person");
    compiler
        .compile_to_binary(&program, &temp_bin, "O3")
        .unwrap();

    let output = Command::new(&temp_bin).output().unwrap();

    let _ = fs::remove_file(&f1);
    let _ = fs::remove_file(&f2);
    let _ = fs::remove_file(&temp_bin);

    assert_eq!(output.status.code(), Some(25));
}

#[test]
fn test_multi_file_translator_c_output() {
    let dir = std::env::temp_dir();
    let f1 = dir.join(format!("mod1_{}.carbon", std::process::id()));
    let f2 = dir.join(format!("mod2_{}.carbon", std::process::id()));

    fs::write(&f1, "fn StepOne() -> i32 { return 10; }\n").unwrap();
    fs::write(&f2, "fn Main() -> i32 { return StepOne() + 5; }\n").unwrap();

    let files = vec![
        f1.to_string_lossy().to_string(),
        f2.to_string_lossy().to_string(),
    ];
    let (_, program) = parse_and_combine_files(&files, false, false).unwrap();

    let mut translator = Translator::new();
    let c_code = translator.translate(&program).unwrap();

    let _ = fs::remove_file(&f1);
    let _ = fs::remove_file(&f2);

    assert!(c_code.contains("int32_t StepOne(void);"));
    assert!(c_code.contains("int32_t Main(void);"));
    assert!(c_code.contains("return (StepOne() + 5);"));
}
