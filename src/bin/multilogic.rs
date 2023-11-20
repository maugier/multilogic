use std::io::{stdin, Read};

use multilogic::*;
use clap::Parser;
use anyhow::{anyhow, Result};
use termcolor::BufferWriter;

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

fn main() -> Result<()> {
    use Command::*;
    match Command::parse() {
        KDoku => kdoku(),
        Stars => stars(),
        Voisimage => voisimage(),
        _ => panic!("game not yet implemented")
    }

}

fn kdoku() -> Result<()> {
    use kdoku::*;
    let constraints: Vec<kdoku::Constraint> = stdin()
        .lines()
        .map(|l| l.unwrap())
        .filter(|l| l.trim() != "")
        .map(|l| kdoku::parse::constraint(&l).expect("parse error").1)
        .collect();

    let grid = BaseGrid::new();
    let solution = grid.solve(&constraints[..]).expect("unsolvable");
    println!("{}", solution);
    Ok(())
}

fn stars() -> Result<()> {
    use stars::*;
    let mut buf = vec![];
    stdin().lock().read_to_end(&mut buf)?;
    let buf = std::str::from_utf8(&buf)?;

    let problem: Problem = buf.parse()?;
    if let Some(s) = problem.solve() {
        let w = BufferWriter::stdout(termcolor::ColorChoice::Auto);
        s.color_fmt(w)?;
    } else {
        eprintln!("Unsolvable grid");
    }
    Ok(())

}

fn voisimage() -> Result<()> {
    use voisimage::*;
    let mut buf = vec![];
    stdin().lock().read_to_end(&mut buf)?;
    let buf = std::str::from_utf8(&buf)?;

    let problem: Problem = buf.parse()?;

    let solution = problem.solve()
       .ok_or_else(|| anyhow!("unsolvable grid"))?;

    let w = BufferWriter::stdout(termcolor::ColorChoice::Auto);
    color::Pretty(&problem, &solution).color_fmt(w)?;
    Ok(())

}
