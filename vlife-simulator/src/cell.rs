use nalgebra::UnitComplex;
use num_traits::float::FloatConst;
use rand::Rng;
use std::ops::Neg;

use vlife_macros::BuildGenome;

use crate::physics::{Object, ObjectId, Physics};
use crate::{neurons::Neurons, simulator::SimulationContext, V};
use crate::{Scalar, Vec2};

pub const NUM_MOLECULES: usize = 8;

pub const MAX_ENERGY: Scalar = 1000.0;
pub const ALIVE_ENERGY_THRESHOLD: Scalar = 0.1;
pub const MAX_ZERO_ENERGY_TIME: Scalar = 60.0;
pub const MAX_DIVISION_THRESHOLD_FACTOR: Scalar = 10.0;
pub const MAX_MOLECULE_AMOUNT: Scalar = 100.0;
pub const MAX_MOLECULE_ENERGY_CONVERSION: Scalar = 1.0;
pub const MAX_CONTRACTION: Scalar = 0.8;
pub const MAX_CONTACT_ENERGY_ABSORPTION: Scalar = 0.8;
pub const MAX_SIZE: Scalar = 6.0;
pub const MAX_SPEED: Scalar = 40.0;
pub const NEURON_COST: Scalar = 0.0000005;
pub const MOVEMENT_COST: Scalar = 0.0001;
pub const CONTRACTION_COST: Scalar = 0.0001;
pub const DIVISION_COST: Scalar = 0.001;

/// Model for a cell.
#[derive(BuildGenome)]
pub struct Cell {
    /// Reference to the Physics object.
    pub(crate) object_id: ObjectId,

    /// Neuronal network.
    /// This is used to model complex behaviour based on external/internal signals.
    /// External signals are related to perception, while internal ones allow for feedback loops.
    /// The outputs can be used to represent impulses, like the contraction. But they are also
    /// used to model the regulation of the genome expression, like the
    /// `contact_energy_absorption_amount` which represents the amount of membrane channels
    /// used to absorb energy from other cells.
    /// Source: Weights and biases from the Genome.
    #[build_genome(nested)]
    pub(crate) neurons: Neurons,

    /// Timestamp when it was born.
    pub(crate) age: Scalar,
    /// The size of the cell when there is no contraction.
    #[build_genome(gen)]
    pub(crate) size: Scalar,
    /// The area of the cell when there is no contraction.
    pub(crate) area: Scalar,
    /// The amount of energy in the cell. Energy can be generated from internal molecules
    /// as well as absorbed from other cells.
    pub(crate) energy: Scalar,
    /// Last energy level used to calculate the energy delta of an step. Source: Processing.
    pub(crate) last_energy: Scalar,
    /// Amount of energy that could be obtained from the molecules.
    /// This is processed from the amount of existing molecules and their conversion to energy.
    pub(crate) stored_energy: Scalar,
    /// Maximum time the cell can stay alive with zero energy.
    #[build_genome(gen)]
    pub(crate) zero_energy_limit: Scalar,
    /// Time that the cell has remained with zero energy.
    pub(crate) zero_energy_time: Scalar,

    /// The cells reserve energy for the division.
    pub(crate) division_energy_reserve: Scalar,
    /// Amount of energy required to start division.
    #[build_genome(gen)]
    pub(crate) division_threshold: Scalar,
    /// After division, while a cell is growing, this is the ratio of the maximum cell size.
    pub(crate) division_grow_factor: Scalar,

    // Cells have molecules floating around that can be used for different purposes
    /// Amount of molecules floating around. There are NUM_MOLECULES different types.
    pub(crate) molecules: V<NUM_MOLECULES>,
    /// Conversion ratio for a unit of molecule to energy.
    /// This determines the ability of the cell to create energy from the molecules,
    /// or to store energy as molecules. Source: Genome.
    #[build_genome(nested)]
    pub(crate) molecules_energy_conversion: V<NUM_MOLECULES>,

    // Cells have cilia that allow them to move
    /// Direction of the cell movement (in radians). Source: Neurons.
    pub(crate) movement_direction: Scalar,
    /// Maximum speed of the cell movement. Source: Genome.
    #[build_genome(gen)]
    pub(crate) movement_speed_limit: Scalar,
    /// Speed of the cilia movement. Source: Neurons.
    pub(crate) movement_speed: Scalar,
    /// Last calculated velocity for the cell movement. Source: Processing.
    pub(crate) movement_velocity: Vec2,

