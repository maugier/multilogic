use std::ops::RangeInclusive;

pub mod integer;
pub mod solve;
pub mod matrix;

pub fn intersect<T: Ord + Copy>(a: RangeInclusive<T>, b: RangeInclusive<T>) -> RangeInclusive<T> {
    let start = a.start().max(b.start());
    let stop = b.end().min(b.end());
    *start ..= *stop
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