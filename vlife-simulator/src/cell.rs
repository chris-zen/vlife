use nalgebra::UnitComplex;
use num_traits::float::FloatConst;
use rand::Rng;

use vlife_physics::{Object, ObjectId, Scalar, Vec2};

use crate::{neurons::Neurons, simulator::SimulationContext, M, V};

pub const NUM_MOLECULES: usize = 8;

pub const NEURON_COST: Scalar = 0.000001;
pub const MAX_ENERGY: Scalar = 100.0;
pub const ALIVE_ENERGY_THRESHOLD: Scalar = 0.1;
pub const MAX_ZERO_ENERGY_TIME: Scalar = 60.0;
pub const MAX_MOLECULE_AMOUNT: Scalar = 100.0;
pub const MAX_MOLECULE_ENERGY_CONVERSION: Scalar = 1.0;
pub const MAX_MOVEMENT_COST: Scalar = 0.0005;
pub const MAX_CONTRACTION_COST: Scalar = 0.001;
pub const MAX_CONTRACTION: Scalar = 0.8;
pub const MAX_CONTACT_ENERGY_ABSORPTION: Scalar = 0.8;
pub const MAX_SIZE: Scalar = 10.0;
pub const MAX_SPEED: Scalar = 40.0;

/// Model for a cell.
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
    pub(crate) neurons: Neurons,

    /// Timestamp when it was born.
    pub(crate) _born_time: Scalar,
    /// The size of the cell when there is no contraction.
    pub(crate) size: Scalar,
    /// The amount of energy in the cell. Energy can be generated from internal molecules
    /// as well as absorbed from other cells.
    pub(crate) energy: Scalar,
    /// Last energy level used to calculate the energy delta of an step. Source: Processing.
    pub(crate) last_energy: Scalar,
    /// Amount of energy that could be obtained from the molecules.
    /// This is processed from the amount of existing molecules and their conversion to energy.
    pub(crate) stored_energy: Scalar,
    /// Maximum time the cell can stay alive with zero energy.
    pub(crate) zero_energy_limit: Scalar,
    /// Time that the cell has remained with zero energy.
    pub(crate) zero_energy_time: Scalar,

    // Cells have molecules floating around that can be used for different purposes
    /// Amount of molecules floating around. There are NUM_MOLECULES different types.
    pub(crate) molecules: V<NUM_MOLECULES>,
    /// Conversion ratio for a unit of molecule to energy.
    /// This determines the ability of the cell to create energy from the molecules,
    /// or to store energy as molecules. Source: Genome.
    pub(crate) molecules_energy_conversion: V<NUM_MOLECULES>,

    // Cells have cilia that allow them to move
    /// Cost of moving. Source: Genome.
    pub(crate) movement_cost: Scalar,
    /// Direction of the cell movement (in radians). Source: Neurons.
    pub(crate) movement_direction: Scalar,
    /// Maximum speed of the cell movement. Source: Genome.
    pub(crate) movement_speed_limit: Scalar,
    /// Speed of the cilia movement. Source: Neurons.
    pub(crate) movement_speed: Scalar,
    /// Last calculated velocity for the cell movement. Source: Processing.
    pub(crate) movement_velocity: Vec2,

    // Cells can contract like a muscle.
    /// Cost of contracting. Source: Genome.
    pub(crate) contraction_cost: Scalar,
    /// Maximum contraction ratio respect the cell size. Source: Genome.
    pub(crate) contraction_limit: Scalar,
    /// Contraction ratio respect the cell size. Source: Neurons.
    pub(crate) contraction_amount: Scalar,

    // When cells contact with others, energy can diffuse between them
    // thanks to special transporters in their membrane. The expression
    // of those transporters is regulated by neurons.
    /// Maximum amount of energy that can be absorbed from another cell. Source: Genome.
    pub(crate) contact_energy_absorption_limit: Scalar,
    /// Amount of energy that can be absorbed from another cell. Source: Neurons.
    pub(crate) contact_energy_absorption_amount: Scalar,
    /// Number of contacts with other cells. Source: Physics.
    pub(crate) contact_count: Scalar,
    /// The normal of all the contacts.
    pub(crate) contact_normal: Vec2,
}

