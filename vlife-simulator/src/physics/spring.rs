use num_traits::Float;

use crate::object_set::ObjectSet;
use crate::physics::{engine::ParticleHandle, particle::Particle};
use crate::Real;

pub struct Spring {
    pub(crate) particle1: ParticleHandle,
    pub(crate) particle2: ParticleHandle,
    pub(crate) length: Real,
    pub(crate) strength: Real,
}

impl Spring {
    pub fn new(
        particle1: ParticleHandle,
        particle2: ParticleHandle,
        length: Real,
        strength: Real,
    ) -> Self {
        Self {
            particle1,
            particle2,
            length,
            strength,
        }
    }

    pub fn apply_constrain(&self, particles: &mut ObjectSet<Particle>) {
        if let Some((particle1, particle2)) = particles.get_pair_mut(self.particle1, self.particle2)
        {
            let axis = particle2.position - particle1.position;
            let distance = axis.magnitude() + Real::epsilon();
            let norm_dist_strength =
                (distance - self.length) / (distance * (particle1.mass + particle2.mass)) * self.strength;
            particle1.position += axis * norm_dist_strength * particle1.mass;
            particle2.position -= axis * norm_dist_strength * particle2.mass;
        }
    }
}
