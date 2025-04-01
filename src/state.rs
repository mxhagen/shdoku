use crate::sudoku::*;
use crate::Dir::*;

use std::time;

/// the entire game logic state
pub struct State {
    pub board: Board,
    pub modifiable: [[bool; 9]; 9],
    pub markups: [[[bool; 9]; 9]; 9],

    pub preselection: u8,
    pub cur_row: usize,
    pub cur_col: usize,

    pub mode: Mode,
    pub next_mode: Mode,

    pub difficulty: Difficulty,
    pub start_time: time::Instant,

    pub undo_stack: Vec<DiffStep>,
    pub redo_stack: Vec<DiffStep>,
}

impl State {
    /// returns a new `State` with a randomly generated
    /// sudoku `Board` of the provided `Difficulty`
    pub fn init(difficulty: Difficulty) -> Self {
        let board = generate_sudoku(difficulty);
        let modifiable = State::init_modifiables(board);

        Self {
            board,
            modifiable,
            markups: [[[false; 9]; 9]; 9],

            preselection: 1,
            cur_row: 4,
            cur_col: 4,

            mode: Mode::default(),
            next_mode: Mode::default(),

            difficulty,
            start_time: time::Instant::now(),

            undo_stack: Vec::with_capacity(160),
            redo_stack: Vec::with_capacity(80),
        }
    }

    /// returns a boolean mask of the board, indicating which cells
    /// can be modified by the user and which are part of the puzzle constraints.
    /// makes only cells that are initialized with the value 0 modifiable
    fn init_modifiables(board: Board) -> [[bool; 9]; 9] {
        let mut modifiable = [[false; 9]; 9];
        for (board_row, modifiable_row) in board.iter().zip(modifiable.iter_mut()) {
            for (board_cell, modifiable_flag) in board_row.iter().zip(modifiable_row.iter_mut()) {
                if *board_cell == 0 {
                    *modifiable_flag = true;
                }
            }
        }
        modifiable
    }

    pub fn current_cell_is_modifiable(&self) -> bool {
        self.modifiable[self.cur_row][self.cur_col]
    }

    /// returns a mutable reference to the number
    /// contained in the current cell.
    pub fn current_cell(&mut self) -> &mut u8 {
        &mut self.board[self.cur_row][self.cur_col]
    }

    pub fn move_cursor(&mut self, direction: Dir) {
        self.cur_col = match direction {
            Left => (self.cur_col + 8) % 9,
            Right => (self.cur_col + 1) % 9,
            FarLeft => (self.cur_col + 6) % 9,
            FarRight => (self.cur_col + 3) % 9,
            _ => self.cur_col,
        };

        self.cur_row = match direction {
            Up => (self.cur_row + 8) % 9,
            Down => (self.cur_row + 1) % 9,
            FarUp => (self.cur_row + 6) % 9,
            FarDown => (self.cur_row + 3) % 9,
            _ => self.cur_row,
        };
    }

    pub fn preselect_num(&mut self, num: u8) {
        self.preselection = num;
    }

    pub fn toggle_current_cell(&mut self) {
        if self.current_cell_is_modifiable() {
            if *self.current_cell() == self.preselection {
                self.delete_current_cell();
            } else {
                let old_num = *self.current_cell();
                *self.current_cell() = self.preselection;

                let affected =
                    self.delete_colliding_marks(self.preselection, self.cur_row, self.cur_col);

                self.push_to_undos_invalidating_redos(DiffStep::Edit(
                    old_num,
                    (self.cur_row, self.cur_col),
                    affected,
                    self.preselection,
                ))
            }
        }
    }

    pub fn delete_current_cell(&mut self) {
        if self.current_cell_is_modifiable() {
            let cur = self.current_cell();
            let old = *cur;
            *cur = 0;

            self.push_to_undos_invalidating_redos(DiffStep::Edit(
                old,
                (self.cur_row, self.cur_col),
                vec![],
                0,
            ));
        }
    }

