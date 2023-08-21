use std::io;

use crossterm::{
    cursor::{MoveTo, SetCursorStyle},
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
    pub data: Vec<String>,
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

        let mut screen = Screen {
            data: vec![String::with_capacity(width as usize); height as usize],
            ostream,
            width: width as usize,
            height: height as usize,
        };

        enable_raw_mode().expect("[-]: Error: ui::init: Failed to enable raw mode.");
        queue!(screen.ostream, EnterAlternateScreen)
            .expect("[-]: Error: ui::init: Failed to enter alternate screen.");
        screen
    }

    pub fn deinit(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(self.ostream, LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn update_dimensions(&mut self) -> io::Result<()> {
        let (width, height) = size()?;
        self.width = width as usize;
        self.height = height as usize;
        Ok(())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        queue!(self.ostream, Clear(All))
    }

    pub fn draw(&mut self, row: usize, col: usize) -> io::Result<()> {
        self.draw_screen()?;
        self.place_cursor(row, col)?;
        self.ostream.flush()?;
        Ok(())
    }

    pub fn draw_screen(&mut self) -> io::Result<()> {
        self.clear()?;
        let s: String = self.data.join("\r\n");
        write!(self.ostream, "{}", s)?;
        Ok(())
    }

    pub fn render(&mut self, board: [[u8; 9]; 9]) {
        let mut lines = Vec::new();
        lines.push("┌────────┬────────┬────────┐".to_string().chars().collect());

        for (row, row_slice) in board.iter().enumerate() {
            match row {
                3 | 6 => {
                    lines.push("├────────┼────────┼────────┤".to_string().chars().collect());
                }
                _ => {}
            }
            let mut line = String::new();
            for (col, cur_cell) in row_slice.iter().enumerate() {
                match col {
                    0 => line.push_str("│ "),
                    3 | 6 => line.push_str(" │ "),
                    _ => {}
                }
                line.push(match cur_cell {
                    0 => ' ',
                    n => (b'0' + n) as char,
                });
                line.push(' ');
            }
            line.push_str(" │");
            lines.push(line);
        }
        lines.push("└────────┴────────┴────────┘".to_string().chars().collect());

        let pad_hori = self.width / 2 + lines[0].chars().count() / 2;
        let pad_vert = self.height - lines.len();
        let pad_top = pad_vert / 2;
        let pad_bot = pad_vert - pad_top;

        let mut new_data = Vec::new();

        for _ in 0..pad_top {
            new_data.push(String::new());
        }
        for line in lines {
            let padded = format!("{: >width$}", line, width = pad_hori);
            new_data.push(padded);
        }
        for _ in 0..pad_bot {
            new_data.push(String::new());
        }

        self.data = new_data;
    }

    pub fn place_cursor(&mut self, row: usize, col: usize) -> io::Result<()> {
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
