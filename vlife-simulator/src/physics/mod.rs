mod engine;
mod particle;
mod spring;
mod collider;
mod bounding_box;

pub use {
    engine::{Physics, ParticleHandle, SpringHandle},
    particle::Particle,
    spring::Spring,
    collider::Collider,
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

