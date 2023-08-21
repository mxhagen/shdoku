// TODO: remove when done
#![cfg_attr(debug_assertions, allow(unused))]

use crate::generator::Difficulty::*;
use crate::rand::{seq::SliceRandom, thread_rng};
use crate::sudoku::{Board, New};

/// categories of difficulty, indicating how many
/// empty spaces will be on a sudoku board.
#[allow(unused)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Extreme,
    Custom(usize),
}

impl Difficulty {
    /// the number of cells to be deleted from a filled
    /// sudoku board of the given `Difficulty`
    pub fn removal_count(&self) -> usize {
        match self {
            Easy => 35,
            Medium => 45,
            Hard => 52,
            Extreme => 62,
            Custom(x) => *x,
        }
    }
}

/// generate a random, unsolved sudoku board with a given `Difficulty`.
pub fn generate_sudoku(difficulty: Difficulty) -> Board {
    let mut board = Board::new();

    while solve_random(&mut board).is_err() {}

    let removal_count = difficulty.removal_count();

    let mut remove_positions = (0..81).map(|i| (i / 9, i % 9)).collect::<Vec<_>>();
    remove_positions.shuffle(&mut thread_rng());

    for position in remove_positions.into_iter().take(removal_count) {
        let (row, col) = position;
        board[row][col] = 0;
    }

    board
}

/// TODO: currently empty cells needs to be recreated
///       on each recursive call, and are randomized anew
///       each time, which is an unnecessary overhead.
///
/// solves a sudoku, randomizing which empty cell of equal
/// mrv () to choose next.
/// this is used to generate random, fully solvable sudokus.
fn solve_random(board: &mut Board) -> Result<(), ()> {
    let mut empty_cells: Vec<(usize, usize)> = Vec::new();

    let mut rows: Vec<usize> = (0..9).collect::<Vec<_>>();
    let mut cols: Vec<usize> = (0..9).collect::<Vec<_>>();
    rows.shuffle(&mut thread_rng());
    cols.shuffle(&mut thread_rng());

    for &r in &rows {
        for &c in &cols {
            if board[r][c] == 0 {
                empty_cells.push((r, c));
            }
        }
    }

    if empty_cells.is_empty() {
        return Ok(());
    }

    // choose an empty cell with minimum remaining values heuristic
    empty_cells.sort_by_key(|&(r, c)| {
        let mut possibilities = 0;
        for x in 1..=9 {
            if valid_move(board, r, c, x) {
                possibilities += 1;
            }
        }
        possibilities
    });
    let (r, c) = empty_cells[0];

    let mut values: Vec<u8> = (1..=9).filter(|&x| valid_move(board, r, c, x)).collect();

    // sort possible values by least constraining value heuristic
    values.sort_by_key(|&x| {
        let mut constraints = 0;
        for &(row, col) in &empty_cells {
            if valid_move(board, row, col, x) {
                constraints += 1;
            }
        }
        constraints
    });

    for x in values {
        board[r][c] = x;
        match solve_random(board) {
            Err(_) => board[r][c] = 0, // reset value after backtrack
            _ => return Ok(()),        // done
        }
    }

    Err(()) // no valid solution for cell, backtrack
}

/// check if placing value `x` in the cell located at `row`, `col`
/// is a valid move on the given `board`.
fn valid_move(board: &Board, row: usize, col: usize, x: u8) -> bool {
    for i in 0..9 {
        if board[row][i] == x
            || board[i][col] == x
            || board[row / 3 * 3 + i / 3][col / 3 * 3 + i % 3] == x
        {
            return false;
        }
    }
    true
}
