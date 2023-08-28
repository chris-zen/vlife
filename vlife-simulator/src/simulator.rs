use indexmap::{map::Iter, IndexMap};
use num_traits::float::FloatConst;
use num_traits::Zero;
use rand::{prelude::ThreadRng, Rng};
use std::{collections::HashMap, fmt::Display, ops::Deref};

use vlife_physics::{Object, ObjectId, Physics, Scalar, Vec2};

use crate::cell::{Cell, MAX_SIZE};

pub type CellId = usize;

pub struct Simulator {
    world_size: Vec2,
    next_cell_id: CellId,
    cells: IndexMap<CellId, Cell>,
    physics: Physics,
    time: Scalar,
    dead_cells: Vec<CellId>,
    object_cell: HashMap<ObjectId, CellId>,
    // reactions: M<NUM_MOLECULES, NUM_MOLECULES>,
}

impl Simulator {
    pub fn new(world_size: Vec2) -> Self {
        Self {
            world_size,
            next_cell_id: 0,
            cells: IndexMap::new(),
            physics: Physics::new(world_size),
            time: 0.0,
            dead_cells: Vec::new(),
            object_cell: HashMap::new(),
            // reactions: Self::init_reactions(),
        }
    }

    // fn init_reactions() -> M<NUM_MOLECULES, NUM_MOLECULES> {
    //     let mut reactions = M::identity();
    //
    //     // Initial proportions
    //     let mut proportion = 1.0;
    //     for row in 0..(NUM_MOLECULES - 1) {
    //         let col = row + 1;
    //         reactions[(row, col)] = proportion;
    //         proportion += 0.0;
    //     }
    //     println!("{:.2}", reactions);
    //     // Transitive proportions
    //     for row in (0..=(NUM_MOLECULES - 3)).rev() {
    //         for col in (row + 2)..NUM_MOLECULES {
    //             reactions[(row, col)] = reactions[(row, row + 1)] * reactions[(row + 1, col)];
    //         }
    //     }
    //     println!("{:.2}", reactions);
    //     let mut i = 1.0;
    //     // Inverse proportions
    //     for row in 1..NUM_MOLECULES {
    //         for col in 0..(row - 1) {
    //             // reactions[(row, col)] = 1.0 / reactions[(col, row)];
    //             reactions[(col, row)] = i;
    //             i += 1.0;
    //         }
    //     }
    //     println!("{:.2}", reactions);
    //     // panic!();
    //     reactions
    // }

    pub fn time(&self) -> Scalar {
        self.time
    }

    pub fn add_testing_cell(&mut self) -> CellId {
        let position = Vec2::new(20.0, 200.0);
        let radius = 10.0;
        let object_id = self.physics.add_object(position, radius);
        let cell_id = self.next_cell_id;
        self.next_cell_id += 1;
        let mut cell = Cell::random(self.time, object_id, radius);
        cell.molecules.set_zero();
        cell.energy = 10000.0;
        cell.movement_speed_limit = 10.0;
        cell.movement_direction = 0.20 * Scalar::PI();
        cell.movement_speed = 10.0;
        self.cells.insert(cell_id, cell);
        self.object_cell.insert(object_id, cell_id);
        cell_id
    }

    pub fn add_random_cell(&mut self) -> CellId {
        let mut rng = rand::thread_rng();

        let radius = rng.gen_range(1.0..=MAX_SIZE);
        let position = self.find_free_position(&mut rng, radius);

        let object_id = self.physics.add_object(position, radius);

        let cell_id = self.next_cell_id;
        self.next_cell_id += 1;
        let cell = Cell::random(self.time, object_id, radius);
        self.cells.insert(cell_id, cell);
        self.object_cell.insert(object_id, cell_id);
        cell_id
    }

    fn find_free_position(&mut self, rng: &mut ThreadRng, radius: Scalar) -> Vec2 {
        loop {
            let x = rng.gen_range(0.0..self.world_size.x);
            let y = rng.gen_range(0.0..self.world_size.y);
            let position = Vec2::new(x, y);
            let no_collision = self
                .get_cell_id_closer_to(x, y)
                .and_then(|cell_id| self.cells.get(&cell_id))
                .and_then(|cell| self.physics.get_object(cell.object_id))
                .map_or(true, |object| {
                    (position - object.position()).norm() > radius + object.radius()
                });

            if no_collision {
                break position;
            }
        }
    }

    pub fn cells(&self) -> Cells {
        Cells(self.cells.iter())
    }

    pub fn get_cell_object(&self, id: CellId) -> Option<&Object> {
        self.cells
            .get(&id)
            .and_then(|cell| self.physics.get_object(cell.object_id))
    }

