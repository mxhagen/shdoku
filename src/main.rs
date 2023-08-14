extern crate crossterm;
use crossterm::{
    cursor::{MoveTo, SetCursorStyle},
    event::{poll, read, Event::*, KeyCode::*},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType::All},
};

use std::{
    io::{self, stdout, Stdout, Write},
    time::Duration,
};

fn main() -> io::Result<()> {
    let (width, height) = size()?;
    let (mut width, mut height) = (width as usize, height as usize);

    let mut screen = vec![String::with_capacity(width); height];

    let modifiable = [
        [false, false, true, true, false, true, true, true, true],
        [false, true, true, false, false, false, true, true, true],
        [true, false, false, true, true, true, true, false, true],
        [false, true, true, true, false, true, true, true, false],
        [false, true, true, false, true, false, true, true, false],
        [false, true, true, true, false, true, true, true, false],
        [true, false, true, true, true, true, false, false, true],
        [true, true, true, false, false, false, true, true, false],
        [true, true, true, true, false, true, true, false, false],
    ];

    let mut board = [
        [5, 3, 0, 0, 7, 0, 0, 0, 0],
        [6, 0, 0, 1, 9, 5, 0, 0, 0],
        [0, 9, 8, 0, 0, 0, 0, 6, 0],
        [8, 0, 0, 0, 6, 0, 0, 0, 3],
        [4, 0, 0, 8, 0, 3, 0, 0, 1],
        [7, 0, 0, 0, 2, 0, 0, 0, 6],
        [0, 6, 0, 0, 0, 0, 2, 8, 0],
        [0, 0, 0, 4, 1, 9, 0, 0, 5],
        [0, 0, 0, 0, 8, 0, 0, 7, 9],
    ];

    let mut cur_row = 0;
    let mut cur_col = 0;

    let mut stdout = stdout();
    enable_raw_mode()?;

    loop {
        let (w, h) = size()?;
        (width, height) = (w as usize, h as usize);

        if poll(Duration::from_millis(250))? {
            if let Ok(Key(k)) = read() {
                match k.code {
                    Char('q') => break,
                    Char('h') => cur_col = (cur_col + 8) % 9,
                    Char('j') => cur_row = (cur_row + 1) % 9,
                    Char('k') => cur_row = (cur_row + 8) % 9,
                    Char('l') => cur_col = (cur_col + 1) % 9,
                    Char('H') => cur_col = (cur_col + 6) % 9,
                    Char('J') => cur_row = (cur_row + 3) % 9,
                    Char('K') => cur_row = (cur_row + 6) % 9,
                    Char('L') => cur_col = (cur_col + 3) % 9,
                    Char('x') => board[cur_row][cur_col] = 0,
                    Char(num) if ('1'..='9').contains(&num) => {
                        if modifiable[cur_row][cur_col] {
                            board[cur_row][cur_col] = num as u8 - b'0'
                        }
                    }
                    _ => {}
                }
            }
        }

        render_to_screen(&mut screen, width, height, board);
        draw_screen(&mut stdout, &screen)?;
        place_cursor(&mut stdout, cur_row, cur_col, width, height)?;
    }

    disable_raw_mode()?;

    Ok(())
}

fn place_cursor(
    stdout: &mut Stdout,
    cur_row: usize,
    cur_col: usize,
    width: usize,
    height: usize,
) -> io::Result<()> {
    let (x, y) = ((width / 2 - 14) as u16, (height / 2 - 7) as u16);
    let (x, y) = (x + 2, y + 1);
    let (x, y) = (x + 2 * cur_col as u16, y + cur_row as u16);
    let x = match cur_col {
        0..=2 => x,
        3..=5 => x + 3,
        _ => x + 6,
    };
    let y = match cur_row {
        0..=2 => y,
        3..=5 => y + 1,
        _ => y + 2,
    };
    execute!(stdout, MoveTo(x, y), SetCursorStyle::SteadyBlock)
}

fn draw_screen(stdout: &mut Stdout, screen: &[String]) -> io::Result<()> {
    queue!(stdout, Clear(All))?;
    let s: String = screen.join("\r\n");
    write!(stdout, "{}", s)?;
    stdout.flush()?;
    Ok(())
}

fn render_to_screen(screen: &mut Vec<String>, cols: usize, rows: usize, board: [[u8; 9]; 9]) {
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
        for (col, cur_field) in row_slice.iter().enumerate() {
            match col {
                0 => {
                    line.push_str("│ ");
                }
                3 | 6 => {
                    line.push_str(" │ ");
                }
                _ => {}
            }
            line.push(match cur_field {
                0 => ' ',
                n => (b'0' + n) as char,
            });
            line.push(' ');
        }
        line.push_str(" │");
        lines.push(line);
    }
    lines.push("└────────┴────────┴────────┘".to_string().chars().collect());

    let pad_hori = cols / 2 + lines[0].chars().count() / 2;
    let pad_vert = rows - lines.len();
    let pad_top = pad_vert / 2;
    let pad_bot = pad_vert - pad_top;

    let mut new_screen = Vec::new();

    for _ in 0..pad_top {
        new_screen.push(String::new());
    }
    for line in lines {
        let padded = format!("{: >width$}", line, width = pad_hori);
        new_screen.push(padded);
    }
    for _ in 0..pad_bot {
        new_screen.push(String::new());
    }

    *screen = new_screen;
}
