use std::io::stdin;

use multilogic::kdoku;

fn main() {

    let constraints: Vec<kdoku::Constraint> = stdin()
        .lines()
        .map(|l| l.unwrap())
        .filter(|l| l.trim() != "")
        .map(|l| kdoku::parse::constraint(&l).unwrap().1)
        .collect();

    let Ok(solution) = kdoku::BaseGrid::new().solve(&constraints[..]) else {
        eprintln!("Grid is not solvable");
        return;
    };

    println!("{}", solution);
}