    /// deletes all marks of `num` in it's row, column and block.
    /// used to automatically remove marks when placing a number that
    /// invalidates those marks.
    ///
    /// returns a vector of the affected marks board positions
    pub fn delete_colliding_marks(
        &mut self,
        num: u8,
        row: usize,
        col: usize,
    ) -> Vec<(usize, usize)> {
        let mut deleted = Vec::new();

        for i in 0..9 {
            if self.markups[i][col][num as usize - 1] {
                deleted.push((i, col));
            }
            if self.markups[row][i][num as usize - 1] {
                deleted.push((row, i));
            }
            // TODO: avoid doubly including marks that are both in the same block AND row/col
            if self.markups[row / 3 * 3 + i / 3][col / 3 * 3 + i % 3][num as usize - 1] {
                deleted.push((row / 3 * 3 + i / 3, col / 3 * 3 + i % 3));
            }

            self.markups[i][col][num as usize - 1] = false;
            self.markups[row][i][num as usize - 1] = false;
            self.markups[row / 3 * 3 + i / 3][col / 3 * 3 + i % 3][num as usize - 1] = false;
        }

        deleted
    }

    pub fn toggle_current_mark(&mut self) {
        if self.markups[self.cur_row][self.cur_col][self.preselection as usize - 1] {
            self.delete_current_mark();
        } else {
            self.set_current_mark();
        }
    }

    pub fn delete_current_mark(&mut self) {
        if *self.current_cell() != 0 {
            return;
        }

        let marked = self.markups[self.cur_row][self.cur_col][self.preselection as usize - 1];
        self.markups[self.cur_row][self.cur_col][self.preselection as usize - 1] = false;

        self.push_to_undos_invalidating_redos(DiffStep::Mark(
            self.preselection,
            (self.cur_row, self.cur_col),
            marked,
        ));
    }

    pub fn set_current_mark(&mut self) {
        if *self.current_cell() != 0 {
            return;
        }

        let marked = self.markups[self.cur_row][self.cur_col][self.preselection as usize - 1];
        self.markups[self.cur_row][self.cur_col][self.preselection as usize - 1] = true;

        self.push_to_undos_invalidating_redos(DiffStep::Mark(
            self.preselection,
            (self.cur_row, self.cur_col),
            marked,
        ));
    }

    pub fn move_cursor_to(&mut self, row: usize, col: usize) {
        assert!(
            row < 9 && col < 9,
            "[!] Error: State::go_to: Can't move to row/column out of bounds."
        );
        self.cur_row = row;
        self.cur_col = col;
    }

    pub fn enter_mode(&mut self, mode: Mode) {
        match mode {
            Mode::Go => {
                self.enter_mode_once(mode);
                return;
            }
            Mode::Edit => self.mode = Mode::Edit,
            Mode::Markup => self.mode = Mode::Markup,
        }
        self.next_mode = self.mode;
    }

    /// enter a mode until one action has been taken,
    /// then returning to the previous mode
    pub fn enter_mode_once(&mut self, mode: Mode) {
        if self.mode != mode {
            self.next_mode = self.mode;
            self.mode = mode;
        }
    }

    pub fn enter_next_mode(&mut self) {
        self.mode = self.next_mode;
    }

    fn get_elapsed_time(&self) -> time::Duration {
        time::Instant::now() - self.start_time
    }

    /// returns number of filled cells
    pub fn get_completion_string(&self) -> String {
        let mut count = 0;
        for row in self.board {
            for cell in row {
                if cell != 0 {
                    count += 1;
                }
            }
        }
        let to_char = |x| (x + b'0') as char;
        [to_char(count / 10), to_char(count % 10)].iter().collect()
    }

    /// returns how many of the 9 final occurences of the
    /// preselected number have been found.
    /// returns `'!'` if the user erroneously placed more than 9.
    pub fn get_preselection_completion_char(&self) -> char {
        let count = self
            .board
            .into_iter()
            .flatten()
            .filter(|&cell| cell == self.preselection)
            .count();
        match count {
            10.. => '!',
            _ => (count as u8 + b'0') as char,
        }
    }

