use crate::sudoku::Board;

/// returns true if the provided sudoku `Board` is in a solved state
pub fn is_solution(sudoku: &Board) -> bool {
    for i in 0..9 {
        let mut row_set = 0u16;
        let mut col_set = 0u16;
        for j in 0..9 {
            if sudoku[i][j] == 0 || sudoku[i][j] > 9 {
                return false;
            }
            let (tmp_row, tmp_col) = (row_set, col_set);
            row_set ^= 1 << sudoku[i][j];
            col_set ^= 1 << sudoku[j][i];
            if row_set < tmp_row || col_set < tmp_col {
                return false;
            }
        }
    }

    for i in 0..9 {
        let mut block_set = 0u16;
        for j in 0..9 {
            let tmp = block_set;
            block_set ^= 1 << sudoku[i / 3 * 3 + i % 3][j / 3 * 3 + j % 3];
            if block_set < tmp {
                return false;
            }
        }
    }

    true
}
