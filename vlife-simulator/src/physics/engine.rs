use crate::object_set::{ObjectHandle, ObjectSet};
use crate::physics::collisions::collider::Collider;
use crate::physics::collisions::contact::Contact;
use crate::physics::collisions::resolver::CollisionResolver;
use crate::physics::collisions::CollisionsContext;
use crate::physics::{particle::Particle, spring::Spring};
use crate::{Real, Vec2};

const DEFAULT_STEP_TIME: Real = 1.0 / 60.0;
const DEFAULT_NUM_ITERATIONS: usize = 10;
const DEFAULT_GRAVITY: Real = 9.81;
pub const DEFAULT_DRAG: Real = 0.1;
pub const DEFAULT_RESTITUTION: Real = 0.5;
pub const DEFAULT_FRICTION: Real = 0.6;

pub type ParticleHandle = ObjectHandle<Particle>;
pub type SpringHandle = ObjectHandle<Spring>;
pub type ColliderHandle = ObjectHandle<Collider>;

pub struct Physics {
    time: Real,
    world_size: Vec2,
    step_time: Real,
    num_iterations: usize,
    gravity: Vec2,
    drag: Real,
    restitution: Real,
    friction: Real,
    particles: ObjectSet<Particle>,
    springs: ObjectSet<Spring>,
    colliders: ObjectSet<Collider>,
    contacts: Vec<Contact>,
}

impl Physics {
    pub fn new(world_size: Vec2) -> Self {
        Self {
            time: 0.0,
            world_size,
            step_time: DEFAULT_STEP_TIME,
            num_iterations: DEFAULT_NUM_ITERATIONS,
            gravity: Vec2::new(0.0, DEFAULT_GRAVITY),
            drag: DEFAULT_DRAG,
            restitution: DEFAULT_RESTITUTION,
            friction: DEFAULT_FRICTION,
            particles: ObjectSet::new(),
            springs: ObjectSet::new(),
            colliders: ObjectSet::new(),
            contacts: Vec::new(),
        }
    }

    pub fn time(&self) -> Real {
        self.time
    }

    pub fn world_size(&self) -> Vec2 {
        self.world_size
    }

    pub fn step_time(&self) -> Real {
        self.step_time
    }

    pub fn num_iterations(&self) -> usize {
        self.num_iterations
    }

    pub fn add_particle(&mut self, particle: Particle) -> ParticleHandle {
        self.particles.insert(particle)
    }

    pub fn get_particle(&self, handle: ParticleHandle) -> Option<&Particle> {
        self.particles.get(handle)
    }

    pub fn get_particle_mut(&mut self, handle: ParticleHandle) -> Option<&mut Particle> {
        self.particles.get_mut(handle)
    }

    pub fn remove_particle(&mut self, handle: ParticleHandle) -> Option<Particle> {
        self.particles.remove(handle)
    }

    pub fn add_spring(&mut self, spring: Spring) -> SpringHandle {
        self.springs.insert(spring)
    }

    pub fn get_spring(&self, handle: SpringHandle) -> Option<&Spring> {
        self.springs.get(handle)
    }

    pub fn get_spring_mut(&mut self, handle: SpringHandle) -> Option<&mut Spring> {
        self.springs.get_mut(handle)
    }

    pub fn remove_spring(&mut self, handle: SpringHandle) -> Option<Spring> {
        self.springs.remove(handle)
    }

    pub fn add_collider<C>(&mut self, collider: C) -> ColliderHandle
    where
        C: Into<Collider>,
    {
        self.colliders.insert(collider.into())
    }

    pub fn get_collider(&self, handle: ColliderHandle) -> Option<&Collider> {
        self.colliders.get(handle)
    }

    pub fn remove_collider(&mut self, handle: ColliderHandle) -> Option<Collider> {
        self.colliders.remove(handle)
    }

    pub fn update(&mut self) {
        let sub_step_time = self.step_time / self.num_iterations as Real;
        for _ in 0..self.num_iterations {
            self.update_particles(sub_step_time);
            self.apply_world_boundaries();
            self.apply_springs();
            self.resolve_collisions();
        }
        self.time += self.step_time;
    }