    // Cells can contract like a muscle.
    /// Maximum contraction ratio respect the cell size. Source: Genome.
    #[build_genome(gen)]
    pub(crate) contraction_limit: Scalar,
    /// Contraction ratio respect the cell size. Source: Neurons.
    pub(crate) contraction_amount: Scalar,

    // When cells contact with others, energy can diffuse between them
    // thanks to special transporters in their membrane. The expression
    // of those transporters is regulated by neurons.
    /// Maximum amount of energy that can be absorbed from another cell. Source: Genome.
    #[build_genome(gen)]
    pub(crate) contact_energy_absorption_limit: Scalar,
    /// Amount of energy that can be absorbed from another cell. Source: Neurons.
    pub(crate) contact_energy_absorption_amount: Scalar,
    /// Number of contacts with other cells. Source: Physics.
    pub(crate) contact_count: Scalar,
    /// The normal of all the contacts.
    pub(crate) contact_normal: Vec2,

    pub(crate) stats: CellStats,
}

impl Cell {
    pub fn random(object_id: ObjectId, size: Scalar) -> Self {
        let mut rng = rand::thread_rng();
        let area = Scalar::PI() * size * size;
        Self {
            object_id,
            neurons: Neurons::random(),
            age: 0.0,
            size,
            area,
            energy: MAX_ENERGY,
            last_energy: MAX_ENERGY,
            stored_energy: 0.0,
            zero_energy_limit: rng.gen_range(0.0..=MAX_ZERO_ENERGY_TIME),
            zero_energy_time: 0.0,
            division_energy_reserve: 0.0,
            division_threshold: area * rng.gen_range(1.0..=MAX_DIVISION_THRESHOLD_FACTOR),
            division_grow_factor: 1.0,
            molecules: V::from_fn(|_, _| rng.gen_range(0.0..=MAX_MOLECULE_AMOUNT)),
            molecules_energy_conversion: V::from_fn(|_, _| {
                rng.gen_range(0.0..MAX_MOLECULE_ENERGY_CONVERSION)
            }),
            movement_direction: 0.0,
            movement_speed_limit: rng.gen_range(0.0..=MAX_SPEED),
            movement_speed: 0.0,
            movement_velocity: Vec2::zeros(),
            contraction_limit: rng.gen_range(0.0..=MAX_CONTRACTION),
            contraction_amount: 0.0,
            contact_energy_absorption_limit: rng.gen_range(0.0..=MAX_CONTACT_ENERGY_ABSORPTION),
            contact_energy_absorption_amount: 0.0,
            contact_count: 0.0,
            contact_normal: Vec2::zeros(),
            stats: CellStats::default(),
        }
    }

    pub fn child_from(
        object_id: ObjectId,
        cell: &Cell,
        energy: Scalar,
        molecules: V<NUM_MOLECULES>,
    ) -> Cell {
        Self {
            object_id,
            neurons: cell.neurons.clone(),
            age: 0.0,
            size: cell.size,
            area: cell.area,
            energy,
            last_energy: energy,
            stored_energy: 0.0,
            zero_energy_limit: cell.zero_energy_limit,
            zero_energy_time: 0.0,
            division_energy_reserve: 0.0,
            division_threshold: cell.division_threshold,
            division_grow_factor: cell.size.recip(),
            molecules,
            molecules_energy_conversion: cell.molecules_energy_conversion,
            movement_direction: cell.movement_direction,
            movement_speed_limit: cell.movement_speed_limit,
            movement_speed: cell.movement_speed,
            movement_velocity: Vec2::zeros(),
            contraction_limit: cell.contraction_limit,
            contraction_amount: 0.0,
            contact_energy_absorption_limit: cell.contact_energy_absorption_limit,
            contact_energy_absorption_amount: 0.0,
            contact_count: 0.0,
            contact_normal: Vec2::zeros(),
            stats: CellStats::default(),
        }
    }

    pub fn energy(&self) -> Scalar {
        self.energy
    }

    pub fn energy_delta(&self) -> Scalar {
        self.energy - self.last_energy
    }

    pub fn basal_energy(&self) -> Scalar {
        self.neurons.num_working_neurons() * NEURON_COST
    }

    pub fn size(&self) -> Scalar {
        self.size
    }

    pub fn contracted_size(&self) -> Scalar {
        (self.size * (1.0 - self.contraction_amount) * self.division_grow_factor).max(1.0)
    }

    pub fn movement_normal(&self) -> Vec2 {
        UnitComplex::new(-self.movement_direction).transform_vector(&Vec2::x())
    }

    pub fn movement_direction(&self) -> Scalar {
        self.movement_direction
    }

    pub fn movement_velocity(&self) -> Vec2 {
        self.movement_velocity
    }

