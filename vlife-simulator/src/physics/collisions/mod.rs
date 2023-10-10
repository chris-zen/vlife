use crate::object_set::ObjectSet;
use crate::physics::collisions::contact::Contact;
use crate::physics::Particle;

pub mod collider;
pub mod collision;
pub mod contact;
pub mod resolver;

pub(crate) struct CollisionsContext<'a> {
    pub(crate) particles: &'a mut ObjectSet<Particle>,
    pub(crate) contacts: &'a mut Vec<Contact>,
}

impl<'a> CollisionsContext<'a> {
    pub(crate) fn new(
        particles: &'a mut ObjectSet<Particle>,
        contacts: &'a mut Vec<Contact>,
    ) -> Self {
        Self {
            particles,
            contacts,
        }
    }
}
