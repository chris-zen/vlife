use rand::{seq::SliceRandom, Rng};
use vlife_macros::BuildGenome;

use crate::genome::{BuildGenome, Gen, GenomeBuilder};
use crate::Scalar;
use crate::{cell::NUM_MOLECULES, VView, M, V};

macro_rules! define_inputs {
    ( $name:ident $(,)?) => {
        define_inputs!(@next 0, [$name]);

    };

    ( $name:ident, $($args:tt),* $(,)?) => {
        define_inputs!(@next 0, [$name, $($args),*]);
    };

    ( ($name:ident, $len:expr) $(,)?) => {
        define_inputs!(@next 0, [($name, $len)]);

    };

    ( ($name:ident, $len:expr), $($args:tt),* $(,)?) => {
        define_inputs!(@next 0, [($name, $len), $($args),*]);
    };

    (@next $start:expr, [$name:ident $(,)?]) => {
        define_inputs!(@scalar $name, $start);
        const NUM_INPUTS: usize = $start + 1;
    };

    (@next $start:expr, [$name:ident, $($args:tt),* $(,)?]) => {
        define_inputs!(@scalar $name, $start);
        define_inputs!(@next $start + 1, [$($args),*]);
    };

    (@next $start:expr, [($name:ident, $len:expr) $(,)?]) => {
        define_inputs!(@vector $name, $start, $len);
        const NUM_INPUTS: usize = $start + $len;
    };

    (@next $start:expr, [($name:ident, $len:expr), $($args:tt),* $(,)?]) => {
        define_inputs!(@vector $name, $start, $len);
        define_inputs!(@next $start + $len, [$($args),*]);
    };

    (@scalar $name:ident, $start:expr) => {
        paste::paste! {
            impl Neurons {
                pub fn [<set_ $name>](&mut self, value: Scalar) {
                    self.inputs[$start] = value;
                }
            }
        }
    };

    (@vector $name:ident, $start:expr, $len:expr) => {
        paste::paste! {
            impl Neurons {
                pub fn [<set_ $name>](&mut self, value: &V<$len>) {
                    self.inputs.fixed_rows_mut::<{ $len }>($start).set_column(0, value);
                }
            }
        }
    };
}

macro_rules! define_outputs {
    ( $name:ident $(,)?) => {
        define_outputs!(@next 0, [$name]);

    };

    ( $name:ident, $($args:tt),* $(,)?) => {
        define_outputs!(@next 0, [$name, $($args),*]);
    };

    ( ($name:ident, $len:expr) $(,)?) => {
        define_outputs!(@next 0, [($name, $len)]);

    };

    ( ($name:ident, $len:expr), $($args:tt),* $(,)?) => {
        define_outputs!(@next 0, [($name, $len), $($args),*]);
    };

    (@next $start:expr, [$name:ident $(,)?]) => {
        define_outputs!(@scalar $name, $start);
        const NUM_OUTPUTS: usize = $start + 1;
    };

    (@next $start:expr, [$name:ident, $($args:tt),* $(,)?]) => {
        define_outputs!(@scalar $name, $start);
        define_outputs!(@next $start + 1, [$($args),*]);
    };

    (@next $start:expr, [($name:ident, $len:expr) $(,)?]) => {
        define_outputs!(@vector $name, $start, $len);
        const NUM_OUTPUTS: usize = $start + $len;
    };

    (@next $start:expr, [($name:ident, $len:expr), $($args:tt),* $(,)?]) => {
        define_outputs!(@vector $name, $start, $len);
        define_outputs!(@next $start + $len, [$($args),*]);
    };

    (@scalar $name:ident, $start:expr) => {
        paste::paste! {
            impl Neurons {
                pub fn [<get_ $name>](&self) -> Scalar {
                    self.output_layer.outputs()[$start]
                }
            }
        }
    };

    (@vector $name:ident, $start:expr, $len:expr) => {
        paste::paste! {
            impl Neurons {
                pub fn [<get_ $name>](&self) -> VView<'_, $len, NUM_OUTPUTS> {
                    let outputs = self.output_layer.outputs();
                    outputs.fixed_rows::<$len>($start)
                }
            }
        }
    };
}

const NUM_PROCESSING: usize = NUM_INPUTS / 2;

#[derive(Clone, BuildGenome)]
pub struct Neurons {
    inputs: V<NUM_INPUTS>,
    #[build_genome(nested)]
    input_layer: Layer<NUM_INPUTS, NUM_PROCESSING>,
    #[build_genome(nested)]
    processing_layer: Layer<NUM_PROCESSING, NUM_PROCESSING>,
    #[build_genome(nested)]
    output_layer: Layer<NUM_PROCESSING, NUM_OUTPUTS>,
    working_neurons: Scalar,
}

impl Neurons {
    pub fn random() -> Self {
        let mut input_layer = Layer::random();
        input_layer.activation = ActivationFunction::Sigmoid;
        let mut processing_layer = Layer::random();
        processing_layer.activation = ActivationFunction::Tanh;
        let mut output_layer = Layer::random();
        output_layer.activation = ActivationFunction::Tanh;
        let working_neurons = input_layer.num_working_neurons()
            + processing_layer.num_working_neurons()
            + output_layer.num_working_neurons();
        Self {
            inputs: V::zeros(),
            input_layer,
            processing_layer,
            output_layer,
            working_neurons,
        }
    }

    pub fn num_working_neurons(&self) -> Scalar {
        self.working_neurons
    }

