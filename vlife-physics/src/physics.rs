use indexmap::{map::Iter, IndexMap};
use nalgebra::SimdComplexField;
use num_traits::zero;
use std::collections::HashSet;

use crate::{object::Object, Scalar, Vec2};

pub const SUB_STEPS: usize = 1;
pub const RESPONSE_COEF: Scalar = 0.1;

pub type ObjectId = usize;

pub struct Physics {
    time: Scalar,
    world_size: Vec2,
    response_coef: Scalar,
    next_id: ObjectId,
    objects: IndexMap<ObjectId, Object>,
    contacts: HashSet<(ObjectId, ObjectId)>,
}

impl Physics {
    pub fn new(world_size: Vec2) -> Self {
        Self {
            time: zero(),
            world_size,
            response_coef: RESPONSE_COEF,
            next_id: 0,
            objects: IndexMap::new(),
            contacts: HashSet::new(),
        }
    }

    pub fn get_time(&self) -> Scalar {
        self.time
    }

    pub fn add_object(&mut self, position: Vec2, radius: Scalar) -> ObjectId {
        let id = self.next_id;
        self.next_id += 1;
        let object = Object::new(radius, position);
        self.objects.insert(id, object);
        id
    }

    pub fn set_object_velocity(&mut self, id: ObjectId, velocity: Vec2, dt: Scalar) {
        if let Some(object) = self.objects.get_mut(&id) {
            object.set_velocity(velocity, dt)
        }
    }

    pub fn objects(&self) -> Objects {
        Objects(self.objects.iter())
    }

    pub fn get_object(&self, id: ObjectId) -> Option<&Object> {
        self.objects.get(&id)
    }

    pub fn get_object_mut(&mut self, id: ObjectId) -> Option<&mut Object> {
        self.objects.get_mut(&id)
    }

    pub fn remove_object(&mut self, id: ObjectId) {
        self.objects.remove(&id);
    }

    pub fn contacts(&self) -> impl Iterator<Item = (ObjectId, ObjectId)> + '_ {
        self.contacts.iter().cloned()
    }

    pub fn update(&mut self, dt: Scalar) {
        self.time += dt;
        self.contacts.clear();
        let step_dt = dt / SUB_STEPS as Scalar;
        self.begin_update();
        for _ in 0..SUB_STEPS {
            self.check_collisions();
            self.apply_constraints();
            self.update_objects(step_dt);
        }
        self.end_update(dt);
    }

    fn begin_update(&mut self) {}

    fn end_update(&mut self, dt: Scalar) {
        for (_, object) in self.objects.iter_mut() {
            object.velocity = (object.position - object.last_position) / dt;
            object.acceleration = Vec2::zeros();
        }
    }

    fn check_collisions(&mut self) {
        let objects = self.objects.as_mut_slice();
        for i in 0..objects.len() {
            let (visited, remaining) = objects.split_at_mut(i + 1);
            let (id1, o1) = &mut visited.iter_mut().nth(i).unwrap();
            for (id2, o2) in remaining {
                let v = o1.position - o2.position;
                let dist2 = v.norm_squared();
                let min_dist = o1.radius + o2.radius;
                if dist2 < min_dist * min_dist {
                    self.contacts.insert((**id1, *id2));
                    let total_mass = o1.mass + o2.mass;
                    let mass_ratio_1 = o1.mass / total_mass;
                    let mass_ratio_2 = o2.mass / total_mass;
                    let dist = dist2.simd_sqrt();
                    let overlap = min_dist - dist;
                    let n = v / dist;
                    let delta = -0.5 * self.response_coef * overlap;
                    o1.position -= n * (mass_ratio_1 * delta);
                    o2.position += n * (mass_ratio_2 * delta);
                }
            }
        }
    }

    fn apply_constraints(&mut self) {
        for (_, object) in self.objects.iter_mut() {
            Self::apply_world_constraints(object, self.world_size, self.response_coef);
        }
    }

    fn apply_world_constraints(o1: &mut Object, world_size: Vec2, response_coef: Scalar) {
        let response = 0.5 * response_coef;
        if o1.position.x + o1.radius >= world_size.x {
            let overlap = o1.position.x + o1.radius - world_size.x;
            o1.position.x -= response * overlap;
        } else if o1.position.x - o1.radius < 0.0 {
            let overlap = o1.radius - o1.position.x;
            o1.position.x += response * overlap;
        }
        if o1.position.y + o1.radius >= world_size.y {
            let overlap = o1.position.y + o1.radius - world_size.y;
            o1.position.y -= response * overlap;
        } else if o1.position.y - o1.radius < 0.0 {
            let overlap = o1.radius - o1.position.y;
            o1.position.y += response * overlap;
        }
    }

    fn update_objects(&mut self, dt: Scalar) {
        for (_, object) in self.objects.iter_mut() {
            let velocity = object.position - object.last_position;
            object.last_position = object.position;
            object.position += velocity + object.acceleration * (dt * dt);
            object.acceleration = Vec2::zeros();
        }
    }

    fn _vision(&mut self, _dt: Scalar) {
        let objects = self.objects.as_mut_slice();
        for i in 0..objects.len() {
            let (visited, remaining) = objects.split_at_mut(i + 1);
            let o1 = &mut visited[i];
            for (_, o2) in remaining {
                let v = o1.position - o2.position;
                let dist2 = v.norm_squared();
                let min_dist = o1.radius + o2.radius;
                if dist2 < min_dist * min_dist {
                    let dist = dist2.simd_sqrt();
                    let n = v / dist;
                    let mass_ratio_1 = o1.radius / min_dist;
                    let mass_ratio_2 = o2.radius / min_dist;
                    let delta = 0.5 * self.response_coef * (dist - min_dist); // * dt;
                    o1.position -= n * (mass_ratio_2 * delta);
                    o2.position += n * (mass_ratio_1 * delta);
                }
            }
        }
    }
}

pub struct Objects<'a>(Iter<'a, ObjectId, Object>);

impl<'a> Iterator for Objects<'a> {
    type Item = (ObjectId, &'a Object);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(id, object)| (*id, object))
    }
}
