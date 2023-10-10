use nalgebra::UnitComplex;
use rand::Rng;
use std::fmt::Display;

use crate::cell::Cell;
use crate::cell_body::{CellBody, CellHandle, CellView};
use crate::object_set::ObjectSet;
use crate::physics::collisions::collider::polygon::PolygonCollider;
use crate::physics::{Particle, Physics, Spring};
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

        let num_particles = 9;
        let radius = 48.0;
        let surface_strength = 0.9;
        let internal_strength = 0.001;
        let include_springs = true;

        let angle_step = Real::TWO_PI / num_particles as Real;
        let r = UnitComplex::new(-angle_step);

        let center = Vec2::new(
            rng.gen_range((100.0 + radius)..=(self.world_size.x - radius - 100.0)),
            rng.gen_range((100.0 + radius)..=(self.world_size.y - radius - 100.0)),
        );
        // let center = Vec2::new(250.0, 150.0);
        // TODO Check that the space is empty

        let velocity = Vec2::new(rng.gen_range(-2.0..2.0), rng.gen_range(-2.0..2.0))
            * self.physics.step_time()
            / self.physics.num_iterations() as Real;
        // let velocity =
        //     Vec2::new(40.0, 5.0) * self.physics.step_time() / self.physics.num_iterations() as Real;

        let mut v = Vec2::x() * radius;
        let mut particles = Vec::new();
        let mut springs = Vec::new();
        let mut last_particle = None;
        let mut last_position = None::<Vec2>;
        let center_particle = self
            .physics
            .add_particle(Particle::new(center).with_velocity(velocity));
        for _ in 0..num_particles {
            let position = center + v;
            let particle = Particle::new(position).with_velocity(velocity);
            let particle = self.physics.add_particle(particle);
            particles.push(particle);

            if include_springs {
                let spring = self.physics.add_spring(Spring::new(
                    center_particle,
                    particle,
                    radius,
                    internal_strength,
                ));
                springs.push(spring);
            }

            match last_particle.zip(last_position) {
                None => {}
                Some((last_particle, last_position)) => {
                    if include_springs {
                        let length = (position - last_position).magnitude();
                        let spring = self.physics.add_spring(Spring::new(
                            last_particle,
                            particle,
                            length,
                            surface_strength,
                        ));
                        springs.push(spring);
                    }
                }
            }
            last_particle = Some(particle);
            last_position = Some(position);
            v = r.transform_vector(&v);
        }

        if include_springs {
            let length = (last_position.unwrap() - (center + Vec2::x() * radius)).magnitude();
            let spring = self.physics.add_spring(Spring::new(
                last_particle.unwrap(),
                particles[0],
                length,
                surface_strength,
            ));
            springs.push(spring);
        }

        let collider = PolygonCollider::new(particles.clone());
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
