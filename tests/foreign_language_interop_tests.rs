// Copyright (c) 2026 jk9576. All rights reserved.
// Carbonation Language Toolchain
// Licensed under the MIT License.

use std::fs;
use std::process::Command;

fn get_bin_path(bin_name: &str) -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push(bin_name);
    path
}

#[test]
fn test_cff_gcd_devirtualization_interop() {
    let temp_dir = std::env::temp_dir();
    let c_path = temp_dir.join("interop_cff_gcd.c");
    let carbon_path = temp_dir.join("interop_cff_main.carbon");
    let bin_path = temp_dir.join("interop_cff_app");

    // Real Control Flow Flattening (CFF) state machine with opaque predicates
    // inspired by VMP_DEVIRT/tests/test_cff.c
    let c_code = r#"
#include <stdint.h>

uint32_t flattened_gcd(uint32_t a, uint32_t b) {
    uint32_t state = 0x1A2B;
    uint32_t t = 0;
    uint32_t opaque_var = a;

    while (state != 0x9999) {
        switch (state) {
            case 0x1A2B: // Header: check loop condition
                if ((opaque_var * (opaque_var + 1)) % 2 == 0) {
                    if (b != 0) {
                        state = 0x3C4D;
                    } else {
                        state = 0x5E6F;
                    }
                } else {
                    state = 0xDEAD;
                }
                break;

            case 0x3C4D: // Body 1: t = b
                t = b;
                state = 0x7A8B;
                break;

            case 0x7A8B: // Body 2: b = a % b; a = t
                b = a % b;
                a = t;
                state = 0x1A2B;
                break;

            case 0x5E6F: // Exit block
                state = 0x9999;
                break;

            case 0xDEAD: // Bogus unreachable trap block
                a = a ^ 0xCAFEBABE;
                state = 0x9999;
                break;

            default:
                state = 0x9999;
                break;
        }
    }
    return a;
}
"#;

    let carbon_code = r#"
package InteropCff api;

extern fn flattened_gcd(a: i32, b: i32) -> i32;

fn Main() -> i32 {
  // Test GCD(1071, 462) = 21
  var g1: i32 = flattened_gcd(1071, 462);
  if (g1 != 21) {
    return 1;
  }

  // Test GCD(48, 18) = 6
  var g2: i32 = flattened_gcd(48, 18);
  if (g2 != 6) {
    return 2;
  }

  // Test GCD(101, 103) = 1 (primes)
  var g3: i32 = flattened_gcd(101, 103);
  if (g3 != 1) {
    return 3;
  }

  return 0;
}
"#;

    fs::write(&c_path, c_code).unwrap();
    fs::write(&carbon_path, carbon_code).unwrap();

    let carbonation = get_bin_path("carbonation");
    let status = Command::new(&carbonation)
        .arg("compile")
        .arg(&carbon_path)
        .arg(&c_path)
        .arg("-o")
        .arg(&bin_path)
        .arg("-O3")
        .status()
        .expect("failed to run carbonation compile");

    assert!(
        status.success(),
        "carbonation compile failed for CFF C interop"
    );

    let run_status = Command::new(&bin_path)
        .status()
        .expect("failed to run compiled binary");

    assert_eq!(
        run_status.code(),
        Some(0),
        "CFF GCD binary exited with failure code"
    );

    let _ = fs::remove_file(&c_path);
    let _ = fs::remove_file(&carbon_path);
    let _ = fs::remove_file(&bin_path);
}

