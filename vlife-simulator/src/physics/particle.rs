use crate::real::Real;
use crate::{real::RealConst, Vec2};

#[derive(Debug, Clone)]
pub struct Particle {
    pub(crate) mass: Real,
    pub(crate) radius: Real,
    pub(crate) position: Vec2,
    pub(crate) velocity: Vec2,
    pub(crate) acceleration: Vec2,
}

impl Particle {
    pub fn new(radius: Real, position: Vec2) -> Self {
        let mass = 1.0; //Real::PI * radius * radius;
        Self {
            mass,
            radius,
            position,
            velocity: Vec2::zeros(),
            acceleration: Vec2::zeros(),
        }
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

    pub fn radius(&self) -> Real {
        self.radius
    }

    pub fn set_radius(&mut self, radius: Real) {
        self.radius = radius;
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn acceleration(&self) -> Vec2 {
        self.acceleration
    }
}
