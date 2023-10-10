use crate::real::Real;
use crate::{real::RealConst, Vec2};

#[derive(Debug, Clone)]
pub struct Particle {
    pub(crate) mass: Real,
    pub(crate) radius: Real,
    pub(crate) position: Vec2,
    pub(crate) previous: Vec2,
    pub(crate) velocity: Vec2,
    pub(crate) acceleration: Vec2,
}

impl Particle {
    pub fn new(position: Vec2) -> Self {
        Self {
            mass: 1.0,
            radius: 0.0,
            position,
            previous: position,
            velocity: Vec2::zeros(),
            acceleration: Vec2::zeros(),
        }
    }

    pub fn with_mass(mut self, mass: Real) -> Self {
        self.mass = mass;
        self
    }
    pub fn mass(&self) -> Real {
        self.mass
    }

    pub fn inv_mass(&self) -> Real {
        1.0 / self.mass
    }

    pub fn set_mass(&mut self, mass: Real) {
        self.mass = mass;
    }

    pub fn with_radius(mut self, radius: Real) -> Self {
        self.radius = radius;
        self
    }
    pub fn radius(&self) -> Real {
        self.radius
    }

    pub fn set_radius(&mut self, radius: Real) {
        self.radius = radius;
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.previous = self.position - velocity;
        self
    }

    pub fn velocity(&self) -> Vec2 {
        self.position - self.previous
    }

    pub fn acceleration(&self) -> Vec2 {
        self.acceleration
    }
}