    pub fn should_divide(&self) -> bool {
        self.energy >= self.division_cost()
            && self.division_energy_reserve >= self.division_threshold
            && self.division_grow_factor >= 1.0
    }

    fn division_cost(&self) -> Scalar {
        self.area * DIVISION_COST
    }

    pub fn is_dead(&self) -> bool {
        self.energy + self.stored_energy <= ALIVE_ENERGY_THRESHOLD
            || self.zero_energy_time >= self.zero_energy_limit
    }

    pub fn energy_diffusion(&self) -> Scalar {
        self.energy * self.energy_permeability()
    }

    pub fn energy_permeability(&self) -> Scalar {
        // 1.0 - (0.5 + absorption).recip()
        1.5 * (1.0 + (-5.0 * self.contact_energy_absorption_amount).exp()).recip() - 0.5
    }

    pub fn energy_absorption_from(&self, other: &Cell, dt: Scalar) -> Scalar {
        // println!(
        //     "a: {:.4} eo: {:.4} r: {:.4} dt: {:.4}",
        //     self.contact_energy_absorption_amount,
        //     other.energy,
        //     resistance(other.contact_energy_absorption_amount),
        //     dt
        // );
        self.contact_energy_absorption_amount * other.energy_diffusion() * dt
    }

    pub fn on_surface_contact(&mut self, normal: Vec2) {
        self.contact_count += 1.0;
        self.contact_normal += normal
    }

    pub fn on_cell_contact(&mut self, energy_delta: Scalar, normal: Vec2) {
        self.energy += energy_delta;
        if energy_delta > 0.0 {
            self.stats.update_energy_absorbed_in(energy_delta);
        } else if energy_delta < 0.0 {
            self.stats.update_energy_absorbed_out(-energy_delta);
        }
        self.contact_count += 1.0;
        self.contact_normal += normal
    }

    pub fn update(&mut self, dt: Scalar, context: SimulationContext<'_>) {
        let energy_delta = self.energy - self.last_energy;
        self.last_energy = self.energy;
        self.age += dt;

        self.process_neurons(dt, energy_delta, &context);

        let basal_energy = self.basal_energy().min(self.energy);
        self.energy -= basal_energy;
        self.stats.update_energy_consumed(basal_energy);

        self.compute_contraction(dt);
        self.compute_movement(dt, context.object);
        self.compute_contact_energy_absorption(dt);
        self.compute_energy_metabolism(dt);
        self.compute_division(dt);

        if self.energy <= ALIVE_ENERGY_THRESHOLD {
            self.zero_energy_time += dt;
        } else {
            self.zero_energy_time = 0.0;
        }
    }

    fn process_neurons(&mut self, _dt: Scalar, energy_delta: Scalar, context: &SimulationContext) {
        // self.neurons.set_velocity_pos(&context.object.velocity());
        self.neurons
            .set_velocity_magnitude(context.object.velocity().magnitude());
        // self.neurons
        //     .set_acceleration_pos(&context.object.acceleration());
        self.neurons
            .set_acceleration_magnitude(context.object.acceleration().magnitude());
        self.neurons.set_radius(context.object.radius());

        self.neurons.set_age(self.age);
        self.neurons.set_energy_amount(self.energy);
        self.neurons.set_energy_stored(self.stored_energy);
        self.neurons.set_energy_delta(energy_delta);
        self.neurons
            .set_zero_energy(self.zero_energy_time / self.zero_energy_limit);
        self.neurons
            .set_division_energy_reserve(self.division_energy_reserve / self.division_threshold);
        self.neurons
            .set_division_grow_factor(self.division_grow_factor);

        self.neurons
            .set_molecules_proportion(&self.molecules.normalize());
        self.neurons.set_molecules_total(self.molecules.sum());

        self.neurons.set_movement_direction(self.movement_direction);
        self.neurons.set_movement_speed(self.movement_speed);
        self.neurons.set_movement_velocity(&self.movement_velocity);
        self.neurons
            .set_movement_velocity_magnitude(self.movement_velocity.magnitude());

        self.neurons.set_contact_energy_absorption(
            self.contact_energy_absorption_amount / self.contact_energy_absorption_limit,
        );
        self.neurons.set_contact_count(self.contact_count);
        if self.contact_count > 0.0 {
            self.neurons
                .set_contact_normal(&self.contact_normal.normalize());
            self.neurons
                .set_contact_normal_magnitude(self.contact_normal.magnitude());
        } else {
            self.neurons.set_contact_normal(&Vec2::zeros());
            self.neurons.set_contact_normal_magnitude(0.0);
        }

        self.neurons.process();
    }

