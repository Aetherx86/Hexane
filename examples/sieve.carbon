// Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
import Core library "io";

// Compute and return the number of primes less than 1000.

class Sieve {
  impl as Core.UnformedInit {}

  fn Make() -> Sieve {
    returned var s: Sieve;
    for (n: i32 in Core.Range(1000)) {
      s.is_prime[n] = true;
    }
    return var;
  }

  fn MarkMultiplesNotPrime(ref self, p: i32) {
    var n: i32 = p * 2;
    while (n < 1000) {
      self.is_prime[n] = false;
      n += p;
    }
  }

  var is_prime: array(bool, 1000);
}

fn Run() -> i32 {
  var s: Sieve = Sieve.Make();

  var number_of_primes: i32 = 0;
  for (n: i32 in Core.InclusiveRange(2, 999)) {
    if (s.is_prime[n]) {
      ++number_of_primes;
      Core.Print(n);
      s.MarkMultiplesNotPrime(n);
    }
  }

  return number_of_primes;
}
