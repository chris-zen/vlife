use nalgebra::UnitComplex;
use num_traits::float::FloatConst;
use rand::Rng;

use vlife_physics::{Object, ObjectId, Scalar, Vec2};

use crate::{neurons::Neurons, simulator::SimulationContext, M, V};

pub const NUM_MOLECULES: usize = 8;

pub const MAX_ENERGY: Scalar = 100.0;
pub const MAX_MOLECULE_AMOUNT: Scalar = 100.0;
pub const MAX_MOVEMENT_COST: Scalar = 0.0005;
pub const MAX_CONTRACTION_COST: Scalar = 0.001;
pub const MAX_CONTRACTION: Scalar = 0.8;
pub const MAX_CONTACT_ENERGY_ABSORPTION: Scalar = 1.75;
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

    /// The size of the cell when there is no contraction.
    pub(crate) size: Scalar,
    /// The amount of energy in the cell. Energy can be generated from internal molecules
    /// as well as absorbed from other cells.
    pub(crate) energy: Scalar,
    /// Last energy level used to calculate the energy delta of an step. Source: Processing.
    pub(crate) last_energy: Scalar,

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
}

impl Cell {
    pub fn random(object_id: ObjectId, size: Scalar) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            object_id,
            neurons: Neurons::random(),
            size,
            energy: MAX_ENERGY,
            last_energy: MAX_ENERGY,
            molecules: V::from_fn(|_, _| rng.gen_range(0.0..=MAX_MOLECULE_AMOUNT)),
            molecules_energy_conversion: V::from_fn(|_, _| rng.gen_range(0.0..10.0)),
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
        }
    }

    pub fn energy(&self) -> Scalar {
        self.energy
    }

    pub fn energy_delta(&self) -> Scalar {
        self.energy - self.last_energy
    }

    pub fn size(&self) -> Scalar {
        self.size
    }

    pub fn contracted_size(&self) -> Scalar {
        (self.size * (1.0 - self.contraction_amount)).max(1.0)
    }

    pub fn movement_velocity(&self) -> Vec2 {
        self.movement_velocity
    }

    pub fn energy_absorption_from(&self, other: &Cell, dt: Scalar) -> Scalar {
        fn resistance(absorption: Scalar) -> Scalar {
            (1.0 + absorption).recip()
        }
        // println!(
        //     "a: {:.4} eo: {:.4} r: {:.4} dt: {:.4}",
        //     self.contact_energy_absorption_amount,
        //     other.energy,
        //     resistance(other.contact_energy_absorption_amount),
        //     dt
        // );
        self.contact_energy_absorption_amount
            * other.energy
            * resistance(other.contact_energy_absorption_amount)
            * dt
    }

    pub fn update(&mut self, dt: Scalar, context: SimulationContext<'_>) {
        self.process_neurons(dt, &context);
        self.last_energy = self.energy;
        self.compute_contraction();
        self.compute_movement(dt);
        self.compute_metabolism(dt /*context.reactions*/);
        self.compute_contact_energy_absorption();
        self.compute_produced_energy(dt);
        self.compute_consumed_energy(dt, context.object);
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

        self.neurons.set_molecules_amount(&self.molecules);
        self.neurons.set_molecules_total(self.molecules.sum());
        self.neurons.set_movement_direction(self.movement_direction);
        self.neurons.set_movement_speed(self.movement_speed);
        self.neurons.set_energy_amount(self.energy);
        self.neurons
            .set_energy_delta(self.energy - self.last_energy);

        self.neurons.process();
    }

    fn compute_contraction(&mut self) {
        self.contraction_amount = self.neurons.contraction_out().max(0.0) * self.contraction_limit;
    }

    fn compute_movement(&mut self, _dt: Scalar) {
        let two_pi = 2.0 * Scalar::PI();
        let direction = self.neurons.movement_direction_out() * two_pi;
        // let direction = self.movement_direction + angular_speed * dt;
        // self.movement_direction = direction % two_pi;
        self.movement_direction = direction.abs() % two_pi;
        let rotation = UnitComplex::new(-self.movement_direction);

        self.movement_speed =
            self.neurons.movement_speed_out().max(0.0) * self.movement_speed_limit;

        self.movement_velocity = rotation * Vec2::x_axis().scale(self.movement_speed);
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

    fn compute_contact_energy_absorption(&mut self) {
        let amount =
          self.contact_energy_absorption_amount + self.neurons.contact_energy_absorption_out();
        self.contact_energy_absorption_amount =
          amount.clamp(0.0, self.contact_energy_absorption_limit);
    }

    fn compute_produced_energy(&mut self, dt: Scalar) {
        let zeros = V::<NUM_MOLECULES>::from_element(0.0);
        let energy_source = V::<NUM_MOLECULES>::from(self.neurons.energy_source_out()).sup(&zeros);
        let mut energy_source = (energy_source.component_mul(&self.molecules_energy_conversion)
            * dt)
            .inf(&self.molecules);
        // println!(">>> =================");
        // println!("energy_source: {:.2}", energy_source.transpose());
        let mut produced_energy = energy_source.sum();
        // println!("produced energy: {:.2}", produced_energy);
        if self.energy + produced_energy > MAX_ENERGY {
            let factor = (MAX_ENERGY - self.energy) / produced_energy;
            // println!("factor: {:.2}", factor);
            energy_source *= factor;
            produced_energy *= factor;
        }
        // println!("{:.2}", energy_source);
        if energy_source.fold(true, |positive, x| positive && x >= 0.0) {
            debug_assert!(energy_source.fold(true, |positive, x| positive && x >= 0.0));
            debug_assert!(produced_energy >= 0.0);
            self.molecules -= energy_source;
            self.energy += produced_energy;
        }
    }

    fn compute_consumed_energy(&mut self, dt: Scalar, object: &Object) {
        let velocity = self.movement_velocity.norm();
        let kinetic_energy = 0.5 * object.mass() * velocity * velocity;
        let kinetic_energy = kinetic_energy * self.movement_cost;

        let contraction_energy = self.contraction_amount * self.contraction_cost;

        self.energy -= (kinetic_energy + contraction_energy) * dt;
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Energy: Amount: {:.2} Delta: {:.6}", self.energy, self.energy_delta())?;
        writeln!(
            f,
            "Molecules: {:.2?}, Total: {:.2?}",
            self.molecules,
            self.molecules.sum()
        )?;
        writeln!(
            f,
            "Molecules: Production: {:.2?}",
            self.molecules_energy_conversion
        )?;

        writeln!(
            f,
            "Movement: Speed: {:.2} / {:.2}, Direction: {:3.0} deg, Cost: {:.6}",
            self.movement_speed,
            self.movement_speed_limit,
            self.movement_direction * 360.0 / (2.0 * Scalar::PI()),
            self.movement_cost
        )?;
        let contracted_size = self.contracted_size();
        writeln!(
            f,
            "Contraction: Size: {:.1} / {:.1} ({:5.1}%), Amount: {:.3} / {:.3}, Cost: {:.6}",
            contracted_size,
            self.size,
            (self.size - contracted_size) * 100.0 / self.size,
            self.contraction_amount,
            self.contraction_limit,
            self.contraction_cost
        )?;
        writeln!(
            f,
            "Contact: Energy Abs: {:.4} / {:.4}",
            self.contact_energy_absorption_amount, self.contact_energy_absorption_limit,
        )?;
        write!(f, "{}", self.neurons)?;
        Ok(())
    }
}