impl Cell {
    pub fn random(time: Scalar, object_id: ObjectId, size: Scalar) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            object_id,
            neurons: Neurons::random(),
            _born_time: time,
            size,
            energy: MAX_ENERGY,
            last_energy: MAX_ENERGY,
            stored_energy: 0.0,
            zero_energy_limit: rng.gen_range(0.0..=MAX_ZERO_ENERGY_TIME),
            zero_energy_time: 0.0,
            molecules: V::from_fn(|_, _| rng.gen_range(0.0..=MAX_MOLECULE_AMOUNT)),
            molecules_energy_conversion: V::from_fn(|_, _| {
                rng.gen_range(0.0..MAX_MOLECULE_ENERGY_CONVERSION)
            }),
            movement_cost: rng.gen_range(0.0003..MAX_MOVEMENT_COST),
            movement_direction: 0.0,
            movement_speed_limit: rng.gen_range(0.0..=MAX_SPEED),
            movement_speed: 0.0,
            movement_velocity: Vec2::zeros(),
            contraction_cost: rng.gen_range(0.0003..MAX_CONTRACTION_COST),
            contraction_limit: rng.gen_range(0.0..=MAX_CONTRACTION),
            contraction_amount: 0.0,
            contact_energy_absorption_limit: rng.gen_range(0.0..=MAX_CONTACT_ENERGY_ABSORPTION),
            contact_energy_absorption_amount: 0.0,
            contact_count: 0.0,
            contact_normal: Vec2::zeros(),
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
        (self.size * (1.0 - self.contraction_amount)).max(1.0)
    }

    pub fn movement_normal(&self) -> Vec2 {
        UnitComplex::new(self.movement_direction).transform_vector(&Vec2::x_axis())
    }

    pub fn movement_direction(&self) -> Scalar {
        self.movement_direction
    }

    pub fn movement_velocity(&self) -> Vec2 {
        self.movement_velocity
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

    pub fn update(&mut self, dt: Scalar, context: SimulationContext<'_>) {
        self.process_neurons(dt, &context);

        self.last_energy = self.energy;
        self.energy -= self.basal_energy();

        self.compute_contraction(dt);
        self.compute_movement(dt, context.object);
        self.compute_metabolism(dt /*context.reactions*/);
        self.compute_contact_energy_absorption(dt);
        self.compute_produced_energy(dt);

        if self.energy <= ALIVE_ENERGY_THRESHOLD {
            self.zero_energy_time += dt;
        } else {
            self.zero_energy_time = 0.0;
        }
    }

    fn process_neurons(&mut self, _dt: Scalar, context: &SimulationContext) {
        self.neurons.set_velocity_pos(&context.object.velocity());
        self.neurons
            .set_velocity_magnitude(context.object.velocity().magnitude());
        self.neurons
            .set_acceleration_pos(&context.object.acceleration());
        self.neurons
            .set_acceleration_magnitude(context.object.acceleration().magnitude());
        self.neurons.set_radius(context.object.radius());

        self.neurons.set_energy_amount(self.energy);
        self.neurons
            .set_energy_delta(self.energy - self.last_energy);
        self.neurons.set_energy_stored(self.stored_energy);

        self.neurons.set_molecules_amount(&self.molecules);
        self.neurons.set_molecules_total(self.molecules.sum());

        self.neurons.set_movement_direction(self.movement_direction);
        self.neurons.set_movement_speed(self.movement_speed);

        self.neurons
            .set_contact_energy_absorption(self.contact_energy_absorption_amount);
        self.neurons.set_contact_count(self.contact_count);
        self.neurons.set_contact_normal(&self.contact_normal);

        self.neurons.process();
    }

    fn compute_contraction(&mut self, dt: Scalar) {
        let contraction_energy = self.contraction_amount * self.contraction_cost * dt;
        if self.energy >= contraction_energy {
            self.energy -= contraction_energy;
            self.contraction_amount =
                self.neurons.contraction_out().max(0.0) * self.contraction_limit;
        }
    }

    fn compute_movement(&mut self, dt: Scalar, object: &Object) {
        let two_pi = 2.0 * Scalar::PI();

        // let direction = self.neurons.movement_direction_out().abs() * two_pi;
        let direction =
            self.movement_direction + self.neurons.movement_direction_out() * (0.05 * Scalar::PI());
        let speed = self.neurons.movement_speed_out().max(0.0) * self.movement_speed_limit;

        let kinetic_energy = 0.5 * object.mass() * speed * speed;
        let movement_energy = kinetic_energy * self.movement_cost * dt;
        if self.energy >= movement_energy {
            self.energy -= movement_energy;
            self.movement_direction = direction % two_pi;
            let rotation = UnitComplex::new(-self.movement_direction);
            self.movement_speed = speed;
            self.movement_velocity = rotation * Vec2::x_axis().scale(self.movement_speed);
        } else {
            self.movement_speed = 0.0;
            self.movement_velocity = Vec2::zeros();
        }
    }

    fn compute_metabolism(
        &mut self,
        dt: Scalar, /*reactions: &M<NUM_MOLECULES, NUM_MOLECULES>*/
    ) {
        let metabolism_factors = self.neurons.metabolism_factors_out().abs();
        // println!(">>> -----");
        let mut metabolism =
            M::<NUM_MOLECULES, NUM_MOLECULES>::from_iterator(metabolism_factors.iter().copied());
        metabolism.fill_diagonal(1.0);
        // println!("{:.4}", metabolism);
        let substrates = V::<NUM_MOLECULES>::from_element(dt).inf(&self.molecules);
        // println!("{:.4} {:.4}", substrates.transpose(), substrates.sum());
        for (index, mut metabolism_row) in metabolism.row_iter_mut().enumerate() {
            let f = substrates[index] * metabolism_row.sum().recip();
            metabolism_row.apply(|x| *x *= f);
        }
        // println!("{:.4} {:.4} {:.4}", metabolism, metabolism.row_sum(), metabolism.row_sum().sum());
        let mut products = V::<NUM_MOLECULES>::zeros();
        for (index, metabolism_col) in metabolism.column_iter().enumerate() {
            // products[index] = metabolism_col.dot(&reactions.column(index));
            products[index] = metabolism_col.sum();
        }
        // println!("{:.4} {:.4}", self.molecules.transpose(), self.molecules.sum());
        // println!("{:.4} {:.4}", products.transpose(), products.sum());
        self.molecules += products - substrates;
        // println!("{:.4} {:.4}", self.molecules.transpose(), self.molecules.sum());
        // panic!();
    }

    fn compute_contact_energy_absorption(&mut self, dt: Scalar) {
        let amount = self.contact_energy_absorption_amount
            + self.neurons.contact_energy_absorption_out() * dt;
        self.contact_energy_absorption_amount =
            amount.clamp(0.0, self.contact_energy_absorption_limit);
    }

    fn compute_produced_energy(&mut self, dt: Scalar) {
        let zeros = V::<NUM_MOLECULES>::from_element(0.0);
        let energy_source = V::<NUM_MOLECULES>::from(self.neurons.energy_source_out()).sup(&zeros);
        let energy_source = (energy_source.component_mul(&self.molecules_energy_conversion) * dt)
            .inf(&self.molecules);
        // println!(">>> =================");
        // println!("energy_source: {:.2}", energy_source.transpose());
        let produced_energy = energy_source.sum();
        // println!("produced energy: {:.2}", produced_energy);

        // if self.energy + produced_energy > MAX_ENERGY {
        //     let factor = (MAX_ENERGY - self.energy) / produced_energy;
        //     // println!("factor: {:.2}", factor);
        //     energy_source *= factor;
        //     produced_energy *= factor;
        // }

        // println!("{:.2}", energy_source);
        if energy_source.fold(true, |positive, x| positive && x >= 0.0) {
            debug_assert!(energy_source.fold(true, |positive, x| positive && x >= 0.0));
            debug_assert!(produced_energy >= 0.0);
            self.molecules -= energy_source;
            self.energy += produced_energy;
        }

        self.stored_energy = self
            .molecules
            .component_mul(&self.molecules_energy_conversion)
            .sum();
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total_energy = self.energy + self.stored_energy;
        let energy_delta = self.energy_delta();
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
            "Movement> Speed: {:3.0} % ({:6.2} / {:6.2}), Dir: {:3.0}, Cost: {:.6}",
            self.movement_speed * 100.0 / self.movement_speed_limit,
            self.movement_speed,
            self.movement_speed_limit,
            self.movement_direction * 360.0 / (2.0 * Scalar::PI()),
            self.movement_cost
        )?;
        let contracted_size = self.contracted_size();
        writeln!(
            f,
            "Contraction> Size: {:3.0} % ({:5.1} / {:5.1}), Amount: {:6.2} / {:6.2}, Cost: {:.6}",
            contracted_size * 100.0 / self.size,
            contracted_size,
            self.size,
            self.contraction_amount,
            self.contraction_limit,
            self.contraction_cost
        )?;
        writeln!(
            f,
            "Molecules> {:4.1?}, Total: {:.1?}",
            self.molecules,
            self.molecules.sum()
        )?;
        writeln!(
            f,
            "           Production: {:4.1?}",
            self.molecules_energy_conversion
        )?;
        writeln!(
            f,
            "           Conversion:  {:4.1?}",
            self.neurons.energy_source_out().as_slice(),
        )?;
        write!(f, "{}", self.neurons)?;
        Ok(())
    }
}
