use nalgebra::Point2;

use crate::Real;

#[derive(Debug, Clone, Copy)]
pub struct AxisAlignedBoundingBox {
    center: Point2<Real>,
    size: Point2<Real>,
}

impl AxisAlignedBoundingBox {
    pub fn new(size: Point2<Real>, center: Point2<Real>) -> Self {
        Self { size, center }
    }

    pub fn from_min_max(min: Point2<Real>, max: Point2<Real>) -> Self {
        let center = Point2::new(0.5 * (min.x + max.x), 0.5 * (min.y + max.y));
        let size = Point2::new(max.x - min.x, max.y - min.y);
        Self { center, size }
    }

    pub fn builder() -> AxisAlignedBoundingBoxBuilder {
        AxisAlignedBoundingBoxBuilder::new()
    }

    pub fn intersects(&self, other: &AxisAlignedBoundingBox) -> bool {
        let two_times_distance = (other.center - self.center).abs() * 2.0;
        let total_size = other.size.coords + self.size.coords;
        two_times_distance < total_size
    }
}

pub struct AxisAlignedBoundingBoxBuilder {
    min: Point2<Real>,
    max: Point2<Real>,
}

impl AxisAlignedBoundingBoxBuilder {
    pub fn new() -> Self {
        Self {
            min: Point2::origin(),
            max: Point2::origin(),
        }
    }

    pub fn add_point(&mut self, point: Point2<Real>) {
        self.min = self.min.inf(&point);
        self.max = self.max.sup(&point);
    }

    pub fn build(&self) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox::from_min_max(self.min, self.max)
    }
}
