use num_traits::{float::FloatConst, zero};

use crate::{Scalar, Vec2};

pub struct Object {
    pub(crate) mass: Scalar,
    pub(crate) radius: Scalar,
    pub(crate) position: Vec2,
    pub(crate) last_position: Vec2,
    pub(crate) velocity: Vec2,
    pub(crate) acceleration: Vec2,
}

impl Object {
    pub fn new(radius: Scalar, position: Vec2) -> Self {
        let mass = Scalar::PI() * radius * radius;
        Self {
            mass,
            radius,
            position,
            last_position: position,
            velocity: zero(),
            acceleration: zero(),
        }
    }

    pub fn mass(&self) -> Scalar {
        self.mass
    }

    pub fn radius(&self) -> Scalar {
        self.radius
    }

    pub fn set_radius(&mut self, radius: Scalar) {
        self.radius = radius;
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    pub fn set_velocity(&mut self, velocity: Vec2, dt: Scalar) {
        self.last_position = self.position - velocity * dt;
    }

    pub fn add_velocity(&mut self, velocity: Vec2, dt: Scalar) {
        self.last_position -= velocity * dt;
    }

    pub fn acceleration(&self) -> Vec2 {
        self.acceleration
    }

    pub fn set_acceleration(&mut self, acceleration: Vec2) {
        self.acceleration = acceleration;
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "mass: {:.2?}, radius: {:.2}", self.mass, self.radius)?;
        writeln!(f, "pos: {:.2?}", self.position)?;
        writeln!(
            f,
            "vel: {:.2?} {:6.2?}",
            self.velocity,
            self.velocity.magnitude()
        )?;
        writeln!(
            f,
            "acc: {:.2?} {:6.2?}",
            self.acceleration,
            self.acceleration.magnitude()
        )?;
        Ok(())
    }
}
