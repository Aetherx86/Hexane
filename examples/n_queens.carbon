// Copyright (c) 2026 jk9576. All rights reserved.
// Hexane / Carbonation Language Toolchain
// The N-Queens Algorithmic Benchmark

package Queens api;

fn Abs(x: i32) -> i32 {
  if (x < 0) {
    return -x;
  }
  return x;
}

fn SolveQueens(n: i32) -> i32 {
  var board: array(i32, 16);
  var solutions: i32 = 0;
  var row: i32 = 0;
  board[0] = -1;

  while (row >= 0) {
    board[row] = board[row] + 1;

    while (board[row] < n) {
      var conflict: i32 = 0;
      var i: i32 = 0;
      while (i < row) {
        if (board[i] == board[row]) {
          conflict = 1;
        }
        var diff_row: i32 = row - i;
        var diff_col: i32 = Abs(board[row] - board[i]);
        if (diff_row == diff_col) {
          conflict = 1;
        }
        i = i + 1;
      }

      if (conflict == 0) {
        break;
      }
      board[row] = board[row] + 1;
    }

    if (board[row] < n) {
      if (row == n - 1) {
        solutions = solutions + 1;
      } else {
        row = row + 1;
        board[row] = -1;
      }
    } else {
      row = row - 1;
    }
  }

  return solutions;
}

fn Main() -> i32 {
  // 4-Queens has 2 solutions
  var q4: i32 = SolveQueens(4);
  // 8-Queens has 92 solutions
  var q8: i32 = SolveQueens(8);

  if (q4 == 2 and q8 == 92) {
    return 0;
  }
  return 1;
}