    fn update_particles(&mut self, dt: Real) {
        let half_drag = 0.5 * self.drag;
        for (_, particle) in self.particles.iter_mut() {
            let velocity = particle.position - particle.previous;
            let drag = Self::drag_acceleration(particle.mass, velocity, half_drag, dt);
            particle.acceleration += self.gravity - drag;
            particle.previous = particle.position;
            particle.position += velocity + particle.acceleration * dt * dt;
            particle.acceleration = Vec2::zeros();
        }
    }

    fn drag_acceleration(mass: Real, velocity: Vec2, half_drag: Real, dt: Real) -> Vec2 {
        if velocity == Vec2::zeros() {
            Vec2::zeros()
        } else {
            let velocity_magnitude = velocity.magnitude();
            let drag_force =
                half_drag * velocity_magnitude * velocity_magnitude * velocity.normalize();
            drag_force / mass
        }
    }

    fn apply_world_boundaries(&mut self) {
        for (_, particle) in self.particles.iter_mut() {
            let velocity = particle.position - particle.previous;
            if particle.position.x > self.world_size.x - particle.radius {
                particle.position.x =
                    2.0 * (self.world_size.x - particle.radius) - particle.position.x;
                particle.previous.x = particle.position.x + self.restitution * velocity.x;
                Self::apply_friction_for_boundary_x(particle, self.friction);
            } else if particle.position.x < particle.radius {
                particle.position.x = 2.0 * particle.radius - particle.position.x;
                particle.previous.x = particle.position.x + self.restitution * velocity.x;
                Self::apply_friction_for_boundary_x(particle, self.friction);
            }
            if particle.position.y > self.world_size.y - particle.radius {
                particle.position.y =
                    2.0 * (self.world_size.y - particle.radius) - particle.position.y;
                particle.previous.y = particle.position.y + self.restitution * velocity.y;
                Self::apply_friction_for_boundary_y(particle, self.friction);
            } else if particle.position.y < particle.radius {
                particle.position.y = 2.0 * particle.radius - particle.position.y;
                particle.previous.y = particle.position.y + self.restitution * velocity.y;
                Self::apply_friction_for_boundary_y(particle, self.friction);
            }
        }
    }

    fn apply_friction_for_boundary_x(particle: &mut Particle, friction: Real) {
        let mut new_velocity = particle.position - particle.previous;
        new_velocity.y *= 1.0 - friction;
        particle.previous = particle.position - new_velocity;
    }

    fn apply_friction_for_boundary_y(particle: &mut Particle, friction: Real) {
        let mut new_velocity = particle.position - particle.previous;
        new_velocity.x *= 1.0 - friction;
        particle.previous = particle.position - new_velocity;
    }

    fn apply_springs(&mut self) {
        for (_, spring) in self.springs.iter_mut() {
            if let Some((p1, p2)) = self
                .particles
                .get_pair_mut(spring.particle1, spring.particle2)
            {
                let axis = p2.position - p1.position;
                let distance = axis.magnitude();
                let displacement = distance - spring.length;
                let total_mass = p1.mass + p2.mass;
                let factor = 0.5 * spring.strength * displacement / (distance * total_mass);
                p1.position += 0.5 * axis * factor * p1.mass;
                p2.position -= 0.5 * axis * factor * p2.mass;
                // let force = spring.strength * displacement * (axis / distance);
                // p1.acceleration += force / p1.mass;
                // p2.acceleration -= force / p2.mass;
            }
        }
    }

    fn resolve_collisions(&mut self) {
        self.contacts.clear();
        let resolver = CollisionResolver::new();
        let mut context = CollisionsContext::new(&mut self.particles, &mut self.contacts);

        for (_, collider) in self.colliders.iter_mut() {
            collider.update(&resolver, &mut context);
        }

        let colliders = self.colliders.slice_mut();
        for index1 in 0..colliders.len() {
            for index2 in (index1 + 1)..colliders.len() {
                let (left, right) = colliders.split_at_mut(index1 + 1);
                let (collider1, collider2) = (&mut left[index1], &mut right[index2 - index1 - 1]);
                if collider1.intersects(&collider2) {
                    collider1.resolve_collisions(&collider2, &resolver, &mut context);
                }
            }
        }
    }
}
