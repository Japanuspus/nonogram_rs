use passion_rs::{Puzzle, solve};

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let puzzle: Puzzle = serde_json::from_str(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
    println!("Puzzle: {:?}", puzzle);
    let res = solve(puzzle);
    println!("Solution: {:?}", res);
}
