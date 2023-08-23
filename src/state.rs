use crate::sudoku::*;
use crate::Dir::*;

use std::time;

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

pub struct State {
    pub board: Board,
    pub difficulty: Difficulty,
    pub modifiable: [[bool; 9]; 9],
    pub markups: [[[bool; 9]; 9]; 9],
    pub start_time: time::Instant,
    pub mode: Mode,
    pub next_mode: Mode,
    pub preselection: u8,
    pub cur_row: usize,
    pub cur_col: usize,
}

impl State {
    pub fn init(difficulty: Difficulty) -> Self {
        let board = generate_sudoku(difficulty);
        let modifiable = State::get_modifiables(board);

        Self {
            board,
            difficulty,
            modifiable,
            markups: [[[false; 9]; 9]; 9],
            start_time: time::Instant::now(),
            mode: Mode::default(),
            next_mode: Mode::default(),
            preselection: 1,
            cur_row: 4,
            cur_col: 4,
        }
    }

    fn get_time(&self) -> time::Duration {
        time::Instant::now() - self.start_time
    }

    fn get_modifiables(board: Board) -> [[bool; 9]; 9] {
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
            *self.current_cell() = if *self.current_cell() == self.preselection {
                0
            } else {
                self.preselection
            }
        }
    }

    pub fn delete_current_cell(&mut self) {
        if self.current_cell_is_modifiable() {
            *self.current_cell() = 0;
        }
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

    pub fn enter_mode_once(&mut self, mode: Mode) {
        if self.mode != mode {
            self.next_mode = self.mode;
            self.mode = mode;
        }
    }

    pub fn enter_next_mode(&mut self) {
        self.mode = self.next_mode;
    }

    pub fn move_cursor_to(&mut self, row: usize, col: usize) {
        assert!(
            row < 9 && col < 9,
            "[-] Error: State::go_to: Can't move to row/column out of bounds."
        );
        self.cur_row = row;
        self.cur_col = col;
    }

    pub fn toggle_current_mark(&mut self) {
        self.markups[self.cur_row][self.cur_col][self.preselection as usize] ^= true;
    }

    pub fn delete_current_mark(&mut self) {
        self.markups[self.cur_row][self.cur_col][self.preselection as usize] = false;
    }

    pub fn get_completion_chars(&self) -> [char; 2] {
        let mut count = 0;
        for row in self.board {
            for cell in row {
                if cell != 0 {
                    count += 1;
                }
            }
        }
        let to_char = |x| (x + b'0') as char;
        [to_char(count / 10), to_char(count % 10)]
    }

    pub fn get_preselection_completion_char(&self) -> char {
        let count = self
            .board
            .into_iter()
            .flatten()
            .filter(|&cell| cell == self.preselection)
            .count();
        (count.min(9) as u8 + b'0') as char
    }

    pub fn get_difficulty_chars(&self) -> [char; 5] {
        match self.difficulty {
            Difficulty::Easy => [' ', 'E', 'a', 's', 'y'],
            Difficulty::Mid => [' ', 'M', 'i', 'd', ' '],
            Difficulty::Hard => [' ', 'H', 'a', 'r', 'd'],
            Difficulty::Expert => ['E', 'x', 'p', 'r', 't'],
            Difficulty::Custom(x) => [
                'C',
                '(',
                (x as u8 / 10 + b'0') as char,
                (x as u8 % 10 + b'0') as char,
                ')',
            ],
        }
    }

    pub fn get_timer_chars(&self) -> [char; 5] {
        let mut secs = self.get_time().as_secs();
        let mut chars = [':'; 5];
        let to_char = |x: u64| (x as u8 + b'0') as char;
        chars[0] = to_char(secs / 600);
        secs %= 600;
        chars[1] = to_char(secs / 60);
        secs %= 60;
        chars[3] = to_char(secs / 10);
        secs %= 10;
        chars[4] = to_char(secs);
        chars
    }

    pub fn get_timer_string(&self) -> String {
        self.get_timer_chars().iter().collect()
    }
}
