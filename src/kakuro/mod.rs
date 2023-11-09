use std::{ops::Range, fmt::Display};
use crate::solve::{util::MView, integer};

use super::solve::integer::Var;

struct Constraint {
    vertical: bool,
    index: usize,
    range: Range<usize>,
    target: usize,
}

pub struct Problem {
    size: (usize, usize),
    constraints: Vec<Constraint>,
}

pub struct Solution {
    size: (usize, usize),
    cells: Vec<Option<usize>>
}

impl Constraint {
    fn cells(&self) -> impl Iterator<Item=(usize,usize)> + '_{
        self.range.clone()
            .map(|x| {
                if self.vertical { (x, self.index) } else { (self.index, x) }
            })
    }
}

impl Problem {
    pub fn solve(&self) -> Option<Solution> {

        let size = self.size;
        let mut cells: Vec<Option<Var>> = vec![None; size.0 * size.1];
        let mut cellview = MView::new(&mut cells, self.size.1);

        let mut solver = integer::Problem::new();

        for constraint in &self.constraints {

            let mut cells = vec![];
            let mut sum: Option<Var> = None;

            for (x,y) in constraint.cells() {
                cells.push(cellview[x][y].get_or_insert_with(|| solver.new_var(1..=9)).clone());
            }

            for (i, cell) in cells.iter().enumerate() {

                // Include cell in sum constraint
                sum = sum.map(|s| solver.sum(&s,&cell));
                sum.get_or_insert_with(|| cell.clone());

                // Mutually exclusive with all other cells
                for other in &cells[..i] {
                    solver.not_equals(cell, other);
                }
            }

            solver.equals(sum.as_ref().unwrap(), constraint.target);
        }

        let model = solver.solve()?;

        drop(cellview);

        let cells = cells.into_iter()
            .map(|cell| cell.map(|var| model.value(&var) ))
            .collect();

        Some(Solution { cells, size })
        
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in 0..self.size.0 {
            for column in 0..self.size.1 {
                if let Some(v) = &self.cells[line*self.size.1 + column] {
                    write!(f, "{}", v)?;
                } else {
                    write!(f, " ")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tiny_kakuro() {

        // 1 2 4
        // 6 8 9
        
        let k = Problem {
            size: (2, 3),
            constraints: vec![
                Constraint { vertical: true, index: 0, range: 0..2, target: 7 },
                Constraint { vertical: true, index: 1, range: 0..2, target: 10 },
                Constraint { vertical: true, index: 2, range: 0..2, target: 13 },
                Constraint { vertical: false, index: 0, range: 0..3, target: 7 },
                Constraint { vertical: false, index: 1, range: 0..3, target: 23 },
            ],
        };

        let s = k.solve().unwrap();
        assert_eq!(&k.size, &s.size);
        assert_eq!(&s.cells, &[Some(1),Some(2),Some(4),Some(6),Some(8),Some(9)]);


    }
}
