// Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
package ZkProofDemo api;

// Verifiable arithmetic program targeted for zk-HyperVM execution and trace generation

fn Main() -> i32 {
  var secret_x: i32 = 42;
  var public_multiplier: i32 = 137;
  var commitment: i32 = secret_x * public_multiplier;
  return commitment;
}
