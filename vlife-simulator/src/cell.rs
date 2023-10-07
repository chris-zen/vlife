use rand::Rng;

use crate::neurons::Neurons;
use crate::real::Real;
use crate::real::RealConst;

pub const NUM_MOLECULES: usize = 8;

pub const MAX_RADIUS: Real = 10.0;
pub const MAX_PERIMETER: Real = Real::TWO_PI * MAX_RADIUS;

/// Model for a cell.
pub struct Cell {
    /// Age.
    pub(crate) age: Real,

    /// The amount of energy in the cell. Energy can be generated from internal molecules
    /// as well as absorbed from the environment or other cells.
    pub(crate) energy: Real,

    /// The amount of membrane components. The bigger the membrane, the bigger the cytoplasm for the cell.
    pub(crate) membrane: Real,

    /// Neuronal network.
    /// This is used to model complex behaviour based on external/internal signals.
    /// External signals are related to perception, while internal ones allow for feedback loops.
    /// The outputs can be used to represent impulses, like the contraction. But they are also
    /// used to model the regulation of the genome expression, like the
    /// `contact_energy_absorption_amount` which represents the amount of membrane channels
    /// used to absorb energy from other cells.
    pub(crate) neurons: Neurons,
}

impl Cell {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            age: 0.0,
            energy: 1.0,
            membrane: rng.gen_range(0.1..=1.0),
            neurons: Neurons::random(),
        }
    }

    pub fn radius(&self) -> Real {
        self.membrane * MAX_RADIUS
    }

    pub fn update(&mut self, dt: Real) {
        self.age += dt;
    }

    fn process_neurons(&mut self, _dt: Real, energy_delta: Real) {
        // TODO sensors
        self.neurons.process();
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let total_energy = self.energy + self.stored_energy;
        // let energy_delta = self.energy_delta();
        // let days = (self.age * (1.0 / 86400.0)).floor();
        // let hours = (self.age * (1.0 / 3600.0)).floor();
        // let minutes = (self.age * (1.0 / 60.0)).floor();
        // let seconds = self.age % 60.0;
        // writeln!(
        //     f,
        //     "Age> Days: {:.0}, Time: {:02.0}:{:02.0}:{:04.1}",
        //     days, hours, minutes, seconds
        // )?;
        // writeln!(
        //     f,
        //     "Energy> Available: {:6.2} ({:3.0} %), Stored: {:6.2} ({:3.0} %), Delta: {:7.4}, Basal: {:7.4} Zero: {:5.1} / {:5.1}",
        //     self.energy,
        //     self.energy * 100.0 / total_energy,
        //     self.stored_energy,
        //     self.stored_energy * 100.0 / total_energy,
        //     energy_delta,
        //     self.basal_energy(),
        //     self.zero_energy_time,
        //     self.zero_energy_limit,
        // )?;
        // writeln!(
        //     f,
        //     "Division> Reserve: {:6.2} ({:3.0} %), Threshold: {:6.2}, Signal: {:4.2}",
        //     self.division_energy_reserve,
        //     self.division_energy_reserve * 100.0 / self.division_threshold,
        //     self.division_threshold,
        //     self.neurons.get_division_energy_reserve()
        // )?;
        // writeln!(
        //     f,
        //     "Molecules>  {:5.1?}, Total: {:.1?}",
        //     self.molecules,
        //     self.molecules.sum()
        // )?;
        // writeln!(f, "Conversion: {:5.1?}", self.molecules_energy_conversion)?;
        // writeln!(
        //     f,
        //     "Regulation:  {:5.1?}",
        //     self.neurons.get_energy_metabolism().as_slice(),
        // )?;
        // writeln!(
        //     f,
        //     "Contact> Count: {:2.0}, Energy Absorption: {:3.0} % ({:.4} / {:.4}), Permeability: {:3.1}, Diffusion: {:3.1}",
        //     self.contact_count,
        //     self.contact_energy_absorption_amount * 100.0 / self.contact_energy_absorption_limit,
        //     self.contact_energy_absorption_amount,
        //     self.contact_energy_absorption_limit,
        //     self.energy_permeability(),
        //     self.energy_diffusion(),
        // )?;
        // writeln!(
        //     f,
        //     "Movement> Speed: {:3.0} % ({:6.2} / {:6.2}), Dir: {:3.0}",
        //     self.movement_speed * 100.0 / self.movement_speed_limit,
        //     self.movement_speed,
        //     self.movement_speed_limit,
        //     self.movement_direction * 360.0 / (2.0 * Scalar::PI()),
        // )?;
        // let contracted_size = self.contracted_size();
        // writeln!(
        //     f,
        //     "Contraction> Size: {:3.0} % ({:5.1} / {:5.1}), Amount: {:6.2} / {:6.2}",
        //     100.0 - (contracted_size * 100.0 / self.size),
        //     contracted_size,
        //     self.size,
        //     self.contraction_amount,
        //     self.contraction_limit,
        // )?;
        // let energy_positive = self.stats.energy_produced + self.stats.energy_absorbed_in;
        // let energy_negative = self.stats.energy_consumed + self.stats.energy_absorbed_out;
        // writeln!(
        //     f,
        //     "Stats> Energy Consumed: {:5.1}, Produced: {:5.1}, Absorbed Out: {:5.1}, Absorbed In: {:5.1}, Net: {:5.1}, Ratio: {:6.3}",
        //     self.stats.energy_consumed,
        //     self.stats.energy_produced,
        //     self.stats.energy_absorbed_out,
        //     self.stats.energy_absorbed_in,
        //     energy_positive - energy_negative,
        //     (1.0 + energy_positive) / (1.0 + energy_negative),
        // )?;
        // write!(f, "{}", self.neurons)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CellStats {
    pub energy_consumed: Real,
    pub energy_produced: Real,
    pub energy_absorbed_out: Real,
    pub energy_absorbed_in: Real,
}

impl CellStats {
    fn update_energy_consumed(&mut self, amount: Real) {
        self.energy_consumed += amount;
    }

    fn update_energy_produced(&mut self, amount: Real) {
        self.energy_produced += amount;
    }

    fn update_energy_absorbed_out(&mut self, amount: Real) {
        self.energy_absorbed_out += amount;
    }

    fn update_energy_absorbed_in(&mut self, amount: Real) {
        self.energy_absorbed_in += amount;
    }
}