#[test]
fn test_mba_exhaustive_interop() {
    let temp_dir = std::env::temp_dir();
    let c_path = temp_dir.join("interop_mba.c");
    let carbon_path = temp_dir.join("interop_mba_main.carbon");
    let bin_path = temp_dir.join("interop_mba_app");

    // Real Mixed Boolean-Arithmetic (MBA) obfuscation over Z_{2^8}
    // from VMP_DEVIRT/tests/test_mba.c
    let c_code = r#"
#include <stdint.h>

int32_t canonical_secret(int32_t x, int32_t y) {
    return (int32_t)(int8_t)(((int8_t)x ^ 92) + (int8_t)y);
}

int32_t obfuscated_mba_secret(int32_t x_in, int32_t y_in) {
    int x = (int)(int8_t)x_in;
    int y = (int)(int8_t)y_in;
    int a = 229 * x + 247;
    int b = 237 * a + 214 + ((38 * a + 85) & 254);
    int c = (b + ((-(2 * b) + 255) & 254)) * 3 + 77;
    int d = ((86 * c + 36) & 70) * 75 + 231 * c + 118;
    int e = ((58 * d + 175) & 244) + 99 * d + 46;
    int f = (e & 148);
    int g = (f - (e & 255) + f) * 103 + 13;
    int r = (237 * (45 * g + (174 * g | 34) * 229 + 194 - 247) & 255) + y;
    return (int32_t)(int8_t)r;
}
"#;

    let carbon_code = r#"
package InteropMba api;

extern fn canonical_secret(x: i32, y: i32) -> i32;
extern fn obfuscated_mba_secret(x: i32, y: i32) -> i32;

fn Main() -> i32 {
  // Test a sweep of inputs across the domain
  var x: i32 = -50;
  while (x <= 50) {
    var y: i32 = -50;
    while (y <= 50) {
      var c: i32 = canonical_secret(x, y);
      var o: i32 = obfuscated_mba_secret(x, y);
      if (c != o) {
        return 1;
      }
      y = y + 15;
    }
    x = x + 15;
  }
  return 0;
}
"#;

    fs::write(&c_path, c_code).unwrap();
    fs::write(&carbon_path, carbon_code).unwrap();

    let carbonation = get_bin_path("carbonation");
    let status = Command::new(&carbonation)
        .arg("compile")
        .arg(&carbon_path)
        .arg(&c_path)
        .arg("-o")
        .arg(&bin_path)
        .arg("-O3")
        .status()
        .expect("failed to run carbonation compile");

    assert!(
        status.success(),
        "carbonation compile failed for MBA interop"
    );

    let run_status = Command::new(&bin_path)
        .status()
        .expect("failed to run compiled binary");

    assert_eq!(
        run_status.code(),
        Some(0),
        "MBA binary detected mismatch in deobfuscated semantics"
    );

    let _ = fs::remove_file(&c_path);
    let _ = fs::remove_file(&carbon_path);
    let _ = fs::remove_file(&bin_path);
}

#[test]
fn test_cpp_class_and_templates_interop() {
    let temp_dir = std::env::temp_dir();
    let cpp_path = temp_dir.join("interop_vector.cpp");
    let carbon_path = temp_dir.join("interop_cpp_main.carbon");
    let bin_path = temp_dir.join("interop_cpp_app");

    let cpp_code = r#"
#include <cmath>
#include <stdint.h>

template <typename T>
class Vector3D {
public:
    T x, y, z;
    Vector3D(T x, T y, T z) : x(x), y(y), z(z) {}
    T norm_sq() const { return x * x + y * y + z * z; }
};

extern "C" {
    int32_t cpp_vector3_norm_sq(int32_t x, int32_t y, int32_t z) {
        Vector3D<int32_t> v(x, y, z);
        return v.norm_sq();
    }

    int32_t cpp_dot_product(int32_t x1, int32_t y1, int32_t z1,
                            int32_t x2, int32_t y2, int32_t z2) {
        return x1 * x2 + y1 * y2 + z1 * z2;
    }
}
"#;

    let carbon_code = r#"
package InteropCpp api;

extern fn cpp_vector3_norm_sq(x: i32, y: i32, z: i32) -> i32;
extern fn cpp_dot_product(x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32) -> i32;

fn Main() -> i32 {
  // Vector (3, 4, 12) -> norm_sq = 9 + 16 + 144 = 169 (13^2)
  var n: i32 = cpp_vector3_norm_sq(3, 4, 12);
  if (n != 169) {
    return 1;
  }

  // Dot product (1, 2, 3) . (4, 5, 6) = 4 + 10 + 18 = 32
  var dot: i32 = cpp_dot_product(1, 2, 3, 4, 5, 6);
  if (dot != 32) {
    return 2;
  }

  return 0;
}
"#;

    fs::write(&cpp_path, cpp_code).unwrap();
    fs::write(&carbon_path, carbon_code).unwrap();

    let carbonation = get_bin_path("carbonation");
    let status = Command::new(&carbonation)
        .arg("compile")
        .arg(&carbon_path)
        .arg(&cpp_path)
        .arg("-o")
        .arg(&bin_path)
        .arg("-O2")
        .status()
        .expect("failed to run carbonation compile for C++");

    assert!(
        status.success(),
        "carbonation compile failed for C++ interop"
    );

    let run_status = Command::new(&bin_path)
        .status()
        .expect("failed to run compiled binary");

    assert_eq!(
        run_status.code(),
        Some(0),
        "C++ interop binary exited with failure code"
    );

    let _ = fs::remove_file(&cpp_path);
    let _ = fs::remove_file(&carbon_path);
    let _ = fs::remove_file(&bin_path);
}

