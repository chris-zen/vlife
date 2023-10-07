use num_traits::Float;

pub trait RealConst: Float {
    const PI: Self;
    const TWO_PI: Self;
}

impl RealConst for f32 {
    const PI: Self = std::f32::consts::PI;
    const TWO_PI: Self = 2.0 * Self::PI;
}

impl RealConst for f64 {
    const PI: Self = std::f64::consts::PI;
    const TWO_PI: Self = 2.0 * Self::PI;
}

pub type Real = f64;
