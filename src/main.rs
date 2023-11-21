use std::io::{stdin, Read};

use multilogic::*;
use clap::Parser;
use anyhow::{anyhow, Result};
use termcolor::BufferWriter;

#[derive(Parser)]
#[command()]
enum Command {
    /// Islands connected with a given number of bridges.
    Archipel,

    /// Balanced squares of bits.
    /// 
    /// Bineros are square grids filled with 0 or 1, such that:
    ///  - no three consecutive cells contain the same value
    ///  - every row and every column has half 0 cells and half 1 cells.
    /// 
    /// Input: A grid of N lines of length N containing the characters `0`, `1` or ` `.
    /// 
    /// Output: A valid completion of the same grid, with all the spaces filled; or nothing.
    Binero,

    /// Magic squares of pairs.
    EulerSquare,

    /// 3x3 matrices of unique single-digit numbers with known row and column sums.
    Fubuki,

    /// Irregular grid of single-digit numbers, with known sums.
    Kakuro,

    /// Grid of numbers with arithmetical constraints.
    /// 
    /// KDokus are 6x6 grids where every row and every column is a permutation
    /// of the numbers 1-6. In addition, the grid is divided into irregular areas,
    /// each associated with an operation (`+`,`-`,`*`,`/`) and a result, such that
    /// the sum, product, difference or quotient of the cells in the area is equal to
    /// the given result.
    /// 
    /// `-` and `/` operators can only be applied to areas containing exactly 2 cells.
    /// 
    /// Input: A list of area descriptions, one per line.
    /// 
    /// The contraints are in format: 7+ [(0,0),(0,1),(1,1)]
    /// 
    /// First comes the result, then the operation code, then a list of all
    /// the cell coordinate pairs. Coordinates are in the 0-5 range.
    /// 
    /// Output: A solution to the grid, or nothing.
    KDoku,

    /// Place stars on a colored grid.
    /// 
    /// Stars are N*N grids divided into N colored areas. The goal of the game is to place
    /// N stars on the grid such that there is exactly one star per line, per column, and per
    /// colored area.
    /// 
    /// Input: N lines containing N whitespace-separated integers in the range [0;N[.
    /// The integer indicates the color of the cell.
    /// 
    /// Output: A N*N colored text grid for a valid solution, with star locations indicated by a `*` character;
    /// or nothing.
    Stars,
    Sudoku,
    Tectonic,

    /// Paint a grid, from hints about local neighborhoods.
    /// 
    /// Voisimage is a rectangular grid of binary cells, with some cells containing a number. When the number
    /// is present, it indicates the number of active adjacent cells, present cell included. The numbers are
    /// in the range `0-9` (`0-6` on the edges, `0-4` in the corners)
    /// 
    /// Input: A rectangular grid of digits in the range `0-9` or the character
    /// `.` for an empty cell.
    /// 
    /// Output: The same grid, with the cells colored according to a valid solution.
    Voisimage {
        /// Output using Unicode block drawing characters.
        /// 
        /// The default output mode prints the hints and colors the picture with ansi codes.
        /// This mode hides the hints and makes it possible to copy/paste the picture.
        #[arg(short, long)]
        box_drawing: bool
    }
}

fn main() -> Result<()> {
    use Command::*;
    match Command::parse() {
        Binero => binero(),
        KDoku => kdoku(),
        Stars => stars(),
        Voisimage { box_drawing } => voisimage(box_drawing),
        _ => panic!("game not yet implemented")
    }

}

fn binero() -> Result<()> {
    use binero::*;
    let mut buf = vec![];
    stdin().lock().read_to_end(&mut buf)?;
    let p = std::str::from_utf8(&buf)?;
    if let Some(s) = p.parse::<Problem>()?.solve() {
        println!("{}", s);
    } else {
        eprintln!("No solution");
    }
    Ok(())
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

fn voisimage(unicode: bool) -> Result<()> {
    use voisimage::*;
    let mut buf = vec![];
    stdin().lock().read_to_end(&mut buf)?;
    let buf = std::str::from_utf8(&buf)?;

    let problem: Problem = buf.parse()?;

    let solution = problem.solve()
       .ok_or_else(|| anyhow!("unsolvable grid"))?;

    if unicode {
        println!("{}", solution);
    } else {
        let w = BufferWriter::stdout(termcolor::ColorChoice::Auto);
        color::Pretty(&problem, &solution).color_fmt(w)?;
    }
        Ok(())

}