#[test]
fn test_native_assembly_interop() {
    let temp_dir = std::env::temp_dir();
    let asm_path = temp_dir.join("interop_fast_math.S");
    let carbon_path = temp_dir.join("interop_asm_main.carbon");
    let bin_path = temp_dir.join("interop_asm_app");

    // Multi-architecture assembly file (ARM64 and x86_64 compatible)
    let asm_code = r#"
.text
#if defined(__arm64__) || defined(__aarch64__)
.globl _asm_fast_mult_add
.globl asm_fast_mult_add
.p2align 2
_asm_fast_mult_add:
asm_fast_mult_add:
    madd w0, w0, w1, w2
    ret
#elif defined(__x86_64__)
.globl _asm_fast_mult_add
.globl asm_fast_mult_add
_asm_fast_mult_add:
asm_fast_mult_add:
    imull %esi, %edi
    addl %edx, %edi
    movl %edi, %eax
    ret
#else
#error Unsupported architecture for test assembly
#endif
"#;

    let carbon_code = r#"
package InteropAsm api;

extern fn asm_fast_mult_add(a: i32, b: i32, c: i32) -> i32;

fn Main() -> i32 {
  // 6 * 7 + 8 = 50
  var res: i32 = asm_fast_mult_add(6, 7, 8);
  if (res == 50) {
    return 0;
  }
  return 1;
}
"#;

    fs::write(&asm_path, asm_code).unwrap();
    fs::write(&carbon_path, carbon_code).unwrap();

    let carbonation = get_bin_path("carbonation");
    let status = Command::new(&carbonation)
        .arg("compile")
        .arg(&carbon_path)
        .arg(&asm_path)
        .arg("-o")
        .arg(&bin_path)
        .status()
        .expect("failed to run carbonation compile for assembly");

    assert!(
        status.success(),
        "carbonation compile failed for assembly interop"
    );

    let run_status = Command::new(&bin_path)
        .status()
        .expect("failed to run compiled binary");

    assert_eq!(
        run_status.code(),
        Some(0),
        "Assembly interop binary exited with failure code"
    );

    let _ = fs::remove_file(&asm_path);
    let _ = fs::remove_file(&carbon_path);
    let _ = fs::remove_file(&bin_path);
}

#[test]
fn test_pure_foreign_c_direct_compilation() {
    let temp_dir = std::env::temp_dir();
    let c_path = temp_dir.join("pure_foreign_test.c");
    let bin_path = temp_dir.join("pure_foreign_app");

    let c_code = r#"
#include <stdio.h>
#include <stdint.h>

int main(void) {
    int sum = 0;
    for (int i = 1; i <= 100; ++i) {
        sum += i;
    }
    // Sum from 1 to 100 is 5050
    if (sum == 5050) {
        return 0;
    }
    return 1;
}
"#;

    fs::write(&c_path, c_code).unwrap();

    // Compile pure C file directly using carbonation CLI
    let carbonation = get_bin_path("carbonation");
    let status = Command::new(&carbonation)
        .arg("compile")
        .arg(&c_path)
        .arg("-o")
        .arg(&bin_path)
        .arg("-O3")
        .status()
        .expect("failed to run carbonation compile on pure C file");

    assert!(status.success(), "carbonation direct C compilation failed");

    let run_status = Command::new(&bin_path)
        .status()
        .expect("failed to run pure C compiled binary");

    assert_eq!(run_status.code(), Some(0));

    let _ = fs::remove_file(&c_path);
    let _ = fs::remove_file(&bin_path);
}

#[test]
fn test_all_four_languages_unified_pipeline() {
    let temp_dir = std::env::temp_dir();
    let carbon_path = temp_dir.join("quad_main.carbon");
    let c_path = temp_dir.join("quad_c_layer.c");
    let cpp_path = temp_dir.join("quad_cpp_layer.cpp");
    let asm_path = temp_dir.join("quad_asm_layer.S");
    let bin_path = temp_dir.join("quad_unified_app");

    let asm_code = r#"
.text
#if defined(__arm64__) || defined(__aarch64__)
.globl _asm_triple
.globl asm_triple
_asm_triple:
asm_triple:
    mov w1, #3
    mul w0, w0, w1
    ret
#elif defined(__x86_64__)
.globl _asm_triple
.globl asm_triple
_asm_triple:
asm_triple:
    imull $3, %edi, %eax
    ret
#endif
"#;

    let c_code = r#"
#include <stdint.h>

int32_t c_bitwise_rotate_left(int32_t val, int32_t shift) {
    uint32_t u = (uint32_t)val;
    uint32_t s = (uint32_t)(shift & 31);
    return (int32_t)((u << s) | (u >> (32 - s)));
}
"#;

    let cpp_code = r#"
#include <string>
#include <sstream>

extern "C" {
    int32_t cpp_hex_string_len(int32_t val) {
        std::stringstream ss;
        ss << std::hex << val;
        return (int32_t)ss.str().length();
    }
}
"#;

    let carbon_code = r#"
package QuadPipeline api;

extern fn asm_triple(val: i32) -> i32;
extern fn c_bitwise_rotate_left(val: i32, shift: i32) -> i32;
extern fn cpp_hex_string_len(val: i32) -> i32;

fn Main() -> i32 {
  // Assembly test: 14 * 3 = 42
  var v_asm: i32 = asm_triple(14);
  if (v_asm != 42) {
    return 1;
  }

  // C test: rotate 1 left by 4 = 16
  var v_c: i32 = c_bitwise_rotate_left(1, 4);
  if (v_c != 16) {
    return 2;
  }

  // C++ test: hex representation of 255 ("ff") has length 2
  var v_cpp: i32 = cpp_hex_string_len(255);
  if (v_cpp != 2) {
    return 3;
  }

  return 0;
}
"#;

    fs::write(&asm_path, asm_code).unwrap();
    fs::write(&c_path, c_code).unwrap();
    fs::write(&cpp_path, cpp_code).unwrap();
    fs::write(&carbon_path, carbon_code).unwrap();

    let carbonation = get_bin_path("carbonation");
    let status = Command::new(&carbonation)
        .arg("compile")
        .arg(&carbon_path)
        .arg(&c_path)
        .arg(&cpp_path)
        .arg(&asm_path)
        .arg("-o")
        .arg(&bin_path)
        .arg("-O3")
        .status()
        .expect("failed to compile 4-language pipeline");

    assert!(status.success(), "quad pipeline compilation failed");

    let run_status = Command::new(&bin_path)
        .status()
        .expect("failed to run quad pipeline binary");

    assert_eq!(
        run_status.code(),
        Some(0),
        "quad pipeline binary exited with failure code"
    );

    let _ = fs::remove_file(&asm_path);
    let _ = fs::remove_file(&c_path);
    let _ = fs::remove_file(&cpp_path);
    let _ = fs::remove_file(&carbon_path);
    let _ = fs::remove_file(&bin_path);
}

