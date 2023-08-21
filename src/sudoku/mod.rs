pub mod generator;
pub mod validator;

pub use generator::*;
pub use validator::*;

pub type Board = [[u8; 9]; 9];

trait New {
    fn new() -> Self;
}

impl New for Board {
    fn new() -> Self {
        [[0u8; 9]; 9]
    }
}
