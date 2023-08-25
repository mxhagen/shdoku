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

    pub undo_stack: Vec<UndoStep>,
    pub redo_stack: Vec<UndoStep>,
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
                *self.current_cell() = self.preselection;
                self.delete_colliding_marks(self.preselection, self.cur_row, self.cur_col);
            }
        }
    }

    pub fn delete_current_cell(&mut self) {
        if self.current_cell_is_modifiable() {
            *self.current_cell() = 0;

            self.push_to_undo_stack(UndoStep::Replace(
                self.preselection,
                (self.cur_row as u8, self.cur_col as u8),
            ));
        }
    }

    /// deletes all marks of `num` in it's row, column and block.
    /// used to automatically remove marks when placing a number that
    /// invalidates those marks.
    pub fn delete_colliding_marks(&mut self, num: u8, row: usize, col: usize) {
        let mut deleted = [None; 27];
        let mut pos = 0;

        for i in 0..9 {
            if self.markups[i][col][num as usize - 1] {
                deleted[pos] = Some((i as u8, col as u8));
                pos += 1;
            }
            if self.markups[row][i][num as usize - 1] {
                deleted[pos] = Some((row as u8, i as u8));
                pos += 1;
            }
            if self.markups[row / 3 * 3 + i / 3][col / 3 * 3 + i % 3][num as usize - 1] {
                deleted[pos] = Some((
                    row as u8 / 3 * 3 + i as u8 / 3,
                    col as u8 / 3 * 3 + i as u8 % 3,
                ));
                pos += 1;
            }

            self.markups[i][col][num as usize - 1] = false;
            self.markups[row][i][num as usize - 1] = false;
            self.markups[row / 3 * 3 + i / 3][col / 3 * 3 + i % 3][num as usize - 1] = false;
        }

        self.push_to_undo_stack(UndoStep::Unplace(
            self.preselection,
            (row as u8, col as u8),
            deleted,
        ));
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

        self.markups[self.cur_row][self.cur_col][self.preselection as usize - 1] = false;

        self.push_to_undo_stack(UndoStep::Remark(
            self.preselection,
            (self.cur_row as u8, self.cur_col as u8),
        ));
    }

    pub fn set_current_mark(&mut self) {
        if *self.current_cell() != 0 {
            return;
        }

        self.markups[self.cur_row][self.cur_col][self.preselection as usize - 1] = true;

        self.push_to_undo_stack(UndoStep::Unmark(
            self.preselection,
            (self.cur_row as u8, self.cur_col as u8),
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
        use UndoStep::*;

        if let Some(undo_step) = self.undo_stack.pop() {
            self.redo_stack.push(undo_step);
            match undo_step {
                Unplace(num, (row, col), deleted_marks) => {
                    self.board[row as usize][col as usize] = 0;
                    for (r, c) in deleted_marks.iter().take_while(|x| x.is_some()).flatten() {
                        self.markups[*r as usize][*c as usize][num as usize - 1] = true;
                    }
                }
                Replace(num, (row, col)) => {
                    self.board[row as usize][col as usize] = num;
                }
                Remark(num, (row, col)) => {
                    self.markups[row as usize][col as usize][num as usize - 1] = true;
                }
                Unmark(num, (row, col)) => {
                    self.markups[row as usize][col as usize][num as usize - 1] = false;
                }
            }
        }
    }

    /// redo an action if one was taken and undone.
    pub fn redo(&mut self) {
        use UndoStep::*;

        if let Some(redo_step) = self.redo_stack.pop() {
            self.undo_stack.push(redo_step);
            match redo_step {
                Unplace(num, (row, col), deleted_marks) => {
                    self.board[row as usize][col as usize] = num;
                    for (r, c) in deleted_marks.iter().take_while(|x| x.is_some()).flatten() {
                        self.markups[*r as usize][*c as usize][num as usize - 1] = false;
                    }
                }
                Replace(_, (row, col)) => {
                    self.board[row as usize][col as usize] = 0;
                }
                Remark(num, (row, col)) => {
                    self.markups[row as usize][col as usize][num as usize - 1] = false;
                }
                Unmark(num, (row, col)) => {
                    self.markups[row as usize][col as usize][num as usize - 1] = true;
                }
            }
        }
    }

    /// pushes a move done by the player to the undo stack
    /// this additionally invalidates the redo stack
    pub fn push_to_undo_stack(&mut self, undo_step: UndoStep) {
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UndoStep {
    /// unplace num at (row, col) and re-mark
    /// (row, col) if there were removed marks
    Unplace(u8, (u8, u8), [Option<(u8, u8)>; 27]),

    /// re-place num at (row, col)
    Replace(u8, (u8, u8)),

    /// unmark num at (row, col)
    Unmark(u8, (u8, u8)),

    /// re-mark num at (row, col)
    Remark(u8, (u8, u8)),
}
