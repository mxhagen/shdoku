use crate::{generator::*, Board};

#[test]
fn generated_sudoku_uniqueness() {
    use std::collections::HashSet;
    use std::time::{Duration, Instant};

    let iterations = 100;
    let mut uniques: HashSet<Board> = HashSet::new();
    let mut total_time = Duration::new(0, 0);

    for i in 0..iterations {
        let inb4 = Instant::now();
        uniques.insert(generate_sudoku(Difficulty::Custom(0)));
        let dt = Instant::now() - inb4;
        total_time += dt;
        println!("iteration {: >5} took {:.5} s", i, dt.as_secs_f64());
    }

    println!(
        "total time taken to generate {} sudoku boards: {}",
        iterations,
        total_time.as_secs_f64()
    );

    assert!(
        uniques.len() == iterations,
        "expected {} generated sudokus to all be unique, but got {} duplicates.",
        iterations,
        iterations - uniques.len()
    );
}


#[test]
fn generated_sudoku_solvability_small() {
    use Difficulty::*;

    let iterations = 20;

    for difficulty in [Easy, Mid, Hard, Expert] {
        for _ in 0..iterations {
            let mut sudoku = generate_sudoku(difficulty);
            solve_random(&mut sudoku).expect("Generated sudoku not solvable");
        }
    }
}

#[test]
#[ignore]
fn generated_sudoku_solvability_large() {
    use Difficulty::*;

    let iterations = 400;

    for difficulty in [Easy, Mid, Hard, Expert] {
        for _ in 0..iterations {
            let mut sudoku = generate_sudoku(difficulty);
            solve_random(&mut sudoku).expect("Generated sudoku not solvable");
        }
    }
}
