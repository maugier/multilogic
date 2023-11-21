use std::{str::FromStr, fmt::{Display, Write}};

use thiserror::Error;
use varisat::{Solver, ExtendFormula, Var};

use crate::util::{matrix::{Matrix, ShapeError}, solve::DnfFormula};

pub struct Problem(pub Matrix<Option<bool>>);

pub struct Solution(pub Matrix<bool>);

impl Problem {
    pub fn solve(&self) -> Option<Solution> {

        let size = self.0.shape().0;
        let k = size / 2;

        let mut solver = Solver::new();
        let vars = solver.new_var_iter(self.0.len()).collect();

        let grid = Matrix::new(vars, self.0.shape())
            .expect("inconsistent len and shape");

        // For columns and rows, have at least a 1 and a 0 for all three consecutive cells
        for (x,y) in grid.indices() {
            if x >= 2 {
                not_uniform(&mut solver, &[grid[x-2][y], grid[x-1][y], grid[x][y]]);
            }
            if y >= 2 {
                not_uniform(&mut solver, &[grid[x][y-2], grid[x][y-1], grid[x][y]]);
            }
        }

        // For rows and columns, have exactly 5 cells set
        for x in 0..size {
            solver.add_popcount(&grid[x], k);
        }
        for y in 0..size {
            let column: Vec<_> = (0..size).map(|x| grid[x][y]).collect();
            solver.add_popcount(&column, k);
        }

        // Problem constraints
        self.0.zip_with(&grid, |(p,c)| {
            if let Some(p) = p {
                solver.add_clause(&[c.lit(*p)]);
            }
        }).expect("inconsistent shape");

        solver.solve().expect("solver failure");
        let m = solver.model()?;

        let solution = grid.map(|v| m.contains(&v.positive()));
        Some(Solution(solution))

    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Empty grid")]
    EmptyGrid,
    #[error("Invalid char {0}")]
    InvalidChar(char),
    #[error("Building matrix: {0}")]
    Build(#[from] ShapeError)
}


impl FromStr for Problem {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cells = vec![];
        let mut h = 0;
        for line in s.lines() {
            for c in line.chars() {
                cells.push(match c {
                    '1' => Some(true),
                    '0' => Some(false),
                    '.'|' '|'-' => None,
                    other => return Err(ParseError::InvalidChar(other))
                })
            }
            h += 1;
        }
        let w = cells.len() / h;

        Ok(Problem(Matrix::new(cells, (h,w))?))
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.0.lines() {
            for cell in line {
                f.write_char(if *cell { '1' } else { '0' })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn not_uniform(solver: &mut Solver, vars: &[Var]) {
    solver.add_clause(&vars.iter().copied().map(Var::positive).collect::<Vec<_>>());
    solver.add_clause(&vars.iter().copied().map(Var::negative).collect::<Vec<_>>());
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn sample() {
        let p = "\
.0...00..1
..00.1..0.
...0......
1.1......0
1.......0.
..1.1....1
...0......
.0....0.1.
....0....0
0.0.00..0.
";

        let s = "\
1001100101
1100110100
0010011011
1011001010
1101100100
0010110011
0110011001
1001100110
0110011010
0101001101
";

        assert_eq!(
            p.parse::<Problem>()
             .unwrap()
             .solve()
             .unwrap()
             .to_string()
        , s);

    }

}
