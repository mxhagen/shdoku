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
            "[!] Error: State::go_to: Can't move to row/column out of bounds."
        );
        self.cur_row = row;
        self.cur_col = col;
    }

    pub fn toggle_current_mark(&mut self) {
        self.markups[self.cur_row][self.cur_col][self.preselection as usize - 1] ^= true;
    }

    pub fn delete_current_mark(&mut self) {
        self.markups[self.cur_row][self.cur_col][self.preselection as usize - 1] = false;
    }

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

    pub fn get_preselection_completion_char(&self) -> char {
        let count = self
            .board
            .into_iter()
            .flatten()
            .filter(|&cell| cell == self.preselection)
            .count();
        (count.min(9) as u8 + b'0') as char
    }

    pub fn get_difficulty_string(&self) -> String {
        match self.difficulty {
            Difficulty::Easy => String::from("Easy "),
            Difficulty::Mid => String::from(" Mid "),
            Difficulty::Hard => String::from("Hard "),
            Difficulty::Expert => String::from("Exprt"),
            Difficulty::Custom(x) => format!("C({:02})", x),
        }
    }

    pub fn get_timer_strings(&self) -> (String, String) {
        let mut n = self.get_time().as_secs();

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

    pub fn get_timer_string(&self) -> String {
        let (mins, secs) = self.get_timer_strings();
        let mut timer = String::with_capacity(5);
        timer.push_str(&mins);
        timer.push(':');
        timer.push_str(&secs);
        timer
    }
}
