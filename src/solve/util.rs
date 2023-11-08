use std::{ops::RangeInclusive};
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