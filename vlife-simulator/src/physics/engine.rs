use nalgebra::partial_ge;
use crate::object_set::{ObjectHandle, ObjectSet};
use crate::physics::{particle::Particle, spring::Spring, Collider};
use crate::{Real, Vec2};

const DEFAULT_STEP_TIME: Real = 1.0 / 60.0;
const DEFAULT_NUM_ITERATIONS: usize = 5;
const DEFAULT_GRAVITY: Real = 9.81;
pub const DEFAULT_DRAG: Real = 0.01;
pub const DEFAULT_RESTITUTION: Real = 0.6;
pub const DEFAULT_FRICTION: Real = 0.5;

pub type ParticleHandle = ObjectHandle<Particle>;
pub type SpringHandle = ObjectHandle<Spring>;
pub type SoftBodyHandle = ObjectHandle<Collider>;

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

    pub fn add_collider(&mut self, collider: Collider) -> SoftBodyHandle {
        self.colliders.insert(collider)
    }

    pub fn get_collider(&self, handle: SoftBodyHandle) -> Option<&Collider> {
        self.colliders.get(handle)
    }

    pub fn remove_collider(&mut self, handle: SoftBodyHandle) -> Option<Collider> {
        self.colliders.remove(handle)
    }

    pub fn update(&mut self) {
        let sub_step_time = self.step_time / self.num_iterations as Real;
        for _ in 0..self.num_iterations {
            self.update_particles(sub_step_time);
            self.update_springs();
            self.check_collisions();
            self.apply_world_constrains();
        }
        self.time += self.step_time;
    }

    fn update_particles(&mut self, dt: Real) {
        let half_dt = 0.5 * dt;
        let half_dt_square = half_dt * dt;
        let half_drag = 0.5 * self.drag;
        for (_, particle) in self.particles.iter_mut() {
            let next_pos =
                particle.position + particle.velocity * dt + particle.acceleration * half_dt_square;
            let next_acc =
                self.gravity - Self::drag_acceleration(particle.mass, particle.velocity, half_drag);
            let next_vel = particle.velocity + (particle.acceleration + next_acc) * half_dt;
            particle.position = next_pos;
            particle.velocity = next_vel;
            particle.acceleration = next_acc;
        }
    }

    fn drag_acceleration(mass: Real, velocity: Vec2, half_drag: Real) -> Vec2 {
        if velocity == Vec2::zeros() {
            Vec2::zeros()
        } else {
            let velocity_magnitude = velocity.magnitude();
            let drag_force =
                half_drag * velocity_magnitude * velocity_magnitude * velocity.normalize();
            drag_force / mass
        }
    }

    fn update_springs(&mut self) {
        for (_, spring) in self.springs.iter_mut() {
            // spring.apply_constrain(&mut self.particles);
            if let Some((p1, p2)) = self
                .particles
                .get_pair_mut(spring.particle1, spring.particle2)
            {
                let axis = p2.position - p1.position;
                let distance = axis.magnitude();
                let d3 = 0.5 * (distance - spring.length) / distance;
                p1.position += 0.5 * axis * d3;
                p2.position -= 0.5 * axis * d3;
            }
        }
        // apply particles constraints?
    }

    fn check_collisions(&mut self) {
        for (_, collider) in self.colliders.iter_mut() {
            collider.check_internal_collisions(&mut self.particles);
        }

        let colliders = self.colliders.slice_mut();
        for index1 in 0..colliders.len() {
            for index2 in (index1 + 1)..colliders.len() {
                let (left, right) = colliders.split_at_mut(index1 + 1);
                let (collider1, collider2) = (&mut left[index1], &mut right[index2 - index1 - 1]);
                if collider1.bbox().intersects(&collider2.bbox()) {
                    collider1.check_collisions(&collider2, &mut self.particles);
                }
            }
        }

        let particles = self.particles.slice_mut();
        for i in 0..particles.len() {
            for j in (i + 1)..particles.len() {}
        }
    }

    fn apply_world_constrains(&mut self) {
        for (_, particle) in self.particles.iter_mut() {
            if particle.position.x > self.world_size.x - particle.radius {
                particle.position.x = self.world_size.x - particle.radius;
                particle.velocity.x *= -self.restitution;
                particle.velocity.y -= particle.velocity.y * self.friction;
            } else if particle.position.x < particle.radius {
                particle.position.x = particle.radius;
                particle.velocity.x *= -self.restitution;
                particle.velocity.y -= particle.velocity.y * self.friction;
            }
            if particle.position.y >= self.world_size.y - particle.radius {
                // particle.position.y = self.world_size.y - particle.radius;
                // particle.velocity.y *= -self.restitution;
                // particle.velocity.x += -particle.velocity.x * self.friction;

                let vx = particle.velocity.x;
                let vy = particle.velocity.y * self.friction;
                let vxs = if particle.velocity.x == 0.0 {
                    particle.velocity.x
                } else {
                    particle.velocity.x.signum()
                };

                let friction = if particle.velocity.x.abs() < 0.1 {
                    0.9
                } else {
                    0.01
                };
                particle.position.y = 2.0 * (self.world_size.y - particle.radius) - particle.position.y;
                particle.velocity.y *= -self.restitution;
                particle.velocity.x -= particle.velocity.x * friction;

                if vx.abs() <= vy.abs() {
                    if vx * vxs > 0.0 {
                        // particle.velocity.x -= 0.5 * particle.velocity.x;
                    }
                } else {
                    if vy * vxs > 0.0 {
                        // particle.velocity.x -= vy;
                    }
                }
            } else if particle.position.y < particle.radius {
                particle.position.y = particle.radius;
                particle.velocity.y *= -self.restitution;
                particle.velocity.x += -particle.velocity.x * self.friction;
            }
        }
    }
}

// #[derive(Debug, Clone)]
// pub enum Contact {
//     Objects {
//         id1: ObjectId,
//         id2: ObjectId,
//         normal: Vec2,
//     },
//     Surface {
//         id: ObjectId,
//         normal: Vec2,
//     },
// }
//
// impl Contact {
//     fn objects(id1: ObjectId, id2: ObjectId, normal: Vec2) -> Self {
//         Self::Objects { id1, id2, normal }
//     }
//
//     fn surface(id: ObjectId, normal: Vec2) -> Self {
//         Self::Surface { id, normal }
//     }
// }