    /// returns the difficulty string used on the ingame scoreboard.
    /// if you want the complete difficulty names use `Difficulty::to_string()`
    pub fn get_difficulty_string(&self) -> String {
        match self.difficulty {
            Difficulty::Easy => String::from("Easy "),
            Difficulty::Mid => String::from(" Mid "),
            Difficulty::Hard => String::from("Hard "),
            Difficulty::Expert => String::from("Exprt"),
            Difficulty::Custom(x) => format!("C({:02})", x),
        }
    }

    /// returns (mins, secs) as a pair of strings
    /// both are zero-padded to a width of 2 characters
    pub fn get_timer_strings(&self) -> (String, String) {
        let mut n = self.get_elapsed_time().as_secs();

        let mut mins = String::with_capacity(2);
        let mut secs = String::with_capacity(2);

        let to_char = |x: u64| (x as u8 + b'0') as char;

        mins.push(to_char(n / 600));
        n %= 600;
        mins.push(to_char(n / 60));
        n %= 60;

        secs.push(to_char(n / 10));
        n %= 10;
        secs.push(to_char(n));

        (mins, secs)
    }

    /// returns timer as string in `mm:ss` format
    pub fn get_timer_string(&self) -> String {
        let (mins, secs) = self.get_timer_strings();
        let mut timer = String::with_capacity(5);
        timer.push_str(&mins);
        timer.push(':');
        timer.push_str(&secs);
        timer
    }

    /// undo an action that has been taken
    pub fn undo(&mut self) {
        self.apply_diff(DiffType::Undo);
    }

    /// redo an action if one was taken and undone.
    pub fn redo(&mut self) {
        self.apply_diff(DiffType::Redo);
    }

    /// apply a undo/redo step diff and get back the inverse step
    pub fn apply_diff(&mut self, diff_type: DiffType) {
        use DiffStep::*;
        use DiffType::*;

        let (used_stack, other_stack) = match diff_type {
            Redo => (&mut self.redo_stack, &mut self.undo_stack),
            Undo => (&mut self.undo_stack, &mut self.redo_stack),
        };

        if let Some(diff) = used_stack.pop() {
            match diff {
                Edit(original, (r, c), marks, replacement) => {
                    self.board[r][c] = original;
                    let affected_mark_num = match diff_type {
                        Redo => original,
                        Undo => replacement,
                    };

                    if affected_mark_num != 0 {
                        marks.iter().for_each(|&(r, c)| {
                            self.markups[r][c][affected_mark_num as usize - 1] =
                                !self.markups[r][c][affected_mark_num as usize - 1]
                        });
                    }

                    other_stack.push(DiffStep::Edit(replacement, (r, c), marks, original));
                }
                Mark(num, (r, c), mark) => {
                    let old_mark = self.markups[r][c][num as usize - 1];
                    self.markups[r][c][num as usize - 1] = mark;
                    other_stack.push(DiffStep::Mark(self.preselection, (r, c), old_mark));
                }
            }
        }
    }

    /// pushes a move done by the player to the undo stack
    /// additionally invalidating the redo stack
    pub fn push_to_undos_invalidating_redos(&mut self, undo_step: DiffStep) {
        self.undo_stack.push(undo_step);
        self.redo_stack.clear();
    }
}

pub enum Dir {
    Up,
    Down,
    Left,
    Right,
    FarUp,
    FarDown,
    FarLeft,
    FarRight,
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum Mode {
    #[default]
    Edit,
    Markup,
    Go,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DiffStep {
    /// `Edit(num, position, affected_mark_positions, new_num)`
    Edit(u8, (usize, usize), Vec<(usize, usize)>, u8),

    /// `Mark(num, position, mark)`
    Mark(u8, (usize, usize), bool),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DiffType {
    Undo,
    Redo,
}
