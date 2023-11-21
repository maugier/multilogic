pub struct Choose(Option<Vec<bool>>);


impl Choose {
    pub fn new(n: usize, k: usize) -> Self {
        if k > n { return Choose(None) }
        let mut state = vec![false; n];
        for b in &mut state[0..k] { *b = true }
        Choose(Some(state))
    }

}

fn advance(state: &Vec<bool>) -> Option<Vec<bool>> {
    let mut seek = state.iter().cloned().enumerate().rev(); 
    let zero = seek.find(|b| !b.1)?.0;
    let one = seek.find(|b| b.1)?.0;
    let tail = state.len() - zero;

    let mut r = state.clone();
    r[one] = false;
    for b in &mut r[zero+1..] { *b = false }
    for b in &mut r[one+1..][..tail] { *b = true }

    Some(r)
}

impl Iterator for Choose {
    type Item = Vec<bool>;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.0.take()?;
        self.0 = advance(&current);
        Some(current)
    } 
}

#[cfg(test)]
mod test {
    use super::Choose;

    #[test]
    fn choose_5_2() {
        let choices: Vec<Vec<bool>> = Choose::new(5, 2).collect();
        let ptrs: Vec<&[bool]> = choices.iter().map(|v| &**v).collect();

        assert_eq!(&ptrs, &[ &[true, true, false, false, false],
                             &[true, false, true, false, false],
                             &[true, false, false, true, false],
                             &[true, false, false, false, true],
                             &[false, true, true, false, false],
                             &[false, true, false, true, false],
                             &[false, true, false, false, true],
                             &[false, false, true, true, false],
                             &[false, false, true, false, true],
                             &[false, false, false, true, true],
                             ]);
    }
}
