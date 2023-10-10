pub mod collisions;
mod engine;
mod geometry;
mod particle;
mod spring;

pub use collisions::collider::polygon::PolygonCollider;
pub use {
    engine::{ParticleHandle, Physics, SpringHandle},
    particle::Particle,
    spring::Spring,
};

trait BuilderExt
where
    Self: Sized,
{
    fn apply<V, F>(self, value: Option<V>, f: F) -> Self
    where
        F: FnOnce(Self, V) -> Self,
    {
        if let Some(value) = value {
            f(self, value)
        } else {
            self
        }
    }
}
