use varisat::{CnfFormula, ExtendFormula, Var, Lit, Solver, solver::SolverError};
use itertools::Itertools;
use thiserror::Error;

macro_rules! ary {
    ($f:expr ; $size:literal) => { [(); $size].map(|_| $f) };
}

#[derive(Clone,Copy,Debug)]
pub struct U6(u8);

#[derive(Clone,Copy,Debug)]
pub struct Solution([[U6; 6]; 6]);

#[derive(Clone, Copy, Debug)]
pub enum Op { Plus, Minus, Times, Div }

#[derive(Debug, Error)]
pub enum LogicalError<'e> {
    #[error("Impossible constraint")]
    ImpossibleConstraint(&'e Constraint),

    #[error("Unsupported constraint")]
    UnsupportedConstraint(&'e Constraint),

    #[error("Unsatisfyable")]
    Unsatisfyable,
    
    #[error("SAT Solver error")]
    SolverError(#[from] SolverError),
}

#[derive(Clone, Debug)]
pub struct BaseGrid {
    formula: CnfFormula,
    vars: [[[Var; 6]; 6]; 6],
}

#[derive(Debug)]
pub struct Constraint {
    pub op: Op,
    pub result: u8,
    pub cells: Vec<(usize, usize)>
}

impl std::fmt::Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.0 {
            for cell in line {
                write!(f, "{}", cell.0)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl BaseGrid {

    pub fn new() -> Self {

        let mut f = CnfFormula::new();

        let vars = ary![ ary![ ary![f.new_var(); 6]; 6]; 6];
    
        for x in 0..6 {
            for y in 0..6 {
    
                let cell = &vars[x][y];
    
                // Each cell has at least one value
                f.add_clause(&cell.map(|v| v.lit(true)));
    
                // Each cell has at most one value
                for v1 in 0..6 {
                    for v2 in 0..6 {
                        if v1 != v2 {
                            f.add_clause(&[ cell[v1].lit(false), cell[v2].lit(false) ])
                        }
                    }
                }
    
            }
        }
    
        //Each row has each number
        for x in 0..6 {
            for v in 0..6 {
                f.add_clause(&vars[x].map(|vs| vs[v].lit(true )))
            }
        }
    
        //Each column has each number
        for y in 0..6 {
            for v in 0..6 {
                f.add_clause(&vars.map(|vs| vs[y][v].lit(true)))
            }
        }
    
        BaseGrid { formula: f, vars }

    }

    // Solve a grid given some logical constraints
    pub fn solve(mut self, constraints: &[Constraint]) -> Result<Solution, LogicalError> {
        for c in constraints { self.add_constraint(c)? };

        let mut solver = Solver::new();
        solver.add_formula(&self.formula);
        solver.solve()?;

        let mut solution = [[U6(0); 6]; 6];

        let model = solver.model().ok_or(LogicalError::Unsatisfyable)?;

        for x in 0..6 {
            for y in 0..6 {
                for v in 0..6 {
                    if model.contains(&self.vars[x][y][v].lit(true)) {
                        solution[x][y] = U6(v as u8 + 1)
                    }
                }
            }
        }

        Ok(Solution(solution))

    }

    fn add_constraint<'c>(&mut self, constraint: &'c Constraint) -> Result<(), LogicalError<'c>> {
        
        let vars: Vec<_> = constraint.cells.iter().map(|(x,y)| self.vars[*x][*y]).collect();

        let terms = match constraint.op {
            Op::Plus => make_associative_constraint(&vars[..], |a,b| a+b, 0, constraint.result as u16),
            Op::Minus => make_binary_constraint(&vars[..], |a,b| a + constraint.result == b || b + constraint.result == a),
            Op::Times => make_associative_constraint(&vars[..], |a,b| a*b, 1, constraint.result as u16),
            Op::Div => make_binary_constraint(&vars[..], |a,b| a * constraint.result == b || b * constraint.result == a),
        }.ok_or(LogicalError::ImpossibleConstraint(constraint))?;

        if terms.is_empty() { return Err(LogicalError::ImpossibleConstraint(constraint))}

        self.add_dnf(terms);

        Ok(())

    }

    /// Add a clause in DNF form, by translating it into helper variables
    fn add_dnf<T>(&mut self, dnf: impl IntoIterator<Item=T>)
        where T: IntoIterator<Item=Lit>
    {

        let mut helpers = vec![];

        for product in dnf {
            let hv = self.formula.new_var();
            helpers.push(hv.lit(true));

            let not_hv = hv.lit(false);

            for term in product {
                self.formula.add_clause(&[not_hv, term])
            }
        }

        self.formula.add_clause(&helpers);

    }

}

/// Generate a DNF constraint for an arithmetic operation
/// Returns None if the number of variables is not exactly 2
fn make_binary_constraint<F>(vars: &[[Var; 6]], op: F) -> Option<Vec<Vec<Lit>>> 
    where F: Fn(u8,u8) -> bool
{

    let [v1, v2] = &vars[..] else { return None };

    let mut terms = vec![];

    for x1 in 0..6 {
        let x1_n = x1 as u8 + 1;
        for x2 in 0..6 {
            let x2_n = x2 as u8 + 1;
            if op(x1_n, x2_n) {
                terms.push(vec![v1[x1].lit(true), v2[x2].lit(true)])
            }
        }
    }

    Some(terms)

}

/// Generate an associative constraint between the given set of vars
/// 
fn make_associative_constraint(vars: &[[Var; 6]], op: fn(u16,u16) -> u16, z: u16, r: u16) -> Option<Vec<Vec<Lit>>> {

    let mut terms = vec![];

    for chosen in vars.iter().map(|_| 0..6).multi_cartesian_product() {
        if chosen.iter().map(|&x| x as u16 + 1).fold(z, op) == r {
            let term = chosen.iter()
                .zip(vars)
                .map(|(&x, &v)| v[x].lit(true))
                .collect();
            terms.push(term);
        }
    }

    if terms.is_empty() { return None }

    Some(terms)

}



#[test]
fn test_sample_grid() {

    let constraints = vec![
        Constraint { op: Op::Plus,  result: 10, cells: vec![ (0,0), (1,0) ] },
        Constraint { op: Op::Plus,  result: 11, cells: vec![ (2,0), (3,0), (4,0), (5,0)]},
        Constraint { op: Op::Plus,  result:  7, cells: vec![ (0,1), (0,2) ]},
        Constraint { op: Op::Plus,  result: 18, cells: vec![ (1,1), (1,2), (2,1), (3,1)]},
        Constraint { op: Op::Plus,  result:  6, cells: vec![ (4,1), (4,2), (4,3) ]},
        Constraint { op: Op::Plus,  result:  7, cells: vec![ (5,1), (5,2)]},
        Constraint { op: Op::Times, result: 30, cells: vec![ (0,3), (1,3), (2,2), (2,3)]},
        Constraint { op: Op::Plus,  result:  8, cells: vec![ (3,2), (3,3)]},
        Constraint { op: Op::Times, result: 24, cells: vec![ (5,3), (5,4)]},
        Constraint { op: Op::Div,   result:  2, cells: vec![ (0,4), (0,5)]},
        Constraint { op: Op::Plus,  result:  2, cells: vec![ (1,4)]},
        Constraint { op: Op::Plus,  result: 13, cells: vec![ (1,5), (2,4), (2,5), (3,5)]},
        Constraint { op: Op::Minus, result:  1, cells: vec![ (3,4), (4,4)]},
        Constraint { op: Op::Minus, result:  3, cells: vec![ (4,5), (5,5)]},

    ];

    let solution = BaseGrid::new().solve(&constraints[..]).unwrap();
    eprintln!("{}", solution);

}