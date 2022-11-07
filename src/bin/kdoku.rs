use std::io::stdin;

use multilogic::kdoku;

fn main() {

    let constraints: Vec<kdoku::Constraint> = stdin()
        .lines()
        .map(|l| kdoku::parse::constraint(&l.unwrap()).unwrap().1)
        .collect();

    let Ok(solution) = kdoku::BaseGrid::new().solve(&constraints[..]) else {
        eprintln!("Grid is not solvable");
        return;
    };

    println!("{}", solution);
}