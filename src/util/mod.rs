use std::ops::{RangeInclusive, Range};

pub mod choice;
pub mod integer;
pub mod solve;
pub mod matrix;

pub fn intersect<T: Ord + Copy>(a: RangeInclusive<T>, b: RangeInclusive<T>) -> RangeInclusive<T> {
    let start = a.start().max(b.start());
    let stop = b.end().min(b.end());
    *start ..= *stop
}

pub fn choices(n: usize, k: usize) -> Vec<Vec<bool>> {
    let mut r = vec![];
    choose(n, k, |c| r.push(c.to_owned()));
    r
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

pub fn pair(range: Range<usize>) -> impl Iterator<Item=(usize,usize)> {
    let end = range.end;
    range.flat_map(move |x| (x+1..end).map(move |y| (x,y)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pair_matches_choose() {
        let bound = 10;
        let pair: Vec<usize> = pair(0..bound).map(|(a,b)| (1 << a) + (1 << b)).collect();
        let mut choice: Vec<usize> = Vec::with_capacity(pair.len());
        choose(bound, 2, |ch| choice.push(ch.iter().enumerate().map(|(i,b)| if *b { 1 << i } else { 0 }).sum()));
        assert_eq!(pair, choice);
    }

    #[test]
    fn choice_edge_cases() {
        assert_eq!(choices(5,0), vec![vec![false, false, false, false, false]]);
        assert_eq!(choices(5,5), vec![vec![true, true, true, true, true]]);
    }
}