    pub fn process(&mut self) {
        // println!("IN: {:.2}", self.inputs.transpose());
        self.input_layer.process(&self.inputs);
        // println!("INL: {:.2}", self.input_layer.outputs().transpose());
        self.processing_layer.process(self.input_layer.outputs());
        // println!("HIL: {:.2}", self.processing_layer.outputs().transpose());
        self.output_layer.process(self.processing_layer.outputs());
        // println!("OUL: {:.2}", self.output_layer.outputs().transpose());
    }
}

// This will generate all the setters for the neuronal network inputs
// (velocity_pos, 2),
// (acceleration_pos, 2),
define_inputs!(
    velocity_magnitude,
    acceleration_magnitude,
    radius,
    age,
    energy_amount,
    energy_stored,
    energy_delta,
    zero_energy,
    division_energy_reserve,
    division_grow_factor,
    (molecules_proportion, NUM_MOLECULES),
    molecules_total,
    movement_direction,
    movement_speed,
    (movement_velocity, 2),
    movement_velocity_magnitude,
    contact_energy_absorption,
    contact_count,
    (contact_normal, 2),
    contact_normal_magnitude,
);

define_outputs!(
    (energy_metabolism, NUM_MOLECULES),
    division_energy_reserve,
    contraction_amount,
    movement_angular_speed,
    movement_kinetic_speed,
    contact_energy_absorption,
);

impl std::fmt::Display for Neurons {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Working neurons: {:.0?}", self.working_neurons)?;
        writeln!(f, "I1: {:6.2?}", self.inputs)?;
        // writeln!(f, "W1: {:.2?}", self.input_layer.weights)?;
        writeln!(f, "B1: {:6.2?}", self.input_layer.bias)?;
        writeln!(f, "O1: {:6.2?}", self.input_layer.outputs)?;
        writeln!(f, "O2: {:.2?}", self.processing_layer.outputs)?;
        writeln!(f, "O3: {:.2?}", self.output_layer.outputs)?;
        // writeln!(
        //     f,
        //     "activations: {:?}",
        //     [self.input_layer.activation, self.output_layer.activation]
        // )?;
        writeln!(
            f,
            "energy_metabolism: {:.2?}",
            self.get_energy_metabolism().clone_owned()
        )?;
        writeln!(f, "contraction: {:.2?}", self.get_contraction_amount())?;
        writeln!(
            f,
            "movement_angular_speed: {:.2?}",
            self.get_movement_angular_speed()
        )?;
        writeln!(
            f,
            "movement_kinetic_speed: {:.2?}",
            self.get_movement_kinetic_speed()
        )?;
        writeln!(
            f,
            "contact_energy_absorption: {:.2?}",
            self.get_contact_energy_absorption()
        )?;
        Ok(())
    }
}

#[derive(Clone, BuildGenome)]
pub struct Layer<const I: usize, const O: usize> {
    /// Every row contains the weights for a given neuron.
    #[build_genome(nested)]
    weights: M<O, I>,
    #[build_genome(nested)]
    bias: V<O>,
    #[build_genome(nested)]
    activation: ActivationFunction,
    outputs: V<O>,
}

impl<const I: usize, const O: usize> Layer<I, O> {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            weights: M::from_fn(|_, _| rng.gen_range(-1.0..1.0)),
            bias: V::from_fn(|_, _| rng.gen_range(-1.0..1.0)),
            activation: ActivationFunction::random(),
            outputs: V::zeros(),
        }
    }

    pub fn process(&mut self, input: &V<I>) {
        let y = self.weights * input + self.bias;
        self.outputs = self.activation.process(y);
    }

    pub fn outputs(&self) -> &V<O> {
        &self.outputs
    }

    fn num_working_neurons(&self) -> Scalar {
        let active = self
            .weights
            .apply_into(|x| *x = if x.abs() > 0.0 { 1.0 } else { 0.0 });
        active
            .row_sum()
            .apply(|x| *x = if x.abs() > 0.0 { 1.0 } else { 0.0 });
        active.sum()
    }
}

#[derive(Clone, Copy)]
pub enum ActivationFunction {
    Linear,
    Sigmoid,
    Tanh,
    Relu,
    Swish,
}

impl ActivationFunction {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let choices = [
            Self::Linear,
            Self::Sigmoid,
            // Self::Tanh,
            Self::Relu,
            Self::Swish,
        ];
        choices.choose(&mut rng).unwrap().clone()
    }

    pub fn process<const N: usize>(&self, input: V<N>) -> V<N> {
        match self {
            Self::Linear => input,
            Self::Sigmoid => input.apply_into(|x| *x = 1.0 / (1.0 + (-*x).exp())),
            Self::Tanh => input.apply_into(|x| {
                let a = x.exp();
                let b = (-*x).exp();
                *x = (a - b) / (a + b);
            }),
            Self::Relu => input.apply_into(|x| *x = x.max(0.0)),
            Self::Swish => input.apply_into(|x| *x = *x / (1.0 + (-*x).exp())),
        }
    }
}

impl BuildGenome for ActivationFunction {
    fn build_genome<'a>(&self, builder: GenomeBuilder) {
        let value = match self {
            ActivationFunction::Linear => 1.0,
            ActivationFunction::Sigmoid => 2.0,
            ActivationFunction::Tanh => 3.0,
            ActivationFunction::Relu => 4.0,
            ActivationFunction::Swish => 5.0,
        };
        builder.add("activation_function", Gen { value: value });
    }
}

impl std::fmt::Debug for ActivationFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Linear => "linear",
            Self::Sigmoid => "sigmoid",
            Self::Tanh => "tanh",
            Self::Relu => "relu",
            Self::Swish => "swish",
        };
        f.write_str(name)
    }
}
