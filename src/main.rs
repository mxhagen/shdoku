extern crate crossterm;
use crossterm::event::{poll, read, Event::*, KeyCode::*};

extern crate rand;

mod state;
use state::*;

mod sudoku;
use sudoku::*;

mod ui;
use ui::*;

use std::{io, time::Duration};

fn main() -> io::Result<()> {
    let mut screen = Screen::init(io::stdout());
    let mut state = State::init(Difficulty::Medium);

    loop {
        if poll(Duration::from_millis(250))? {
            if let Ok(Key(k)) = read() {
                match k.code {
                    Char('h') => state.move_cursor(Dir::Left),
                    Char('j') => state.move_cursor(Dir::Down),
                    Char('k') => state.move_cursor(Dir::Up),
                    Char('l') => state.move_cursor(Dir::Right),
                    Char('H') => state.move_cursor(Dir::FarLeft),
                    Char('J') => state.move_cursor(Dir::FarDown),
                    Char('K') => state.move_cursor(Dir::FarUp),
                    Char('L') => state.move_cursor(Dir::FarRight),
                    Char('x') => {
                        if state.current_cell_modifiable() {
                            *state.current_cell() = 0;
                        }
                    }
                    Char(num) if ('1'..='9').contains(&num) => {
                        if state.current_cell_modifiable() {
                            *state.current_cell() = num as u8 - b'0';
                        }
                        if is_solution(&state.board) {
                            screen.deinit()?;
                            println!("you win");
                            break;
                        }
                    }
                    Char('q') => {
                        screen.deinit()?;
                        break;
                    }
                    _ => {}
                }
            }
        }

        screen.update_dimensions()?;
        screen.render(state.board);
        screen.draw(state.cur_row, state.cur_col)?;
    }

    Ok(())
}

mod tests;