#[test]
fn test_interpreter_safe_extern_handling() {
    use carbonation::interpreter::Interpreter;
    use carbonation::parse_file_raw;

    // Test builtins execution in interpreter
    let src = r#"
package BuiltinsTest api;

extern fn abs(x: i32) -> i32;

fn Main() -> i32 {
  var a: i32 = abs(-42);
  if (a == 42) {
    return 0;
  }
  return 1;
}
"#;

    let temp_dir = std::env::temp_dir();
    let carbon_path = temp_dir.join("test_builtins.carbon");
    fs::write(&carbon_path, src).unwrap();

    let (_, program) = parse_file_raw(carbon_path.to_str().unwrap(), false, false).unwrap();
    let mut interp = Interpreter::new(carbon_path.to_str().unwrap());
    interp.load_program(&program).unwrap();
    let res = interp.run_entry().unwrap();
    assert_eq!(res, carbonation::value::Value::Int(0));

    // Test unknown extern returns safe diagnostic (no panic, no segfault)
    let src_unlinked = r#"
package UnlinkedTest api;

extern fn some_unlinked_c_func(x: i32) -> i32;

fn Main() -> i32 {
  return some_unlinked_c_func(10);
}
"#;

    let unlinked_path = temp_dir.join("test_unlinked.carbon");
    fs::write(&unlinked_path, src_unlinked).unwrap();

    let (_, prog_unlinked) = parse_file_raw(unlinked_path.to_str().unwrap(), false, false).unwrap();
    let mut interp2 = Interpreter::new(unlinked_path.to_str().unwrap());
    interp2.load_program(&prog_unlinked).unwrap();
    let err = interp2.run_entry().unwrap_err();
    assert!(
        err.message.contains("cannot interpret external function"),
        "expected safe diagnostic message, got: {}",
        err.message
    );

    let _ = fs::remove_file(&carbon_path);
    let _ = fs::remove_file(&unlinked_path);
}

#[test]
fn test_assemble_foreign_c_to_native_s_and_o() {
    let temp_dir = std::env::temp_dir();
    let c_path = temp_dir.join("foreign_asm_test.c");
    let s_path = temp_dir.join("foreign_asm_test.s");
    let o_path = temp_dir.join("foreign_asm_test.o");

    let c_code = r#"
int compute_square(int x) {
    return x * x;
}
"#;

    fs::write(&c_path, c_code).unwrap();

    let carbonation = get_bin_path("carbonation");

    // Test -S on foreign C file
    let s_status = Command::new(&carbonation)
        .arg("-S")
        .arg(&c_path)
        .status()
        .expect("failed to run -S");
    assert!(s_status.success());
    assert!(s_path.exists());
    let s_content = fs::read_to_string(&s_path).unwrap();
    assert!(s_content.contains("compute_square"));

    // Test -c on foreign C file
    let c_status = Command::new(&carbonation)
        .arg("-c")
        .arg(&c_path)
        .status()
        .expect("failed to run -c");
    assert!(c_status.success());
    assert!(o_path.exists());

    let _ = fs::remove_file(&c_path);
    let _ = fs::remove_file(&s_path);
    let _ = fs::remove_file(&o_path);
}
