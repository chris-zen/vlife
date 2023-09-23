use indexmap::{map::Iter, IndexMap};
use num_traits::{float::FloatConst, Zero};
use rand::{prelude::ThreadRng, Rng};
use std::{collections::HashMap, fmt::Display, ops::Deref};

use crate::cell::{Cell, MAX_SIZE};
use crate::cell_rank::CellRank;
use crate::genome::Genome;
use crate::physics::{Contact, Object, ObjectId, Physics};
use crate::{Scalar, Vec2};

pub const RANK_SIZE: usize = 100;

pub type CellId = usize;

pub struct Simulator {
    world_size: Vec2,
    next_cell_id: CellId,
    cells: IndexMap<CellId, Cell>,
    physics: Physics,
    time: Scalar,
    dead_cells: Vec<CellId>,
    born_cells: Vec<Cell>,
    object_cell: HashMap<ObjectId, CellId>,
    min_cells: usize,
    rank: CellRank,
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
            born_cells: Vec::new(),
            object_cell: HashMap::new(),
            min_cells: 0,
            rank: CellRank::new(RANK_SIZE),
        }
    }

    pub fn with_min_cells(mut self, min_cells: usize) -> Self {
        self.min_cells = min_cells;
        self
    }

    pub fn time(&self) -> Scalar {
        self.time
    }

    pub fn add_testing_cell(&mut self) -> CellId {
        let position = Vec2::new(20.0, 200.0);
        let radius = 10.0;
        let object_id = self.physics.add_object(position, radius);
        let cell_id = self.next_cell_id;
        self.next_cell_id += 1;
        let mut cell = Cell::random(object_id, radius);
        cell.molecules.set_zero();
        cell.energy = 10000.0;
        cell.movement_speed_limit = 10.0;
        cell.movement_direction = 0.20 * Scalar::PI();
        cell.movement_speed = 10.0;
        self.cells.insert(cell_id, cell);
        self.object_cell.insert(object_id, cell_id);
        cell_id
    }

    fn add_cell(&mut self, _genome: Genome) {
        // Cell::new(genome);
        todo!()
    }

    pub fn add_random_cell(&mut self) -> CellId {
        let mut rng = rand::thread_rng();

        let radius = rng.gen_range(1.0..=MAX_SIZE);
        let position = self.find_free_position(&mut rng, radius);

        let object_id = self.physics.add_object(position, radius);

        let cell_id = self.next_cell_id;
        self.next_cell_id += 1;
        let cell = Cell::random(object_id, radius);
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
        self.add_born_cells();
    }

    fn handle_contacts(&mut self, dt: Scalar) {
        for (_, cell) in self.cells.iter_mut() {
            cell.contact_count = 0.0;
        }
        for contact in self.physics.contacts() {
            // println!(">>>");
            match contact {
                Contact::Surface { id, normal } => {
                    self.object_cell
                        .get(id)
                        .and_then(|cell_id| self.cells.get_mut(cell_id))
                        .iter_mut()
                        .for_each(|cell| cell.on_surface_contact(*normal));
                }
                Contact::Objects { id1, id2, normal } => {
                    let cell1 = self
                        .object_cell
                        .get(id1)
                        .and_then(|cell_id| self.cells.get(cell_id).map(|cell| (cell_id, cell)));
                    let cell2 = self
                        .object_cell
                        .get(id2)
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
                            cell1.on_cell_contact(delta1 - delta2, *normal);
                        }
                        if let Some(cell2) = self.cells.get_mut(id2) {
                            // println!("{:.4} {:.4} {:.4}", cell2.energy, delta2 - delta1, cell2.energy + delta2 - delta1);
                            cell2.on_cell_contact(delta2 - delta1, *normal);
                        }
                    }
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
            } else if cell.should_divide() {
                let born_cell = cell.divide(&mut self.physics);
                self.born_cells.push(born_cell);
            }
        }
    }

    fn remove_dead_cells(&mut self) {
        for cell_id in self.dead_cells.drain(..) {
            if let Some(cell) = self.cells.remove(&cell_id) {
                // TODO transfer any remaining molecules/energy to the world
                let object_id = cell.object_id;
                self.physics.remove_object(object_id);
                let fitness_score = Self::energy_fitness_score(&cell);
                self.rank.insert(fitness_score, cell);
            }
        }
    }

    fn add_born_cells(&mut self) {
        for born_cell in self.born_cells.drain(..) {
            self.cells.insert(self.next_cell_id, born_cell);
            self.next_cell_id += 1;
        }

        while self.cells.len() < self.min_cells {
            if let Some(genome) = self.create_recombined_genome() {
                self.add_cell(genome);
            } else {
                self.add_random_cell();
            }
        }
    }

    fn create_recombined_genome(&self) -> Option<Genome> {
        let genome1 = self.rank.choose_random_genome();
        let genome2 = self.rank.choose_random_genome();
        genome1
            .zip(genome2)
            .map(|(genome1, genome2)| genome1.cross(&genome2))
    }

    fn energy_fitness_score(cell: &Cell) -> Scalar {
        let stats = &cell.stats;
        let energy_positive = stats.energy_produced + stats.energy_absorbed_in;
        let energy_negative = stats.energy_consumed + stats.energy_absorbed_out;
        (1.0 + energy_positive) / (1.0 + energy_negative)
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
