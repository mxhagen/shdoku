use crate::state::*;
use std::io;

use crossterm::{
    cursor::{MoveTo, MoveToColumn, SetCursorStyle},
    execute, queue,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType::All, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

pub struct Screen<T>
where
    T: io::Write,
{
    pub data: Vec<Vec<char>>,
    pub ostream: T,
    pub width: usize,
    pub height: usize,
}

impl<T> Screen<T>
where
    T: io::Write,
{
    pub fn init(ostream: T) -> Self {
        let (width, height) = size().unwrap();
        let (width, height) = (width as usize, height as usize);

        let data = RENDER_TEMPLATE.iter().map(|s| s.to_vec()).collect();

        let mut screen = Screen {
            data,
            ostream,
            width,
            height,
        };

        enable_raw_mode().expect("[!]: Error: ui::init: Failed to enable raw mode.");
        queue!(screen.ostream, EnterAlternateScreen)
            .expect("[!]: Error: ui::init: Failed to enter alternate screen.");
        screen
    }

    pub fn deinit(&mut self) {
        disable_raw_mode().unwrap_or(());
        execute!(self.ostream, LeaveAlternateScreen).unwrap_or(());
    }

    pub fn update_dimensions(&mut self) -> io::Result<()> {
        let (width, height) = size()?;
        self.width = width as usize;
        self.height = height as usize;

        if height < 14 || width < 54 {
            self.clear()?;
            self.deinit();
            eprintln!("[!]: Error: ui::update_dimensions: Terminal size too small to display UI.");
            return Err(io::Error::from(io::ErrorKind::Other));
        }

        Ok(())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        queue!(self.ostream, Clear(All))?;
        queue!(self.ostream, MoveToColumn(0))
    }

    pub fn draw(&mut self, row: usize, col: usize) -> io::Result<()> {
        self.draw_screen()?;
        self.draw_cursor(row, col)?;
        self.ostream.flush()?;
        Ok(())
    }

    pub fn draw_screen(&mut self) -> io::Result<()> {
        self.clear()?;
        let lft_pad = self.width / 2 - 14;
        let bot_pad = self.height / 2 - 7;

        let mut display = String::with_capacity((43 + lft_pad) * 13 + bot_pad * 2);

        display.extend(self.data.iter().flat_map(|line| {
            std::iter::repeat(' ')
                .take(lft_pad)
                .chain(line.to_owned())
                .chain(['\r', '\n'])
        }));

        display.push_str(&"\r\n".repeat(bot_pad));

        write!(self.ostream, "{}", display)?;
        Ok(())
    }

    pub fn render(&mut self, state: &State) {
        self.render_board();
        self.render_board_cells(state);
        self.render_scoreboard_elements(state);
    }

    fn render_board(&mut self) {
        self.data = RENDER_TEMPLATE.iter().map(|s| s.to_vec()).collect();
    }

    fn render_board_cells(&mut self, state: &State) {
        for row in 0..9 {
            for col in 0..9 {
                let i = row + 1 + row / 3;
                let j = col + 2 + col + col / 3 * 3;

                self.data[i][j] = match state.board[row][col] {
                    0 => ' ',
                    x => (x + b'0') as char,
                }
            }
        }
    }

    fn render_scoreboard_elements(&mut self, state: &State) {
        // char indeces of scoreboard elements
        // -----
        //
        // difficulty [1][34] - [1][38]
        // completion [2][34] - [1][35]
        //
        // timer [4][34] - [3][38]
        //
        // > edit:   [6][33]
        // > markup: [7][33]
        // > go:     [8][33]
        //
        // preselection: [10][36]
        // preselection completion: [11][34]

        for (i, c) in (34..39).zip(state.get_difficulty_chars()) {
            self.data[1][i] = c;
        }

        for (i, c) in (34..36).zip(state.get_completion_chars()) {
            self.data[2][i] = c;
        }

        for (i, c) in (34..39).zip(state.get_timer_chars()) {
            self.data[4][i] = c;
        }

        for i in 6..9 {
            self.data[i][33] = ' ';
        }

        self.data[match state.mode {
            Mode::Edit => 6,
            Mode::Markup => 7,
            Mode::Go => 8,
        }][33] = '>';

        self.data[10][36] = (state.preselection + b'0') as char;

        self.data[11][34] = state.get_preselection_completion_char();
    }

    pub fn draw_cursor(&mut self, row: usize, col: usize) -> io::Result<()> {
        let (x, y) = (
            (self.width / 2 - 14) as u16,
            (self.height / 2 - 7 + (self.height & 1)) as u16,
        );
        let (x, y) = (x + 2, y + 1);
        let (x, y) = (x + 2 * col as u16, y + row as u16);
        let x = match col {
            0..=2 => x,
            3..=5 => x + 3,
            _ => x + 6,
        };
        let y = match row {
            0..=2 => y,
            3..=5 => y + 1,
            _ => y + 2,
        };
        queue!(self.ostream, MoveTo(x, y), SetCursorStyle::SteadyBlock)
    }
}

const RENDER_TEMPLATE: [[char; 41]; 13] = [
    [
        '┌', '─', '─', '─', '─', '─', '─', '─', '─', '┬', '─', '─', '─', '─', '─', '─', '─', '─',
        '┬', '─', '─', '─', '─', '─', '─', '─', '─', '┐', ' ', ' ', ' ', ' ', '┌', '─', '─', '─',
        '─', '─', '─', '─', '┐',
    ],
    [
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ',
        ' ', ' ', ' ', ' ', '│',
    ],
    [
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ',
        '/', '8', '1', ' ', '│',
    ],
    [
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', '├', '─', '─', '─',
        '─', '─', '─', '─', '┤',
    ],
    [
        '├', '─', '─', '─', '─', '─', '─', '─', '─', '┼', '─', '─', '─', '─', '─', '─', '─', '─',
        '┼', '─', '─', '─', '─', '─', '─', '─', '─', '┤', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ',
        ' ', ' ', ' ', ' ', '│',
    ],
    [
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', '├', '─', '─', '─',
        '─', '─', '─', '─', '┤',
    ],
    [
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', '│', ' ', ' ', 'E',
        'd', 'i', 't', ' ', '│',
    ],
    [
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', '│', ' ', ' ', 'M',
        'a', 'r', 'k', ' ', '│',
    ],
    [
        '├', '─', '─', '─', '─', '─', '─', '─', '─', '┼', '─', '─', '─', '─', '─', '─', '─', '─',
        '┼', '─', '─', '─', '─', '─', '─', '─', '─', '┤', ' ', ' ', ' ', ' ', '│', ' ', ' ', 'G',
        'o', ' ', ' ', ' ', '│',
    ],
    [
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', '├', '─', '─', '─',
        '─', '─', '─', '─', '┤',
    ],
    [
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', '│', ' ', ' ', '[',
        ' ', ']', ' ', ' ', '│',
    ],
    [
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
        '│', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ', ' ', '│', ' ', ' ', ' ',
        '/', ' ', '9', ' ', '│',
    ],
    [
        '└', '─', '─', '─', '─', '─', '─', '─', '─', '┴', '─', '─', '─', '─', '─', '─', '─', '─',
        '┴', '─', '─', '─', '─', '─', '─', '─', '─', '┘', ' ', ' ', ' ', ' ', '└', '─', '─', '─',
        '─', '─', '─', '─', '┘',
    ],
];
