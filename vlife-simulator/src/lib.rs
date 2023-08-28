pub mod cell;
mod neurons;
mod simulator;

use nalgebra::{Const, MatrixView, SMatrix, SVector};

use vlife_physics::Scalar;

pub use simulator::{CellId, Cells, Simulator};

pub type V<const R: usize> = SVector<Scalar, R>;
pub type M<const R: usize, const C: usize> = SMatrix<Scalar, R, C>;

pub type VView<'a, const R: usize, const S: usize> =
    MatrixView<'a, Scalar, Const<R>, Const<1>, Const<1>, Const<S>>;