    pub fn get_cell_view(&self, cell_id: CellId) -> Option<CellView<'_>> {
        self.cells.get(&cell_id).and_then(|cell| {
            self.physics
                .get_object(cell.object_id)
                .map(|object| CellView {
                    cell_id,
                    cell,
                    object,
                })
        })
    }

    pub fn get_cell_id_closer_to(&self, x: Scalar, y: Scalar) -> Option<CellId> {
        struct Selection {
            cell_id: CellId,
            dist: f64,
            hit: bool,
        }

        let pos = Vec2::new(x, y);
        let mut selected = None;
        for (cell_id, cell) in self.cells.iter() {
            let cell_id = *cell_id;
            if let Some(object) = self.physics.get_object(cell.object_id) {
                let dist = (object.position() - pos).norm();
                let hit = dist <= object.radius();
                match selected.as_ref() {
                    None => selected = Some(Selection { cell_id, dist, hit }),
                    Some(selection) => {
                        if dist < selection.dist && (hit || !selection.hit) {
                            selected = Some(Selection { cell_id, dist, hit });
                        }
                    }
                }
            }
        }
        selected.map(|selection| selection.cell_id)
    }

    pub fn update(&mut self, dt: Scalar) {
        self.dead_cells.clear();
        self.physics.update(dt);
        self.handle_contacts(dt);
        self.update_cells(dt);
        self.remove_dead_cells();
    }

    fn handle_contacts(&mut self, dt: Scalar) {
        for (_, cell) in self.cells.iter_mut() {
            cell.contact_count = 0.0;
        }
        for contact in self.physics.contacts() {
            // println!(">>>");
            let cell1 = self
                .object_cell
                .get(&contact.id1)
                .and_then(|cell_id| self.cells.get(cell_id).map(|cell| (cell_id, cell)));
            let cell2 = self
                .object_cell
                .get(&contact.id2)
                .and_then(|cell_id| self.cells.get(cell_id).map(|cell| (cell_id, cell)));
            let energy_deltas = cell1.zip(cell2).map(|((id1, cell1), (id2, cell2))| {
                let delta1 = cell1.energy_absorption_from(cell2, dt);
                // println!("delta1: {:.4}", delta1);
                let delta2 = cell2.energy_absorption_from(cell1, dt);
                // println!("delta2: {:.4}", delta2);
                ((id1, delta1), (id2, delta2))
            });
            if let Some(((id1, delta1), (id2, delta2))) = energy_deltas {
                if let Some(cell1) = self.cells.get_mut(id1) {
                    // println!("{:.4} {:.4} {:.4}", cell1.energy, delta1 - delta2, cell1.energy + delta1 - delta2);
                    cell1.energy += delta1 - delta2;
                    cell1.contact_count += 1.0;
                    cell1.contact_normal += contact.normal;
                }
                if let Some(cell2) = self.cells.get_mut(id2) {
                    // println!("{:.4} {:.4} {:.4}", cell2.energy, delta2 - delta1, cell2.energy + delta2 - delta1);
                    cell2.energy += delta2 - delta1;
                    cell2.contact_count += 1.0;
                    cell2.contact_normal -= contact.normal;
                }
            }
        }
    }

    fn update_cells(&mut self, dt: Scalar) {
        for (id, cell) in self.cells.iter_mut() {
            if let Some(object) = self.physics.get_object_mut(cell.object_id) {
                let context = SimulationContext {
                    // reactions: &self.reactions,
                    object,
                };
                cell.update(dt, context);
                object.set_radius(cell.contracted_size());

                let current_velocity = object.velocity();
                object.set_velocity(0.5 * (current_velocity + cell.movement_velocity), dt);
                // object.set_velocity(cell.movement_velocity, dt);
                // object.set_acceleration(cell.movement_velocity / (object.mass() * dt));
            }
            if cell.is_dead() {
                self.dead_cells.push(*id);
            }
        }
    }

    fn remove_dead_cells(&mut self) {
        let num_dead_cells = self.dead_cells.len();
        for cell_id in self.dead_cells.drain(..) {
            if let Some(cell) = self.cells.remove(&cell_id) {
                // TODO transfer any remaining molecules to the world
                let object_id = cell.object_id;
                self.physics.remove_object(object_id);
            }
        }
        for _ in 0..num_dead_cells {
            // self.add_random_cell();
        }
    }
}

pub struct Cells<'a>(Iter<'a, CellId, Cell>);

impl<'a> Iterator for Cells<'a> {
    type Item = (CellId, &'a Cell);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(id, cell)| (*id, cell))
    }
}

pub struct CellView<'a> {
    cell_id: CellId,
    cell: &'a Cell,
    object: &'a Object,
}

impl<'a> CellView<'a> {
    pub fn id(&self) -> CellId {
        self.cell_id
    }

    pub fn position(&self) -> Vec2 {
        self.object.position()
    }

    pub fn radius(&self) -> Scalar {
        self.object.radius()
    }

    pub fn velocity(&self) -> Vec2 {
        self.object.velocity()
    }
}

impl<'a> Deref for CellView<'a> {
    type Target = Cell;

    fn deref(&self) -> &Self::Target {
        self.cell
    }
}

impl<'a> Display for CellView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.object)?;
        write!(f, "{}", self.cell)?;
        Ok(())
    }
}

pub struct SimulationContext<'a> {
    // pub(crate) reactions: &'a M<NUM_MOLECULES, NUM_MOLECULES>,
    pub(crate) object: &'a Object,
}
