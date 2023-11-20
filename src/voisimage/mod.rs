use std::{collections::BTreeSet, str::FromStr, fmt::Write};

use crate::util::matrix::{Matrix, ShapeError};

use super::util::{choose, solve::DnfFormula};
use anyhow::{anyhow, bail};
use varisat::{Solver, ExtendFormula};


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Problem(Matrix<Option<u8>>);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Solution(Matrix<bool>);

impl Problem {
    pub fn new(shape: (usize, usize), grid: Vec<Option<u8>>) -> Result<Self, ShapeError> {
        Matrix::new(grid, shape).map(Self)
    }

    pub fn solve(&self) -> Option<Solution> {
        let shape = self.0.shape();

        let mut sat = Solver::new();
        let cells: Vec<_> = sat.new_var_iter(shape.0 * shape.1).collect();
        let grid = Matrix::new(cells, shape).unwrap();
        
        for (x,y) in grid.indices() {

            if let Some(k) = self.0[x][y] {

                let mut clause = vec![];
                let neighs = self.0.neighbors((x,y));

                choose(neighs.len(), k as usize, |bitmap| {
                    let alt = neighs.iter()
                        .zip(bitmap)
                        .map(|(&(x,y), &b)| grid[x][y].lit(b))
                        .collect::<Vec<_>>();
                    clause.push(alt);
                });

                sat.add_dnf(clause);

            }

        }

        sat.solve().expect("solver");


        let good: BTreeSet<_> = sat.model()?
            .into_iter()
            .filter(|l| l.is_positive())
            .map(|l| l.var())
            .collect();

        let grid = grid.map(|var| good.contains(var));

        Some(Solution(grid))
    }
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.0.lines() {
            for cell in line {
                let c = match cell {
                    None => '.',
                    Some(n) => char::from_digit(*n as u32, 10).unwrap(),
                };
                f.write_char(c)?;
            }
            f.write_char('\n')?
        }
        Ok(())
    }
}

impl FromStr for Problem {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut w = None;
        let mut h = 0;
        let mut grid = vec![];

        for line in s.lines() {
            let len = line.len();
            if *w.get_or_insert(len) != len {
                bail!("Unequal line")
            }

            for ch in line.chars() {
                let cell = match ch {
                    '.' => None,
                    '0'..='9' => Some(ch.to_digit(10).unwrap() as u8),
                    other => bail!("Invalid character {:?}", other),
                };
                grid.push(cell);
            }
            h += 1;
        }
        let w = w.ok_or(anyhow!("Empty grid"))?;


        Ok(Self::new((h,w), grid)?)

    }
}

impl std::fmt::Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.0.lines() {
            for &cell in line {
                f.write_char(if cell { '█' } else { '░' })?;
            }
            f.write_char('\n')?
        }
        Ok(())
    }
}

pub mod color {

    use termcolor::{BufferWriter, ColorSpec, Color, WriteColor};
    use std::io::Write;

    use super::*;

    #[derive(Debug)]
    pub struct Pretty<'a>(pub &'a Problem, pub &'a Solution);

    impl Pretty<'_> {

        pub fn color_fmt(&self, w: BufferWriter) -> Result<(), std::io::Error> {
            let mut buf = w.buffer();
            let scheme = |b| {if b { Color::White } else { Color::Black }};

            for (ps, ss) in self.0.0.lines().zip(self.1.0.lines()) {
                for (p, s) in ps.iter().zip(ss) {
                    let mut color = ColorSpec::new();
                    color.set_bold(true)
                         .set_bg(Some(scheme(*s)))
                         .set_fg(Some(scheme(!*s)));


                    buf.set_color(&color)?;
                    write!(buf, "{}", match p { Some(k) => char::from_digit(*k as u32, 10).unwrap(), None => ' ' })?;
                }
                buf.reset()?;
                write!(buf, "\n")?;
            }
            w.print(&buf)
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    fn parse(input: &str) {
        input.parse::<Problem>().unwrap();
    }

    fn solve(input: &str) {
        let p: Problem = input.parse().unwrap();
        eprintln!("{:?}", p);
        p.solve().unwrap();
    }

    fn print(input: &str, solution: &str) {
        let p: Problem = input.parse().unwrap();
        let s = p.solve().unwrap();
        let out = format!("{}", s);
        assert_eq!(out, solution);
    }

    #[test]
    fn all_empty() {
        let p = "\
...
.0.
...
";
        let s = "\
░░░
░░░
░░░
";
        print(p, s);

    }

    #[test]
    fn all_full() {
        let p = "\
...
.9.
...
";
        let s = "\
███
███
███
";
        print(p, s);

    }

    #[test]
    fn corner() {
        let p = "\
4.
..
";
        let s = "\
██
██
";
        print(p, s);

    }

    mod small {

        use crate::util::matrix::mat;

        use super::*;
        const PROBLEM_STRING: &str = "\
243
353
231
";

        fn problem() -> Problem {
            Problem(mat![2,4,3; 3,5,3; 2,3,1].map(|i| Some(*i)))
        }

        fn solution() -> Solution {
            Solution(mat![false, true, true; true, false, true; true, false, false])
        }

        const SOLUTION_STRING: &str = "\
░██
█░█
█░░
";

        #[test]
        fn parse() {
            let p: Problem = PROBLEM_STRING.parse().unwrap();
            assert_eq!(p, problem());
        }

        #[test]
        fn solve() {
            assert_eq!(problem().solve().unwrap(), solution());
        }

        #[test]
        fn print() {
            assert_eq!(&format!("{}", solution()), SOLUTION_STRING);
        }


    }

    mod large {
        use super::Problem;
        const SAMPLE: &str = "\
....4......6....4..20......0.0.
.4....4.0.6...1....2.0..0......
.6..44.2..47..3..7....3......0.
..4..6.....467..7.6...3...0....
...54..4......6...6..5565....0.
.6..6.6..224..435.6.5..5.2.....
..54.3...3....4..2.3...4..0.0.0
.6.423..2..6..3..22....5.......
.63.0.1.4.4.5........35....0...
.5.1....43.3.6....3......6...0.
.433....5445..5.4...12.8.7.53..
..4.5.5..5.675..22.223...7..63.
....6.32..6..32.223.3.55..8....
3.5.3.3.......44....5...4.97...
3..3.4.333.5.4.566666...34...7.
........53456.66.56....5..3..8.
..34...5..4..5.4.4.....85.2.5.5
0...24.6.2.1...32..36...6......
.0..0..4..0.0.0..00.......6875.
.........0.........01.5.3......
";

    #[test]
    fn parse_print() {
        let p: Problem = SAMPLE.parse().unwrap();
        assert_eq!(p.to_string(), SAMPLE)
    }

    const SOLUTION: &str = "";

    #[test] fn parse() { super::parse(SAMPLE) }
    #[test] fn solve() { super::solve(SAMPLE) }
    #[test] fn print() { super::print(SAMPLE, SOLUTION) }

    }

}