    fn compute_contraction(&mut self, dt: Scalar) {
        let contraction_energy = self.contraction_amount * CONTRACTION_COST * dt;
        if self.energy >= contraction_energy {
            self.energy -= contraction_energy;
            self.stats.update_energy_consumed(contraction_energy);
            self.contraction_amount =
                self.neurons.get_contraction_amount().max(0.0) * self.contraction_limit;
        }
    }

    fn compute_movement(&mut self, dt: Scalar, object: &Object) {
        let two_pi = 2.0 * Scalar::PI();

        // let direction = self.neurons.movement_direction_out().abs() * two_pi;
        let direction = self.movement_direction
            + self.neurons.get_movement_angular_speed() * (0.05 * Scalar::PI());
        let speed = self.neurons.get_movement_kinetic_speed().max(0.0) * self.movement_speed_limit;

        let kinetic_energy = 0.5 * object.mass() * speed * speed;
        let movement_energy = kinetic_energy * MOVEMENT_COST * dt;
        if self.energy >= movement_energy {
            self.energy -= movement_energy;
            self.stats.update_energy_consumed(movement_energy);
            self.movement_direction = direction % two_pi;
            let rotation = UnitComplex::new(-self.movement_direction);
            self.movement_speed = speed;
            self.movement_velocity = rotation * Vec2::x_axis().scale(self.movement_speed);
        } else {
            self.movement_speed = 0.0;
            self.movement_velocity = Vec2::zeros();
        }
    }

    fn compute_contact_energy_absorption(&mut self, dt: Scalar) {
        let amount = self.contact_energy_absorption_amount
            + self.neurons.get_contact_energy_absorption() * dt;
        self.contact_energy_absorption_amount =
            amount.clamp(0.0, self.contact_energy_absorption_limit);
    }

    fn compute_energy_metabolism(&mut self, dt: Scalar) {
        let initial_energy = self.energy;

        let zeros = V::<NUM_MOLECULES>::from_element(0.0);
        let source_molecules =
            (V::<NUM_MOLECULES>::from(self.neurons.get_energy_metabolism()).sup(&zeros) * dt)
                .inf(&self.molecules);
        let produced_energy_per_molecule =
            source_molecules.component_mul(&self.molecules_energy_conversion);
        let produced_energy = produced_energy_per_molecule.sum();
        debug_assert!(produced_energy_per_molecule.fold(true, |positive, x| positive && x >= 0.0));
        debug_assert!(produced_energy >= 0.0);
        self.molecules -= source_molecules;
        self.energy += produced_energy;

        let mut produced_molecules =
            V::<NUM_MOLECULES>::from(self.neurons.get_energy_metabolism().neg()).sup(&zeros) * dt;
        let required_energy_per_molecule =
            produced_molecules.component_mul(&self.molecules_energy_conversion);
        let mut consumed_energy = required_energy_per_molecule.sum();
        if consumed_energy > self.energy {
            let factor = self.energy / consumed_energy;
            produced_molecules *= factor;
            consumed_energy = self.energy;
        }

        self.molecules += produced_molecules;
        self.energy -= consumed_energy;

        if self.energy > initial_energy {
            self.stats
                .update_energy_produced(self.energy - initial_energy);
        } else if self.energy < initial_energy {
            self.stats
                .update_energy_consumed(initial_energy - self.energy);
        }

        self.stored_energy = self
            .molecules
            .component_mul(&self.molecules_energy_conversion)
            .sum();
    }

    fn compute_division(&mut self, dt: Scalar) {
        let energy_delta = (self.neurons.get_division_energy_reserve() * dt)
            .max(self.division_energy_reserve.neg())
            .min(self.energy)
            .min(self.division_threshold - self.division_energy_reserve);
        self.division_energy_reserve += energy_delta;
        self.energy -= energy_delta;
        self.division_grow_factor = (self.division_grow_factor + dt).min(1.0);
    }

