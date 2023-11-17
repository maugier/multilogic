use std::{ops::{RangeInclusive, Index, IndexMut}};
use varisat::{ExtendFormula, Lit};

pub trait DnfFormula: ExtendFormula {

    /// Add a constraint in disjunctive normal form (i.e `(a & b) | (c & d)`)
    /// This works by assigning a helper variable h_i that is implied by the conjunction:
    /// 
    /// h1 => (a & b), h2 => (c & d).
    /// 
    /// We can rewrite these constraints as:
    /// (not(h1) | a&b) & (not(h2) | c&d)
    /// 
    /// And distribute to convert to CNF:
    /// (not(h1) | a) & (not(h1) | b) & ...
    /// 
    /// and rewrite the original disjunction as (h1 | h2).
    /// 
    fn add_dnf<T>(&mut self, dnf: impl IntoIterator<Item=T>)
        where T: IntoIterator<Item=Lit>
    {
            
        let mut helpers = vec![];

        for product in dnf {
            let hv = self.new_var();
            helpers.push(hv.positive());

            let not_hv = hv.negative();

            for term in product {
                self.add_clause(&[not_hv, term])
            }
        }

        self.add_clause(&helpers);

    }   
}

impl<T: ExtendFormula> DnfFormula for T {}

pub fn intersect<T: Ord + Copy>(a: RangeInclusive<T>, b: RangeInclusive<T>) -> RangeInclusive<T> {
    let start = a.start().max(b.start());
    let stop = b.end().min(b.end());
    *start ..= *stop
}

pub struct MView<'a, T> {
    vec: &'a mut [T],
    stride: usize,
}

impl <'a,T> MView<'a, T> {
    pub fn new(vec: &'a mut [T], stride: usize) -> Self {
        // TODO assert stride?
        Self { vec, stride }
    }
}

impl <'a, T> Index<usize> for MView<'a, T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index * self.stride..][..self.stride]
    }
}

impl <'a, T> IndexMut<usize> for MView<'a, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.vec[index * self.stride..][..self.stride]
    }
}


pub fn choose(n: usize, k: usize, mut f: impl FnMut(&[bool])) {
    let mut acc = Vec::with_capacity(n);
    choose_acc(&mut acc, n, k, &mut f);
}

fn choose_acc<F>(acc: &mut Vec<bool>, n: usize, k: usize, f: &mut F)
    where F: FnMut(&[bool])
{
    if n == 0 {
        f(&acc[..]); return
    }

    if k > 0 {
        acc.push(true);
        choose_acc(acc, n-1, k-1, f);
        acc.pop();
    }

    if k < n {
        acc.push(false);
        choose_acc(acc, n-1, k, f);
        acc.pop();
    }
}