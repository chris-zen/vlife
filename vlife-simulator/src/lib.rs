pub mod cell;
mod cell_body;
mod environment;
mod genome;
mod neurons;
mod object_set;
mod physics;
mod real;
mod simulator;

use nalgebra::{Const, MatrixView, SMatrix, SVector, Vector2};

pub use cell_body::CellHandle;
pub use real::{Real, RealConst};
pub use simulator::Simulator;

pub type Vec2 = Vector2<Real>;

pub type V<const R: usize> = SVector<Real, R>;
pub type M<const R: usize, const C: usize> = SMatrix<Real, R, C>;

pub type VView<'a, const R: usize, const S: usize> =
    MatrixView<'a, Real, Const<R>, Const<1>, Const<1>, Const<S>>;