    pub fn divide(&mut self, physics: &mut Physics) -> Cell {
        self.energy -= self.division_cost();
        let object = physics.get_object(self.object_id).expect("cell-object");
        let rotation = UnitComplex::new(-self.movement_direction);
        let grow_vec = rotation * Vec2::x().scale(self.contracted_size() - 0.5);
        let position = object.position() - grow_vec;
        let new_object_id = physics.add_object(position, 1.0);
        let energy_reserve = self.division_energy_reserve;
        self.division_energy_reserve = 0.0;
        let molecules = self.molecules * 0.5;
        self.molecules = molecules;
        Cell::child_from(new_object_id, self, energy_reserve, molecules)
    }
}
//
// impl BuildGenome for Cell {
//     fn build_genome<'a>(&self, genome: GenomeBuilder) {
//         self.neurons.build_genome(genome.nested("neurons"));
//         genome.add("size", Gen { value: self.size });
//     }
// }

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total_energy = self.energy + self.stored_energy;
        let energy_delta = self.energy_delta();
        let days = (self.age * (1.0 / 86400.0)).floor();
        let hours = (self.age * (1.0 / 3600.0)).floor();
        let minutes = (self.age * (1.0 / 60.0)).floor();
        let seconds = self.age % 60.0;
        writeln!(
            f,
            "Age> Days: {:.0}, Time: {:02.0}:{:02.0}:{:04.1}",
            days, hours, minutes, seconds
        )?;
        writeln!(
            f,
            "Energy> Available: {:6.2} ({:3.0} %), Stored: {:6.2} ({:3.0} %), Delta: {:7.4}, Basal: {:7.4} Zero: {:5.1} / {:5.1}",
            self.energy,
            self.energy * 100.0 / total_energy,
            self.stored_energy,
            self.stored_energy * 100.0 / total_energy,
            energy_delta,
            self.basal_energy(),
            self.zero_energy_time,
            self.zero_energy_limit,
        )?;
        writeln!(
            f,
            "Division> Reserve: {:6.2} ({:3.0} %), Threshold: {:6.2}, Signal: {:4.2}",
            self.division_energy_reserve,
            self.division_energy_reserve * 100.0 / self.division_threshold,
            self.division_threshold,
            self.neurons.get_division_energy_reserve()
        )?;
        writeln!(
            f,
            "Molecules>  {:5.1?}, Total: {:.1?}",
            self.molecules,
            self.molecules.sum()
        )?;
        writeln!(f, "Conversion: {:5.1?}", self.molecules_energy_conversion)?;
        writeln!(
            f,
            "Regulation:  {:5.1?}",
            self.neurons.get_energy_metabolism().as_slice(),
        )?;
        writeln!(
            f,
            "Contact> Count: {:2.0}, Energy Absorption: {:3.0} % ({:.4} / {:.4}), Permeability: {:3.1}, Diffusion: {:3.1}",
            self.contact_count,
            self.contact_energy_absorption_amount * 100.0 / self.contact_energy_absorption_limit,
            self.contact_energy_absorption_amount,
            self.contact_energy_absorption_limit,
            self.energy_permeability(),
            self.energy_diffusion(),
        )?;
        writeln!(
            f,
            "Movement> Speed: {:3.0} % ({:6.2} / {:6.2}), Dir: {:3.0}",
            self.movement_speed * 100.0 / self.movement_speed_limit,
            self.movement_speed,
            self.movement_speed_limit,
            self.movement_direction * 360.0 / (2.0 * Scalar::PI()),
        )?;
        let contracted_size = self.contracted_size();
        writeln!(
            f,
            "Contraction> Size: {:3.0} % ({:5.1} / {:5.1}), Amount: {:6.2} / {:6.2}",
            100.0 - (contracted_size * 100.0 / self.size),
            contracted_size,
            self.size,
            self.contraction_amount,
            self.contraction_limit,
        )?;
        let energy_positive = self.stats.energy_produced + self.stats.energy_absorbed_in;
        let energy_negative = self.stats.energy_consumed + self.stats.energy_absorbed_out;
        writeln!(
            f,
            "Stats> Energy Consumed: {:5.1}, Produced: {:5.1}, Absorbed Out: {:5.1}, Absorbed In: {:5.1}, Net: {:5.1}, Ratio: {:6.3}",
            self.stats.energy_consumed,
            self.stats.energy_produced,
            self.stats.energy_absorbed_out,
            self.stats.energy_absorbed_in,
            energy_positive - energy_negative,
            (1.0 + energy_positive) / (1.0 + energy_negative),
        )?;
        write!(f, "{}", self.neurons)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CellStats {
    pub energy_consumed: Scalar,
    pub energy_produced: Scalar,
    pub energy_absorbed_out: Scalar,
    pub energy_absorbed_in: Scalar,
}

impl CellStats {
    fn update_energy_consumed(&mut self, amount: Scalar) {
        self.energy_consumed += amount;
    }

    fn update_energy_produced(&mut self, amount: Scalar) {
        self.energy_produced += amount;
    }

    fn update_energy_absorbed_out(&mut self, amount: Scalar) {
        self.energy_absorbed_out += amount;
    }

    fn update_energy_absorbed_in(&mut self, amount: Scalar) {
        self.energy_absorbed_in += amount;
    }
}
