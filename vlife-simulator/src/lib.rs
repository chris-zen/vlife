pub mod cell;
mod cell_rank;
mod genome;
mod neurons;
mod physics;
mod simulator;

use nalgebra::{Const, MatrixView, SMatrix, SVector, Vector2};

pub use simulator::{CellId, Cells, Simulator};

pub type Scalar = f64;
pub type Vec2 = Vector2<Scalar>;

pub type V<const R: usize> = SVector<Scalar, R>;
pub type M<const R: usize, const C: usize> = SMatrix<Scalar, R, C>;

pub type VView<'a, const R: usize, const S: usize> =
    MatrixView<'a, Scalar, Const<R>, Const<1>, Const<1>, Const<S>>;
