use std::io::stdin;

use multilogic::*;
use clap::Parser;

#[derive(Parser)]
#[command()]
enum Command {
    Archipel,
    Binero,
    Fubuki,
    Kakuro,
    KDoku,
    Stars,
    Sudoku,
    Tectonic,
    Voisimage,
}

fn main() {
    use Command::*;
    match Command::parse() {
        KDoku => kdoku(),
        _ => todo!("Not implemented yet")
    }

}

fn kdoku() {
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