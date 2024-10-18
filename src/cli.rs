use clap::{Arg, Command};

pub fn new() -> Command {
    Command::new("shdoku")
        .about("A terminal sudoku game with vim-like controls")
        .arg(
            Arg::new("difficulty")
                .short('d')
                .value_name("easy|mid|hard|expert|0..81")
                .long("difficulty")
                .help("Defined difficulty levels or a custom number of blank spaces"),
        )
}
