use nalgebra::{Point2, SimdComplexField};

use crate::object_set::ObjectSet;
use crate::physics::{
    bounding_box::AxisAlignedBoundingBox, engine::DEFAULT_RESTITUTION, Particle, ParticleHandle,
};
use crate::{Real, Vec2};

const DEFAULT_FRICTION: Real = 0.01;

pub struct Collider {
    particles: Vec<ParticleHandle>,
    restitution: Real,
    surface: Vec<Point2<Real>>,
    bbox: AxisAlignedBoundingBox,
}

impl Collider {
    pub fn new(particles: Vec<ParticleHandle>) -> Self {
        Self {
            particles,
            restitution: DEFAULT_RESTITUTION,
            surface: Vec::new(),
            bbox: AxisAlignedBoundingBox::builder().build(),
        }
    }

    pub fn with_response_coefficient(mut self, coefficient: Real) -> Self {
        self.restitution = coefficient;
        self
    }

    pub fn add_particle(&mut self, handle: ParticleHandle) {
        self.particles.push(handle);
    }

    pub(crate) fn check_internal_collisions(&mut self, particles: &mut ObjectSet<Particle>) {
        self.surface.clear();
        let mut bbox = AxisAlignedBoundingBox::builder();
        for particle_handle in self.particles.iter().cloned() {
            if let Some(particle) = particles.get(particle_handle) {
                let position = particle.position.into();
                self.surface.push(position);
                bbox.add_point(position);
            }
        }
        self.bbox = bbox.build();

        for index1 in 0..self.surface.len() {
            let handle1 = self.particles[index1];
            for index2 in (index1 + 1)..self.surface.len() {
                let handle2 = self.particles[index2];
                if let Some((p1, p2)) = particles.get_pair_mut(handle1, handle2) {
                    self.check_particles_collision(p1, p2);
                }
            }
        }
    }

    fn check_particles_collision(&self, p1: &mut Particle, p2: &mut Particle) {
        let elasticity = self.restitution;
        let axis = if p1.position == p2.position {
            Vec2::new(0.001, 0.0)
        } else {
            p2.position - p1.position
        };
        let squared_distance = axis.magnitude_squared();
        let min_distance = p1.radius + p2.radius;
        if squared_distance < min_distance * min_distance {
            let normal = axis / squared_distance.simd_sqrt();
            let vrel = p2.velocity - p1.velocity;
            let masses = (p1.mass * p2.mass) / (p1.mass + p2.mass);
            let impulse = masses * (1.0 + self.restitution) * vrel.dot(&normal);
            p1.velocity += (impulse / p1.mass) * normal;
            p2.velocity -= (impulse / p2.mass) * normal;
        }
    }

    fn check_particles_collision1(&self, p1: &mut Particle, p2: &mut Particle) {
        let axis = if p1.position == p2.position {
            Vec2::new(0.001, 0.0)
        } else {
            p2.position - p1.position
        };
        let squared_distance = axis.magnitude_squared();
        let min_distance = p1.radius + p2.radius;
        if squared_distance < min_distance * min_distance {
            let total_mass = p1.mass + p2.mass;
            let mass_ratio1 = p1.mass / total_mass;
            let mass_ratio2 = p2.mass / total_mass;
            let distance = squared_distance.simd_sqrt();
            let depth = min_distance - distance;
            let normal = axis / distance;
            let delta = 0.5 * self.restitution * depth;
            p1.position -= normal * (mass_ratio1 * delta);
            p2.position += normal * (mass_ratio2 * delta);
        }
    }

    pub(crate) fn check_collisions(
        &mut self,
        other: &Collider,
        particles: &mut ObjectSet<Particle>,
    ) {
    }

    pub fn surface(&self) -> &[Point2<Real>] {
        self.surface.as_slice()
    }

    pub fn bbox(&self) -> AxisAlignedBoundingBox {
        self.bbox
    }
}
