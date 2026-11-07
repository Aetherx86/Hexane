// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation Language Toolchain
// Licensed under the MIT License.

use carbonation::interpreter::Interpreter;
use carbonation::lexer::Lexer;
use carbonation::parser::Parser;
use carbonation::value::Value;

fn run_carbon(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source, "stress.carbon");
    let tokens = lexer.tokenize().map_err(|e| e.message)?;
    let mut parser = Parser::new(&tokens, "stress.carbon");
    let program = parser.parse_program().map_err(|e| e.message)?;
    let mut interp = Interpreter::new("stress.carbon");
    interp.load_program(&program).map_err(|e| e.message)?;
    interp.run_entry().map_err(|e| e.message)
}

#[test]
fn test_division_by_zero_safe_error() {
    let source = r#"
fn Run() -> i32 {
    var a: i32 = 10;
    var b: i32 = 0;
    return a / b;
}
"#;
    let res = run_carbon(source);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("division by zero"));
}

#[test]
fn test_modulo_by_zero_safe_error() {
    let source = r#"
fn Run() -> i32 {
    var a: i32 = 10;
    var b: i32 = 0;
    return a % b;
}
"#;
    let res = run_carbon(source);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("modulo by zero"));
}

#[test]
fn test_int64_min_overflow_no_panic() {
    // i64::MIN / -1 would panic in debug mode in raw Rust!
    let source = r#"
fn Run() -> i64 {
    var a: i64 = -9223372036854775808;
    var b: i64 = -1;
    return a / b;
}
"#;
    let res = run_carbon(source);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), Value::Int(i64::MIN));
}

#[test]
fn test_shift_overflow_no_panic() {
    // Shift by >= 64 would panic in raw Rust!
    let source = r#"
fn Run() -> i32 {
    var a: i32 = 1;
    var b: i32 = 68; // 68 % 64 = 4 -> 1 << 4 = 16
    return a << b;
}
"#;
    let res = run_carbon(source);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), Value::Int(16));
}

#[test]
fn test_recursion_depth_limit() {
    let source = r#"
fn InfiniteRecursion() {
    InfiniteRecursion();
}

fn Run() {
    InfiniteRecursion();
}
"#;
    let res = run_carbon(source);
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .contains("maximum recursion depth exceeded")
    );
}

#[test]
fn test_factorial_iterative_and_recursive() {
    let source = r#"
fn FactRec(n: i32) -> i32 {
    if (n <= 1) {
        return 1;
    }
    return n * FactRec(n - 1);
}

fn FactIter(n: i32) -> i32 {
    var res: i32 = 1;
    var i: i32 = 2;
    while (i <= n) {
        res *= i;
        i += 1;
    }
    return res;
}

fn Run() -> i32 {
    var r1: i32 = FactRec(6); // 720
    var r2: i32 = FactIter(6); // 720
    if (r1 == r2 && r1 == 720) {
        return 1;
    }
    return 0;
}
"#;
    let res = run_carbon(source).unwrap();
    assert_eq!(res, Value::Int(1));
}

#[test]
fn test_euclid_gcd() {
    let source = r#"
fn Gcd(a: i32, b: i32) -> i32 {
    while (b != 0) {
        var t: i32 = b;
        b = a % b;
        a = t;
    }
    return a;
}

fn Run() -> i32 {
    return Gcd(1071, 462); // 21
}
"#;
    let res = run_carbon(source).unwrap();
    assert_eq!(res, Value::Int(21));
}

#[test]
fn test_array_stress_and_accumulation() {
    let source = r#"
import Core library "io";

fn Run() -> i64 {
    var arr: array(i32, 1000);
    for (i: i32 in Core.Range(1000)) {
        arr[i] = i * 2;
    }

    var total: i64 = 0;
    for (i: i32 in Core.Range(1000)) {
        total += arr[i];
    }
    return total; // 2 * (999 * 1000 / 2) = 999,000
}
"#;
    let res = run_carbon(source).unwrap();
    assert_eq!(res, Value::Int(999_000));
}

#[test]
fn test_bubble_sort_in_place() {
    let source = r#"
fn Run() -> i32 {
    var arr: array(i32, 5);
    arr[0] = 64;
    arr[1] = 34;
    arr[2] = 25;
    arr[3] = 12;
    arr[4] = 22;

    var n: i32 = 5;
    var i: i32 = 0;
    while (i < n - 1) {
        var j: i32 = 0;
        while (j < n - i - 1) {
            if (arr[j] > arr[j + 1]) {
                var temp: i32 = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
            j += 1;
        }
        i += 1;
    }

    // Array should now be: 12, 22, 25, 34, 64
    if (arr[0] == 12 && arr[1] == 22 && arr[2] == 25 && arr[3] == 34 && arr[4] == 64) {
        return 1;
    }
    return 0;
}
"#;
    let res = run_carbon(source).unwrap();
    assert_eq!(res, Value::Int(1));
}

#[test]
fn test_array_out_of_bounds_safe_error() {
    let source = r#"
fn Run() -> i32 {
    var arr: array(i32, 5);
    return arr[10]; // Out of bounds
}
"#;
    let res = run_carbon(source);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("array index out of bounds"));
}

#[test]
fn test_struct_mutation_with_ref_self() {
    let source = r#"
class BankAccount {
    var balance: i32;

    fn Deposit(ref self, amount: i32) {
        self.balance += amount;
    }

    fn Withdraw(ref self, amount: i32) -> bool {
        if (self.balance >= amount) {
            self.balance -= amount;
            return true;
        }
        return false;
    }
}

fn Run() -> i32 {
    var account: BankAccount;
    account.balance = 100;
    account.Deposit(50); // 150
    var ok: bool = account.Withdraw(70); // 80 remaining
    if (ok && account.balance == 80) {
        return 1;
    }
    return 0;
}
"#;
    let res = run_carbon(source).unwrap();
    assert_eq!(res, Value::Int(1));
}
