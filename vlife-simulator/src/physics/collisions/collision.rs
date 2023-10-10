use crate::physics::ParticleHandle;
use crate::Real;
use nalgebra::Point2;

#[derive(Debug)]
pub enum Collision {
    PointInPolygon(PointInPolygon),
}

impl From<PointInPolygon> for Collision {
    fn from(collision: PointInPolygon) -> Self {
        Self::PointInPolygon(collision)
    }
}

#[derive(Debug)]
pub struct PointInPolygon {
    pub(crate) particle_handle: ParticleHandle,
    pub(crate) particle_point: Point2<Real>,
    pub(crate) segment_handle1: ParticleHandle,
    pub(crate) segment_handle2: ParticleHandle,
    pub(crate) segment_point1: Point2<Real>,
    pub(crate) segment_point2: Point2<Real>,
    pub(crate) ratio: Real,
    pub(crate) depth: Real,
}
