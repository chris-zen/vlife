use nalgebra::Point2;
use rand::{random, Rng};

use crate::physics::geometry::bounding_box::AxisAlignedBoundingBox;
use crate::Real;

pub struct ClosedPolygon {
    segments: Vec<SegmentPoint>,
    bounding_box: AxisAlignedBoundingBox,
}

impl ClosedPolygon {
    pub fn new(points: Vec<Point2<Real>>) -> Self {
        let mut polygon = Self {
            segments: Vec::with_capacity(points.len()),
            bounding_box: AxisAlignedBoundingBox::empty(),
        };
        polygon.update(points);
        polygon
    }

    pub fn empty() -> Self {
        Self {
            segments: Vec::new(),
            bounding_box: AxisAlignedBoundingBox::empty(),
        }
    }

    pub fn bounding_box(&self) -> &AxisAlignedBoundingBox {
        &self.bounding_box
    }

    pub fn points(&self) -> impl Iterator<Item = Point2<Real>> + '_ {
        self.segments.iter().map(|segment| segment.point)
    }

    pub fn update<P>(&mut self, points: P)
    where
        P: IntoIterator<Item = Point2<Real>>,
    {
        self.segments.clear();
        let mut bounding_box = AxisAlignedBoundingBox::builder();
        let mut points = points.into_iter().map(|point: Point2<Real>| point);
        if let Some(first_point) = points.next() {
            bounding_box.add_point(first_point);
            let mut prev_point = first_point;
            for point in points {
                bounding_box.add_point(point);
                let inv_length = (point - prev_point).magnitude().recip();
                self.segments.push(SegmentPoint {
                    point: prev_point,
                    inv_length,
                });
                prev_point = point;
            }
            let inv_length = (first_point - prev_point).magnitude().recip();
            self.segments.push(SegmentPoint {
                point: prev_point,
                inv_length,
            });
        }
        self.bounding_box = bounding_box.build();
    }

    pub fn has_point_inside(&self, point: Point2<Real>) -> bool {
        let mut count = 0;
        let len = self.segments.len();
        for i in 0..len {
            let a = self.segments[i].point;
            let b = self.segments[(i + 1) % len].point;
            if (point.y < a.y) != (point.y < b.y)
                && point.x < a.x + ((point.y - a.y) / (b.y - a.y)) * (b.x - a.x)
            {
                count += 1;
            }
        }
        count % 2 == 1
    }

    pub fn closest_segment_within_bounding_box(
        &self,
        point: Point2<Real>,
        bounding_box: &AxisAlignedBoundingBox,
    ) -> Option<ClosestSegment> {
        let mut maybe_closest_segment = None;
        let len = self.segments.len();
        for index1 in 0..len {
            let index2 = (index1 + 1) % len;
            let segment_point1 = &self.segments[index1];
            let segment_point2 = &self.segments[index2];
            let point1 = segment_point1.point;
            let point2 = segment_point2.point;
            if bounding_box.contains_point(point1) || bounding_box.contains_point(point2) {
                let inv_length = segment_point1.inv_length;
                let maybe_candidate_segment = Self::distance_to_segment(
                    point, point1, point2, inv_length,
                )
                .map(|(distance, ratio)| ClosestSegment {
                    index1,
                    index2,
                    point1,
                    point2,
                    depth: distance,
                    ratio,
                });
                match (maybe_closest_segment.as_ref(), maybe_candidate_segment) {
                    (None, Some(candidate_segment)) => {
                        maybe_closest_segment = Some(candidate_segment)
                    }
                    (Some(closest_segment), Some(candidate_segment)) => {
                        if candidate_segment.depth < closest_segment.depth {
                            maybe_closest_segment = Some(candidate_segment);
                        } else if candidate_segment.depth == closest_segment.depth {
                            if random() {
                                maybe_closest_segment = Some(candidate_segment);
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
        maybe_closest_segment
    }

    /// From https://en.wikipedia.org/wiki/Distance_from_a_point_to_a_line#Line_defined_by_two_points
    fn distance_to_segment(
        point: Point2<Real>,
        a: Point2<Real>,
        b: Point2<Real>,
        inv_length: Real,
    ) -> Option<(Real, Real)> {
        let ab = b - a;
        let ap = point - a;
        let projection_ratio = ap.dot(&ab) * inv_length * inv_length;
        if projection_ratio >= 0.0 && projection_ratio <= 1.0 {
            let distance = (ab.x * (a.y - point.y) - (a.x - point.x) * ab.y) * inv_length;
            Some((distance, projection_ratio))
        } else {
            None
        }
    }
}

pub struct ClosestSegment {
    pub index1: usize,
    pub index2: usize,
    pub point1: Point2<Real>,
    pub point2: Point2<Real>,
    pub depth: Real,
    pub ratio: Real,
}

struct SegmentPoint {
    pub(crate) point: Point2<Real>,
    pub(crate) inv_length: Real,
}
