use rand::{seq::SliceRandom, Rng};

use vlife_physics::Scalar;

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

const NUM_METABOLIC_OUTPUTS: usize = NUM_MOLECULES * NUM_MOLECULES;
const NUM_ENERGY_OUTPUTS: usize = NUM_MOLECULES;
const NUM_CONTRACTION_OUTPUTS: usize = 1;
const NUM_MOVEMENT_OUTPUTS: usize = 2;
const NUM_CONTACT_OUTPUTS: usize = 1;

const NUM_PROCESSING: usize = NUM_INPUTS / 2;
const NUM_OUTPUTS: usize = NUM_METABOLIC_OUTPUTS
    + NUM_ENERGY_OUTPUTS
    + NUM_CONTRACTION_OUTPUTS
    + NUM_MOVEMENT_OUTPUTS
    + NUM_CONTACT_OUTPUTS;

pub struct Neurons {
    inputs: V<NUM_INPUTS>,
    layer1: Layer<NUM_INPUTS, NUM_PROCESSING>,
    layer2: Layer<NUM_PROCESSING, NUM_OUTPUTS>,
    working_neurons: Scalar,
}

impl Neurons {
    pub fn random() -> Self {
        let mut layer1 = Layer::random();
        layer1.activation = ActivationFunction::Sigmoid;
        let mut layer2 = Layer::random();
        layer2.activation = ActivationFunction::Tanh;
        let working_neurons = layer1.num_working_neurons() + layer2.num_working_neurons();
        Self {
            inputs: V::zeros(),
            layer1,
            layer2,
            working_neurons,
        }
    }

    pub fn num_working_neurons(&self) -> Scalar {
        self.working_neurons
    }

    pub fn process(&mut self) {
        // println!("IN: {:.2}", self.inputs.transpose());
        self.layer1.process(&self.inputs);
        // println!("HI: {:.2}", self.layer1.outputs().transpose());
        self.layer2.process(self.layer1.outputs());
        // println!("OUT: {:.2}", self.layer2.outputs().transpose());
    }

    fn outputs_slice<const N: usize>(&self, start: usize) -> VView<'_, N, NUM_OUTPUTS> {
        let outputs = self.layer2.outputs();
        outputs.fixed_rows::<N>(start)
    }

    fn outputs(&self) -> &V<NUM_OUTPUTS> {
        self.layer2.outputs()
    }

    pub fn metabolism_factors_out(&self) -> VView<'_, NUM_METABOLIC_OUTPUTS, NUM_OUTPUTS> {
        self.outputs_slice::<NUM_METABOLIC_OUTPUTS>(0)
    }

    pub fn energy_source_out(&self) -> VView<'_, NUM_ENERGY_OUTPUTS, NUM_OUTPUTS> {
        self.outputs_slice::<NUM_ENERGY_OUTPUTS>(NUM_METABOLIC_OUTPUTS)
    }

    pub fn contraction_out(&self) -> Scalar {
        self.outputs()[NUM_METABOLIC_OUTPUTS + NUM_ENERGY_OUTPUTS]
    }

    pub fn movement_direction_out(&self) -> Scalar {
        self.outputs()[NUM_METABOLIC_OUTPUTS + NUM_ENERGY_OUTPUTS + NUM_CONTRACTION_OUTPUTS]
    }

    pub fn movement_speed_out(&self) -> Scalar {
        self.outputs()[NUM_METABOLIC_OUTPUTS + NUM_ENERGY_OUTPUTS + NUM_CONTRACTION_OUTPUTS + 1]
    }

    pub fn contact_energy_absorption_out(&self) -> Scalar {
        self.outputs()[NUM_METABOLIC_OUTPUTS
            + NUM_ENERGY_OUTPUTS
            + NUM_CONTRACTION_OUTPUTS
            + NUM_MOVEMENT_OUTPUTS]
    }
}

// This will generate all the setters for the neuronal network inputs
define_inputs!(
    (velocity_pos, 2),
    velocity_magnitude,
    (acceleration_pos, 2),
    acceleration_magnitude,
    radius,
    energy_amount,
    energy_delta,
    energy_stored,
    (molecules_amount, NUM_MOLECULES),
    molecules_total,
    movement_direction,
    movement_speed,
    contact_energy_absorption,
    contact_count,
    (contact_normal, 2),
);

impl std::fmt::Display for Neurons {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Working neurons: {:.0?}", self.working_neurons)?;
        writeln!(f, "I1: {:.2?}", self.inputs)?;
        writeln!(f, "O1: {:.2?}", self.layer1.outputs)?;
        writeln!(f, "W1: {:.2?}", self.layer1.weights)?;
        writeln!(f, "B1: {:.2?}", self.layer1.bias)?;
        writeln!(f, "O2: {:.2?}", self.layer2.outputs)?;
        writeln!(
            f,
            "activations: {:?}",
            [self.layer1.activation, self.layer2.activation]
        )?;
        // writeln!(
        //     f,
        //     "metabolism_factors: {:.2?}",
        //     self.metabolism_factors_out().clone_owned()
        // )?;
        writeln!(
            f,
            "energy_source: {:.2?}",
            self.energy_source_out().clone_owned()
        )?;
        writeln!(
            f,
            "movement_angular_speed: {:.2?}",
            self.movement_direction_out()
        )?;
        writeln!(f, "movement_speed: {:.2?}", self.movement_speed_out())?;
        writeln!(f, "contraction: {:.2?}", self.contraction_out())?;
        Ok(())
    }
}

pub struct Layer<const I: usize, const O: usize> {
    /// Every row contains the weights for a given neuron.
    weights: M<O, I>,
    bias: V<O>,
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
