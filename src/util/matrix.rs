//! A large number of logical games, by virtue of existing on paper, use 2-dimensional structures.
//! This module implement packed matrices, without the overhead of supporting multiple dimensions.

use std::ops::{Index, IndexMut};

use thiserror::Error;

/// A Matrix of dynamic size, with elements in `T`.
/// Indexing exposes rows as slices. Individual elements of matrix `m`
/// can be accessed with `m[x][y]`.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Matrix<T> {
    stride: usize,
    vec: Vec<T>,
}

#[derive(PartialEq, Eq, Debug,Error)]
#[error("incorrect shape")]
pub struct ShapeError;

impl <T> Matrix<T> {

    /// Create a new matrix from a vector of elements in row-major order.
    /// Will fail if the length of `vec` doesn't match the requested shape.
    pub fn new(vec: Vec<T>, shape: (usize,usize)) -> Result<Self, ShapeError> {
        if vec.len() != shape.0 * shape.1 { return Err(ShapeError) }
        Ok(Self { vec, stride: shape.1 })
    }

    /// Total number of elements in the matrix. Equal to `shape.0 * shape.1`
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Shape of the matrix
    pub fn shape(&self) -> (usize, usize) {
        (self.vec.len() / self.stride, self.stride)
    }

    /// Creates a new matrix of the same shape by applying a closure to every element
    pub fn map<U,F>(&self, f: F) -> Matrix<U>
        where F: FnMut(&T) -> U
    {
        Matrix { vec: self.vec.iter().map(f).collect(), stride: self.stride }
    }

    /// Iterates over the matrix rows
    pub fn lines(&self) -> impl Iterator<Item=&[T]> + '_ {
        (0..self.vec.len())
            .step_by(self.stride)
            .map(|i| &self.vec[i..][..self.stride])
    }

    /// Iterate over all the coordinate pairs in row-major order
    pub fn indices(&self) -> impl Iterator<Item=(usize,usize)> {
        let (h,w) = self.shape();
        (0..h).flat_map(move |x| (0..w).map(move |y| (x,y)))
    }

    /// Lists all the neighbors of the given location, truncating at the edge.
    pub fn neighbors(&self, pos: (usize, usize)) -> Vec<(usize,usize)> {
        let (x,y) = pos;
        let (h, w) = self.shape();
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

    /// Create a new matrix by applying in parallel an operation to every pair of elements from
    /// two source matrices of identical shape.
    pub fn zip_with<U,V,F>(&self, other: &Matrix<U>, f: F) -> Result<Matrix<V>, ShapeError>
        where F: FnMut((&T, &U)) -> V
    {
        if self.shape() != other.shape() {
            return Err(ShapeError)
        }

        Ok(Matrix {
            stride: self.stride,
            vec: self.vec.iter().zip(&other.vec).map(f).collect(),
        })
    }

}

impl <T> Index<usize> for Matrix<T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index * self.stride..][..self.stride]
    }
    
}

impl <T> IndexMut<usize> for Matrix<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.vec[index * self.stride..][..self.stride]
    }
}

macro_rules! umat {
    [$e:expr; $shape:expr] => {
        $crate::util::matrix::Matrix::new(vec![$e; $shape.0 * $shape.1], $shape).unwrap()
    };
}
pub(crate) use umat;

#[allow(unused_macros)]
macro_rules! mat {
    [ $( $( $e:expr ),+  );* ] => {{
        let mut v = vec![];
        let (mut h, mut t) = (0,0);
        $(
            $(
                v.push($e); t += 1;
            )+
            h += 1;
         )*
        $crate::util::matrix::Matrix::new(v, (h, t/h)).unwrap()
    }}
}

#[allow(unused_imports)]
pub(crate) use mat;

#[cfg(test)]
mod test {
    use crate::util::matrix::ShapeError;

    use super::Matrix;

    #[test]
    fn shape() {
        assert_eq!(ShapeError, Matrix::new(vec![1,2,3], (2,2)).unwrap_err())
    }

    #[test]
    fn lines() {
        let m = Matrix::new(vec![1,2,3,4,5,6], (3,2)).unwrap();
        let m2: Vec<_> = m.lines().collect();
        assert_eq!(&m2, &[&[1,2], &[3,4], &[5,6]]);
    }

    #[test]
    fn indices() {
        let m = Matrix::new(vec![(); 6], (3,2)).unwrap();
        let idxs: Vec<_> = m.indices().collect();
        assert_eq!(vec![(0,0),(0,1),(1,0),(1,1),(2,0),(2,1)], idxs);
    }

    #[test]
    fn access() {
        let m = Matrix::new(vec![1,2,3,4], (2,2)).unwrap();
        assert_eq!(m[0][0], 1);
        assert_eq!(m[0][1], 2);
        assert_eq!(m[1][0], 3);
        assert_eq!(m[1][1], 4);
    }

    #[test]
    fn neighbors() {
        let m = umat![(); (4,4)];
        assert_eq!(m.neighbors((0,0)), vec![(0,0),(0,1),(1,0),(1,1)]);
        assert_eq!(m.neighbors((0,2)), vec![(0,1),(0,2),(0,3), (1,1), (1,2), (1,3)]);
        assert_eq!(m.neighbors((1,2)), vec![(0,1),(0,2),(0,3), (1,1), (1,2), (1,3), (2,1), (2,2), (2,3)]);
        assert_eq!(m.neighbors((3,3)), vec![(2,2),(2,3),(3,2),(3,3)]);
    }
}
