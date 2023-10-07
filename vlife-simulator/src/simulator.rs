use rand::Rng;
use std::fmt::Display;
use nalgebra::UnitComplex;

use crate::cell::Cell;
use crate::cell_body::{CellBody, CellHandle, CellView, CellViewMut};
use crate::object_set::ObjectSet;
use crate::physics::{Collider, Particle, Physics, Spring};
use crate::real::{Real, RealConst};
use crate::Vec2;

pub struct Simulator {
    time: Real,
    world_size: Vec2,
    physics: Physics,
    cells: ObjectSet<CellBody>,
}

impl Simulator {
    pub fn new(world_size: Vec2) -> Self {
        Self {
            time: 0.0,
            world_size,
            physics: Physics::new(world_size),
            cells: ObjectSet::new(),
        }
    }

    pub fn create_random_cell(&mut self) -> CellHandle {
        let mut rng = rand::thread_rng();
        let cell = Cell::random();
        // let radius = cell.radius();

        let num_particles = 16;
        let perimeter = 16.0 * num_particles as Real;
        let angle_step = Real::TWO_PI / num_particles as Real;
        let r = UnitComplex::new(-angle_step);
        let radius = perimeter / Real::TWO_PI;

        // let center = Vec2::new(
        //     rng.gen_range((0.0 + radius)..=(self.world_size.x - radius)),
        //     rng.gen_range((0.0 + radius)..=(self.world_size.y - radius)),
        // );
        let center = Vec2::new(
            550.0,
            200.0,
        );
        // TODO Check that the space is empty

        // let velocity = Vec2::new(
        //     rng.gen_range(-100.0..100.0),
        //     rng.gen_range(-100.0..100.0),
        // );
        let velocity = Vec2::new(
            -100.0,
            100.0,
        );

        let mut v = Vec2::x() * radius;
        let mut particles = Vec::new();
        let mut springs = Vec::new();
        let mut last_particle = None;
        let mut last_position = None::<Vec2>;
        let center_particle = self.physics.add_particle(Particle::new(1.0, center).with_velocity(velocity));
        for _ in 0..num_particles {
            let position = center + v;
            let particle = Particle::new(6.0, position).with_velocity(velocity);
            let particle = self.physics.add_particle(particle);
            particles.push(particle);
            let spring = self.physics.add_spring(Spring::new(center_particle, particle, radius, 0.4));
            springs.push(spring);
            match last_particle.zip(last_position) {
                None => {}
                Some((last_particle, last_position)) => {
                    let length = (position - last_position).magnitude();
                    let spring = self.physics.add_spring(Spring::new(last_particle, particle, length, 0.6));
                    springs.push(spring);
                }
            }
            last_particle = Some(particle);
            last_position = Some(position);
            v = r.transform_vector(&v);
        }
        let length = (last_position.unwrap() - (center + Vec2::x() * radius)).magnitude();
        let spring = self.physics.add_spring(Spring::new(last_particle.unwrap(), particles[0], length, 1.0));
        springs.push(spring);

        let collider = Collider::new(particles.clone());
        self.physics.add_collider(collider);

        let cell_body = CellBody {
            cell,
            center: center_particle,
            particles,
            springs,
        };
        self.cells.insert(cell_body)
    }

    pub fn step_time(&self) -> Real {
        self.physics.step_time()
    }

    pub fn time(&self) -> Real {
        self.time
    }

    pub fn physics(&self) -> &Physics {
        &self.physics
    }

    pub fn cells(&self) -> impl Iterator<Item = CellView<'_>> {
        self.cells
            .iter()
            .map(|(handle, cell_body)| cell_body.view(handle, &self.physics))
    }

    pub fn update(&mut self) {
        self.physics.update();
        self.update_cells();
    }

    fn update_cells(&mut self) {
        let dt = self.step_time();
        for (cell_handle, cell_body) in self.cells.iter_mut() {
            let mut cell_view = cell_body.view_mut(cell_handle, &mut self.physics);
            cell_view.cell().update(dt);
        }
    }
}
