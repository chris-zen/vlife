mod object;
mod physics;

use nalgebra::Vector2;

pub type Scalar = f64;
pub type Vec2 = Vector2<Scalar>;

pub use object::Object;
pub use physics::{ObjectId, Objects, Physics};
