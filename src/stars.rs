use std::{str::FromStr, fmt::{Display, Write}, num::ParseIntError};

use thiserror::Error;
use varisat::{Solver, ExtendFormula};

use crate::util::{matrix::{Matrix, ShapeError}, pair};

pub struct Problem(pub Matrix<usize>);

pub struct Solution<'p> {
    problem: &'p Problem,
    solution: Matrix<bool>,
}


impl Problem {

    pub fn size(&self) -> usize {
        self.0.shape().0
    }

    pub fn colors(&self) -> Vec<Vec<(usize,usize)>> {
        let mut r = vec![ vec![]; self.size() ];

        for (x,y) in self.0.indices() {
            r[self.0[x][y]].push((x,y));
        };

        r
    }

    pub fn solve(&self) -> Option<Solution> {

        let size = self.0.shape().0;
        let mut solver = Solver::new();
        let cells = solver.new_var_iter(self.0.len()).map(|v| v.positive()).collect();
        let grid = Matrix::new(cells, self.0.shape()).unwrap();

        // lines
        for line in grid.lines() {
            // At least one star per line)
            solver.add_clause(line);

            // Never two stars in same line
            for (x,y) in pair(0..size) {
                solver.add_clause(&[!line[x], !line[y]])
            }
        }

        // at least one star per column
        for idx in 0..size {
            let column: Vec<_> = (0..size).map(|x| grid[x][idx]).collect();
            solver.add_clause(&column)
        }

        // Never two stars in the same column
        for (a,b) in pair(0..size) {
            for (&x,&y) in grid[a].iter().zip(grid[b].iter()) {
                solver.add_clause(&[!x, !y])
            }
        }

        // colors
        for cells in self.colors() {
            let cells: Vec<_> = cells.iter().map(|(x, y)| grid[*x][*y]).collect();

            // At least one star per color
            solver.add_clause(&cells);

            // Never two stars in the same color
            for (x,y) in pair(0..cells.len()) {
                solver.add_clause(&[!cells[x], !cells[y]])
            }
        }

        // proximity for diagonals
        for x in 0..size-1 {
            for y in 0..size-1 {
                solver.add_clause(&[!grid[x][y], !grid[x+1][y+1]]);
                solver.add_clause(&[!grid[x][y+1], !grid[x+1][y]]);
            }
        }

        solver.solve().expect("solver failure");

        let m = solver.model()?;
        let solution = grid.map(|cell| m.contains(cell));

        Some(Solution{ problem: self, solution })

    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("shape error")]
    ShapeError(#[from] ShapeError),
    #[error("invalid character")]
    TextError(#[from] ParseIntError),
    #[error("bound error")]
    BoundError,
}

impl FromStr for Problem {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut grid = vec![];
        let mut height = 0;
        for line in s.lines() {
            for cell in line.split_whitespace() {
                grid.push(cell.parse()?)
            }
            height += 1;
        }

        if !grid.iter().all(|c| (0..height).contains(c)) {
            return Err(ParseError::BoundError)
        }

        Ok(Self(Matrix::new(grid, (height, height))?))
    }
}

impl Display for Solution<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.solution.lines() {
            for c in line {
                f.write_char(if *c {'*'} else {'.'})?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

mod color {
    use termcolor::{ColorSpec, BufferWriter, WriteColor, Color};
    use std::io::Write;
    use Color::*;

    pub const COLOR_TABLE: [Color; 8] = [
        Red, Blue, Green, Yellow, Magenta, Cyan, White, Black
    ];

    use super::Solution;
    impl Solution<'_> {

        pub fn color_fmt(&self, w: BufferWriter) -> Result<(), std::io::Error> {
            let mut buf = w.buffer();

            for (ps, ss) in self.problem.0.lines().zip(self.solution.lines()) {
                for (p, s) in ps.iter().zip(ss) {
                    let mut color = ColorSpec::new();
                    color.set_bold(true)
                         .set_fg(Some(Color::White))
                         .set_bg(Some(COLOR_TABLE[*p]));

                    buf.set_color(&color)?;
                    write!(buf, "{}", if *s {'*'} else {'.'})?;
                }
                buf.reset()?;
                writeln!(buf)?;
            }
            w.print(&buf)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sample() {

        let problem = "0 0 0 2 2 3 3 3
        0 0 0 2 3 3 3 1
        0 0 0 2 3 4 3 1
        0 5 5 5 4 4 1 1
        0 0 0 7 4 1 1 7
        7 7 7 7 6 6 1 7
        7 7 7 6 6 7 7 7
        7 7 7 7 7 7 7 7";

        let solution = "\
.......*
...*....
.....*..
..*.....
*.......
......*.
....*...
.*......
";

        assert_eq!(solution, &      
        problem.parse::<Problem>()
               .expect("parse error")
               .solve()
               .expect("could not solve sample")
               .to_string());



    }
}
