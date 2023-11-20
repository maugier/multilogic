use std::ops::{Index, RangeInclusive};

use varisat::{self, ExtendFormula, CnfFormula, Lit};
use super::{intersect, solve::DnfFormula};

#[derive(Clone,Debug)]
pub struct Var {
    range: RangeInclusive<usize>,
    values: Vec<Lit>, // cannot be empty
}

impl Var {
    pub fn range(&self) -> RangeInclusive<usize> {
        self.range.clone()
    }

    fn values(&self) -> impl Iterator<Item=(usize, &Lit)> + '_{
        self.range().zip(&self.values)
    }
}

impl Index<usize> for Var {
    type Output = Lit;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index - self.range.start()]
    }
}

#[derive(Clone,Debug)]
pub struct Model {
    inner: Vec<Lit>,
}

impl Model {
    pub fn value(&self, var: &Var) -> usize {
        for (val, term) in var.values() {
            if self.inner.contains(term) {
                return val
            }
        }
        panic!("SAT solver returned invalid solution")
    }
}

#[derive(Clone,Debug)]
pub struct Problem {
    inner:  CnfFormula,
}

impl Problem {
    pub fn new() -> Self {
        Self {
            inner: CnfFormula::new()
        }
    }

    pub fn new_var(&mut self, range: RangeInclusive<usize>) -> Var {
        let values: Vec<Lit> = range.clone()
            .map(|_n| self.inner.new_lit())
            .collect();

        // at least one case is true
        self.inner.add_clause(&values);

        // cases are mutually exclusive
        for (i,a) in values.iter().enumerate() {
            for b in &values[i+1..] {
                self.inner.add_clause(&[ a.var().negative(), b.var().negative()]);
            }
        }

        Var { range, values }
    }

    pub fn sum(&mut self, a: &Var, b: &Var) -> Var {
        let ar = a.range();
        let br = b.range();
        let rr = (ar.start() + br.start())..= (ar.end() + br.end());
        let r = self.new_var(rr);

        let mut buffer = vec![];

        for (ax, av) in a.values() {
            for (bx, bv) in b.values() {
                buffer.push([av.clone(), bv.clone(), r[ax+bx].clone()]);
            }
        }

        self.inner.add_dnf(buffer);

        r
    }
    
    pub fn not_equals(&mut self, a: &Var, b: &Var) {
        for i in intersect(a.range(), b.range()) {
            self.inner.add_clause(&[a[i].var().negative(), b[i].var().negative()]);
        }
    }

    pub fn equals(&mut self, var: &Var, val: usize) {
        self.inner.add_clause(&[var[val].clone()])
    }

    pub fn solve(&self) -> Option<Model> {
        let mut solver = varisat::Solver::new();
        solver.add_formula(&self.inner);
        solver.solve().expect("Solver error");
        Some(Model { inner: solver.model()? })
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn one_variable() {
        let mut ip = Problem::new();

        let d = ip.new_var(1..=6);
        ip.equals(&d, 5);

        let m = ip.solve().unwrap();
        assert_eq!(m.value(&d), 5);

    }

    #[test]
    fn two_variables() {
        let mut ip = Problem::new();

        let a = ip.new_var(1..=6);
        let b = ip.new_var(1..=8);
        let s = ip.sum(&a, &b);
        ip.equals(&s, 14);

        let m = ip.solve().unwrap();
        assert_eq!(m.value(&a), 6);
        assert_eq!(m.value(&b), 8);

    }

    #[test]
    fn distinct_numbers() {

        let mut ip = Problem::new();

        let a = ip.new_var(1..=9);
        let b = ip.new_var(1..=9);
        let c = ip.new_var(1..=9);

        ip.not_equals(&a, &b);
        ip.not_equals(&a, &c);
        ip.not_equals(&c, &b);

        let r = ip.sum(&a, &b);
        let r = ip.sum(&r, &c);

        ip.equals(&r, 7);

        let m = ip.solve().unwrap();

        let mut abc = [&a, &b, &c].map(|v| m.value(v));
        abc.sort();

        assert_eq!(abc, [1,2,4]);

    }

}
