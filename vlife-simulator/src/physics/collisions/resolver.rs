use crate::physics::collisions::collision::{Collision, PointInPolygon};
use crate::physics::collisions::CollisionsContext;
use crate::Vec2;

pub struct CollisionResolver {}

impl CollisionResolver {
    pub fn new() -> Self {
        Self {}
    }

    pub(crate) fn resolve<'a, C>(&self, collision: C, context: &mut CollisionsContext<'a>)
    where
        C: Into<Collision>,
    {
        match collision.into() {
            Collision::PointInPolygon(collision) => {
                self.resolve_point_in_polygon(collision, context)
            }
        }
    }

    fn resolve_point_in_polygon<'a>(
        &self,
        collision: PointInPolygon,
        context: &mut CollisionsContext<'a>,
    ) {
        let PointInPolygon {
            particle_handle,
            particle_point,
            segment_handle1,
            segment_handle2,
            segment_point1,
            segment_point2,
            ratio,
            depth,
        } = collision;

        println!("=================================");
        println!("({particle_point:.2?}), ([{segment_point1:.2?}] -- [{segment_point2:.2?}])");

        let inv_mass0 = context
            .particles
            .get(particle_handle)
            .map(|particle| particle.inv_mass())
            .unwrap_or_default();
        let inv_mass1 = context
            .particles
            .get(segment_handle1)
            .map(|particle| particle.inv_mass())
            .unwrap_or_default();
        let inv_mass2 = context
            .particles
            .get(segment_handle2)
            .map(|particle| particle.inv_mass())
            .unwrap_or_default();

        let normal = (segment_point2 - segment_point1)
            .normalize()
            .component_mul(&Vec2::new(-1.0, 1.0));

        let total_inv_mass = inv_mass0 + inv_mass1 + inv_mass2;
        let particle_depth = depth * inv_mass0 / total_inv_mass;
        let segment_depth = depth * (inv_mass1 + inv_mass2) / total_inv_mass;

        let particle_point = particle_point + particle_depth * normal;

        let inv_ratio = 1.0 - ratio;
        let q = ratio * segment_point1.coords + inv_ratio * segment_point2.coords;
        let qp = q - particle_point.coords;
        let lambda = 0.01; //(particle_point.coords - q).dot(&qp) / (ratio * ratio + inv_ratio * inv_ratio) * qp.magnitude();
        let segment_point1 = segment_point1.coords; // - ratio * lambda * segment_depth * normal;
        let segment_point2 = segment_point2.coords; // - inv_ratio * lambda * segment_depth * normal;

        if let Some(p0) = context.particles.get_mut(particle_handle) {
            p0.position = particle_point.coords;
        }
        if let Some((p1, p2)) = context.particles.get_pair_mut(segment_handle1, segment_handle2) {
            // p1.position = segment_point1;
            // p2.position = segment_point2;
        }

        println!("({particle_point:.2?}), ({segment_point1:.2?} -- {segment_point2:.2?}), {lambda}");
    }
}
