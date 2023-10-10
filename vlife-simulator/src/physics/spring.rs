use crate::physics::engine::ParticleHandle;
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
}
