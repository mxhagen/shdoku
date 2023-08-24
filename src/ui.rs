use crate::state::*;

use std::cmp::Ordering::*;
use std::io;

use crossterm::{
    cursor::{MoveDown, MoveLeft, MoveRight, MoveTo, MoveToColumn, MoveUp, SetCursorStyle},
    execute, queue,
    style::{Color, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType::All, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

pub struct Screen<T>
where
    T: io::Write,
{
    pub presel_color_pair: (Color, Color),
    pub markup_color_background: Color,
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

        let presel_color_pair = (Color::Black, Color::Cyan);
        let markup_color_background = Color::Cyan;

        let mut screen = Screen {
            presel_color_pair,
            markup_color_background,
            ostream,
            width,
            height,
        };

        enable_raw_mode().expect("[-]: Error: ui::init: Failed to enable raw mode.");
        queue!(screen.ostream, Clear(All), EnterAlternateScreen)
            .expect("[-]: Error: ui::init: Failed to enter alternate screen.");
        screen
    }

    pub fn deinit(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(self.ostream, Clear(All), LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn update_dimensions(&mut self) -> io::Result<()> {
        let old_dimensions = (self.width, self.height);

        let (width, height) = size()?;
        self.width = width as usize;
        self.height = height as usize;

        if height < 14 || width < 54 {
            self.clear()?;
            self.deinit()?;
            eprintln!("[!]: Error: ui::update_dimensions: Terminal size too small to display UI.");
            return Err(io::Error::from(io::ErrorKind::Other));
        }

        if old_dimensions != (self.width, self.height) {
            self.clear()?;
            self.draw_board()?;
        }

        Ok(())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        queue!(self.ostream, Clear(All))?;
        queue!(self.ostream, MoveToColumn(0))
    }

    pub fn draw(&mut self, state: &State) -> io::Result<()> {
        self.update_dimensions()?;

        self.draw_numbers(state)?;
        self.draw_scoreboard(state)?;
        self.draw_cursor(state)?;

        self.ostream.flush()
    }

    pub fn draw_board(&mut self) -> io::Result<()> {
        self.init_cursor_offset()?;
        queue!(self.ostream, SetForegroundColor(Color::Reset))?;
        queue!(self.ostream, SetBackgroundColor(Color::Reset))?;

        let board_template = board_template();

        for line in board_template {
            write!(self.ostream, "{}", line)?;
            self.move_cursor_by(-41, 1)?;
        }
        Ok(())
    }

    fn draw_numbers(&mut self, state: &State) -> io::Result<()> {
        self.init_cursor_offset()?;
        queue!(self.ostream, SetForegroundColor(Color::Reset))?;
        queue!(self.ostream, SetBackgroundColor(Color::Reset))?;
        self.move_cursor_by(1, 0)?;

        for row in 0..9 {
            match row {
                3 | 6 => self.move_cursor_by(0, 2),
                _ => self.move_cursor_by(0, 1),
            }?;

            for col in 0..9 {
                match col {
                    3 | 6 => self.move_cursor_by(4, 0),
                    _ => self.move_cursor_by(1, 0),
                }?;

                let chr = match state.board[row][col] {
                    x if x == state.preselection => {
                        queue!(
                            self.ostream,
                            SetForegroundColor(self.presel_color_pair.0),
                            SetBackgroundColor(self.presel_color_pair.1)
                        )
                        .unwrap_or(());
                        (x + b'0') as char
                    }
                    0 => {
                        match state.markups[row][col][state.preselection as usize - 1] {
                            true => {
                                queue!(
                                    self.ostream,
                                    SetBackgroundColor(self.markup_color_background)
                                )?;
                            }
                            false => {
                                queue!(
                                    self.ostream,
                                    SetForegroundColor(Color::Reset),
                                    SetBackgroundColor(Color::Reset)
                                )?;
                            }
                        }
                        ' '
                    }
                    x => {
                        queue!(self.ostream, SetForegroundColor(Color::Reset)).unwrap_or(());
                        queue!(self.ostream, SetBackgroundColor(Color::Reset)).unwrap_or(());
                        (x + b'0') as char
                    }
                };
                write!(self.ostream, "{}", chr)?;
            }

            self.move_cursor_by(-24, 0)?;
        }
        Ok(())
    }

    fn draw_scoreboard(&mut self, state: &State) -> io::Result<()> {
        self.init_cursor_offset()?;
        queue!(self.ostream, SetForegroundColor(Color::Reset))?;
        queue!(self.ostream, SetBackgroundColor(Color::Reset))?;

        self.move_cursor_by(34, 1)?;
        write!(self.ostream, "{}", state.get_difficulty_string())?;

        self.move_cursor_by(-5, 1)?;
        queue!(self.ostream, SetForegroundColor(self.presel_color_pair.1))?;
        write!(self.ostream, "{}", state.get_completion_string())?;

        self.move_cursor_by(-2, 2)?;
        let (mins_string, secs_string) = state.get_timer_strings();
        write!(self.ostream, "{}", mins_string)?;
        self.move_cursor_by(1, 0)?;
        write!(self.ostream, "{}", secs_string)?;
        queue!(self.ostream, SetForegroundColor(Color::Reset))?;

        self.move_cursor_by(-6, 2)?;
        for _ in 0..3 {
            write!(self.ostream, " ")?;
            self.move_cursor_by(-1, 1)?;
        }

        self.move_cursor_by(0, -3)?;

        let selected_mode_idx = match state.mode {
            Mode::Edit => 0,
            Mode::Markup => 1,
            Mode::Go => 2,
        };

        queue!(self.ostream, SetForegroundColor(self.presel_color_pair.1))?;
        for i in 0..3 {
            if i == selected_mode_idx {
                write!(self.ostream, ">")?;
            }
            self.move_cursor_by(0, 1)?;
        }

        self.move_cursor_by(2, 1)?;
        write!(self.ostream, "{}", (state.preselection + b'0') as char)?;

        self.move_cursor_by(-3, 1)?;
        write!(self.ostream, "{}", state.get_preselection_completion_char())?;

        queue!(self.ostream, SetForegroundColor(Color::Reset))?;
        Ok(())
    }

    fn draw_cursor(&mut self, state: &State) -> io::Result<()> {
        let (row, col) = (state.cur_row, state.cur_col);
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

    fn move_cursor_by(&mut self, x: isize, y: isize) -> io::Result<()> {
        match x.cmp(&0) {
            Equal => Ok(()),
            Less => queue!(self.ostream, MoveLeft(-x as u16)),
            Greater => queue!(self.ostream, MoveRight(x as u16)),
        }?;
        match y.cmp(&0) {
            Equal => Ok(()),
            Less => queue!(self.ostream, MoveUp(-y as u16)),
            Greater => queue!(self.ostream, MoveDown(y as u16)),
        }?;
        Ok(())
    }

    fn init_cursor_offset(&mut self) -> io::Result<()> {
        let lft_pad = (self.width / 2 - 14) as u16;
        let top_pad = (self.height / 2 - 6) as u16;
        queue!(self.ostream, MoveTo(lft_pad, top_pad))
    }
}

fn board_template() -> [String; 13] {
    [
        String::from("┌────────┬────────┬────────┐    ┌───────┐"),
        String::from("│        │        │        │    │       │"),
        String::from("│        │        │        │    │   /81 │"),
        String::from("│        │        │        │    ├───────┤"),
        String::from("├────────┼────────┼────────┤    │   :   │"),
        String::from("│        │        │        │    ├───────┤"),
        String::from("│        │        │        │    │  Edit │"),
        String::from("│        │        │        │    │  Mark │"),
        String::from("├────────┼────────┼────────┤    │  Go   │"),
        String::from("│        │        │        │    ├───────┤"),
        String::from("│        │        │        │    │  [ ]  │"),
        String::from("│        │        │        │    │   / 9 │"),
        String::from("└────────┴────────┴────────┘    └───────┘"),
    ]
}

pub trait UiCrash {
    fn or_crash(&self);
}

impl<T> UiCrash for io::Result<T> {
    fn or_crash(&self) {
        if self.is_err() {
            std::process::exit(1);
        }
    }
}
