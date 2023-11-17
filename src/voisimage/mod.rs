use std::{collections::BTreeSet, str::FromStr, fmt::Write};

use crate::solve::util::MView;

use super::solve::util::{choose, DnfFormula};
use anyhow::{anyhow, bail};
use varisat::{self, Var, Solver, ExtendFormula};


#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Problem {
    size: (usize, usize),
    grid: Vec<Option<u8>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Solution {
    size: (usize, usize),
    grid: Vec<bool>
}

impl Problem {
    pub fn new(size: (usize, usize), grid: Vec<Option<u8>>) -> Self {
        Self { size, grid }
    }

    pub fn neighbors(&self, pos: (usize, usize)) -> Vec<(usize,usize)> {
        let (x,y) = pos;
        let (h, w) = self.size;
        let mut neighs = Vec::with_capacity(9);

        let mut row = |x| {
            if y > 0 { neighs.push((x, y-1)) };
            neighs.push((x, y));
            if y+1 < w { neighs.push((x, y+1))};
        };

        if x > 0 { row(x-1) };
        row(x);
        if x+1 < h { row(x+1) };
        neighs
    }

    pub fn solve(&self) -> Option<Solution> {
        let size = self.size;
        let (h,w) = size;

        let mut sat = Solver::new();
        let mut cells: Vec<_> = sat.new_var_iter(h * w).collect();
        let mv = MView::new(&mut cells, w);

        for x in 0..h {
            for y in 0..w {

                if let Some(k) = self.grid[x*w + y] {

                    let mut clause = vec![];
                    let neighs = self.neighbors((x,y));

                    choose(neighs.len(), k as usize, |bitmap| {
                        let alt = neighs.iter()
                            .zip(bitmap)
                            .map(|(&(x,y), &b)| mv[x][y].lit(b))
                            .collect::<Vec<_>>();
                        clause.push(alt);
                    });

                    sat.add_dnf(clause);

                }

            }
        }

        sat.solve().expect("solver");

        let good: BTreeSet<_> = sat.model()?
            .into_iter()
            .filter(|l| l.is_positive())
            .map(|l| l.var())
            .collect();

        let grid = (0..(h*w))
            .map(|p| { good.contains(&cells[p])})
            .collect();

        Some(Solution { size, grid })
    }
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (h,w) = self.size;
        let mut cells = self.grid.iter();
        for _ in 0..h {
            for _ in 0..w {
                let c = match cells.next().expect("invalid problem structure") {
                    None => ' ',
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


        Ok(Self::new((h,w), grid))

    }
}

impl std::fmt::Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (h,w) = self.size;
        let mut cells = self.grid.iter();
        for _ in 0..h {
            for _ in 0..w {
                f.write_char(if *cells.next().expect("invalid solution structure") { '█' } else { '░' })?;
            }
            f.write_char('\n')?
        }
        Ok(())
    }
}

impl Solution {
    fn display_compact(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
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
        p.solve().unwrap();
    }

    fn print(input: &str, solution: &str) {
        let p: Problem = input.parse().unwrap();
        let s = p.solve().unwrap();
        let out = format!("{}", s);
        assert_eq!(out, solution);
    }

    mod small {
        use itertools::Itertools;

        use super::*;
        const PROBLEM_STRING: &str = "\
243
353
231
";

        fn problem() -> Problem {
            Problem {
                size: (3,3),
                grid: vec![2,4,3,3,5,3,2,3,1].into_iter().map(Some).collect_vec(),
            }
        }

        fn solution() -> Solution {
            Solution {
                size: (3,3),
                grid: vec![false, true, true, true, false, true, true, false, false],
            }
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
.........0.........0105.3......
";

    const SOLUTION: &str = "";

    #[test] fn parse() { super::parse(SAMPLE) }
    #[test] fn solve() { super::solve(SAMPLE) }
    #[test] fn print() { super::print(SAMPLE, SOLUTION) }

    }

}
