extern crate crossterm;
use crossterm::event::{poll, read, Event::*, KeyCode::*};

extern crate rand;

mod cli;
mod state;
mod sudoku;
mod ui;
use {state::*, sudoku::*, ui::*};

use std::{io, time::Duration};

fn main() {
    let args = cli::new().get_matches();
    let difficulty = match args.get_one::<String>("difficulty") {
        None => Difficulty::Mid,
        Some(d) => d.parse().unwrap(),
    };

    let mut screen = Ui::init(io::stdout());
    let mut state = State::init(difficulty);
    screen.draw_static_elements().or_crash();

    loop {
        if poll(Duration::from_millis(250)).unwrap_or(false) {
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

                    Char('i') => state.enter_mode(Mode::Edit),
                    Char('I') => state.enter_mode_once(Mode::Edit),

                    Char('a') => state.enter_mode(Mode::Markup),
                    Char('A') => state.enter_mode_once(Mode::Markup),

                    Char('g') | Char('G') => state.enter_mode_once(Mode::Go),

                    Char(' ') => match state.mode {
                        Mode::Markup => {
                            state.toggle_current_mark();
                            state.enter_next_mode();
                        }
                        Mode::Edit => {
                            state.toggle_current_cell();
                            state.enter_next_mode();
                            if is_solution(&state.board) {
                                screen.deinit().or_crash();
                                println!("+------------+");
                                println!("| You Win :) |");
                                println!("+------------+");
                                println!("Difficulty: {}", state.difficulty);
                                println!("Final Time: {}", state.get_timer_string());
                                break;
                            }
                        }
                        _ => {}
                    },

                    Char('x') => {
                        if state.current_cell_is_modifiable() {
                            match state.mode {
                                Mode::Go => {}
                                Mode::Edit => state.delete_current_cell(),
                                Mode::Markup => state.delete_current_mark(),
                            }
                        }
                    }

                    Char(num) if ('1'..='9').contains(&num) => match state.mode {
                        Mode::Go => {
                            let idx = (num as u8 - b'1') as usize;
                            state.move_cursor_to(1 + idx / 3 * 3, 1 + idx % 3 * 3);
                            state.enter_next_mode();
                        }
                        _ => state.preselect_num(num as u8 - b'0'),
                    },

                    Char('u') | Char('U') => state.undo(),
                    Char('r') | Char('R') => state.redo(),

                    Char('q') | Char('Q') => {
                        screen.deinit().or_crash();
                        break;
                    }

                    Esc => state.enter_mode(Mode::Edit),
                    _ => {}
                }
            }
        }

        screen.draw(&state).or_crash();
    }
}

mod tests;
