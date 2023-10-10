use nalgebra::{Point2, SimdComplexField};

use crate::object_set::ObjectSet;
use crate::physics::collisions::collision::PointInPolygon;
use crate::physics::collisions::resolver::CollisionResolver;
use crate::physics::collisions::CollisionsContext;
use crate::physics::{
    engine::DEFAULT_RESTITUTION, geometry::polygon::ClosedPolygon, Particle, ParticleHandle,
};
use crate::{Real, Vec2};

pub struct PolygonCollider {
    particle_handles: Vec<ParticleHandle>,
    restitution: Real,
    polygon: ClosedPolygon,
}

impl PolygonCollider {
    pub fn new(particle_handles: Vec<ParticleHandle>) -> Self {
        Self {
            particle_handles,
            restitution: DEFAULT_RESTITUTION,
            polygon: ClosedPolygon::empty(),
        }
    }

    pub fn with_restitution(mut self, coefficient: Real) -> Self {
        self.restitution = coefficient;
        self
    }

    pub fn add_particle_handle(&mut self, handle: ParticleHandle) {
        self.particle_handles.push(handle);
    }

    pub fn polygon(&self) -> &ClosedPolygon {
        &self.polygon
    }

    pub fn intersects_bounding_box(&self, other: &PolygonCollider) -> bool {
        self.polygon
            .bounding_box()
            .intersects(&other.polygon.bounding_box())
    }

    pub(crate) fn update<'a>(
        &mut self,
        _resolver: &CollisionResolver,
        context: &mut CollisionsContext<'a>,
    ) {
        let points = self
            .particle_handles
            .iter()
            .cloned()
            .filter_map(|handle| context.particles.get(handle))
            .map(|particle| Point2::from(particle.position));

        self.polygon.update(points);
    }

    pub(crate) fn resolve_collisions_with_polygon<'a>(
        &self,
        other: &PolygonCollider,
        resolver: &CollisionResolver,
        context: &mut CollisionsContext<'a>,
    ) {
        Self::resolve_collisions_between_polygons(&self, other, resolver, context);
        Self::resolve_collisions_between_polygons(other, &self, resolver, context);
    }

    fn resolve_collisions_between_polygons(
        collider: &PolygonCollider,
        other: &PolygonCollider,
        resolver: &CollisionResolver,
        context: &mut CollisionsContext,
    ) {
        for (particle_handle, point) in collider
            .particle_handles
            .iter()
            .cloned()
            .zip(collider.polygon.points())
        {
            if other.polygon.has_point_inside(point) {
                if let Some(segment) = other
                    .polygon
                    .closest_segment_within_bounding_box(point, collider.polygon.bounding_box())
                {
                    let segment_handle1 = collider.particle_handles[segment.index1];
                    let segment_handle2 = collider.particle_handles[segment.index2];

                    resolver.resolve(
                        PointInPolygon {
                            particle_handle,
                            particle_point: point,
                            segment_handle1,
                            segment_handle2,
                            segment_point1: segment.point1,
                            segment_point2: segment.point2,
                            ratio: segment.ratio,
                            depth: segment.depth,
                        },
                        context,
                    );
                }
            }
        }
    }

    // fn check_particles_collision2(&self, p1: &mut Particle, p2: &mut Particle) {
    //     let axis = if p1.position == p2.position {
    //         Vec2::new(0.001, 0.0)
    //     } else {
    //         p2.position - p1.position
    //     };
    //     let squared_distance = axis.magnitude_squared();
    //     let min_distance = p1.radius + p2.radius;
    //     if squared_distance < min_distance * min_distance {
    //         let normal = axis / squared_distance.simd_sqrt();
    //         let vrel = p2.velocity - p1.velocity;
    //         let masses = (p1.mass * p2.mass) / (p1.mass + p2.mass);
    //         let impulse = masses * (1.0 + self.restitution) * vrel.dot(&normal);
    //         p1.velocity += (impulse / p1.mass) * normal;
    //         p2.velocity -= (impulse / p2.mass) * normal;
    //     }
    // }
    //
    // fn check_particles_collision1(&self, p1: &mut Particle, p2: &mut Particle) {
    //     let axis = if p1.position == p2.position {
    //         Vec2::new(0.001, 0.0)
    //     } else {
    //         p2.position - p1.position
    //     };
    //     let squared_distance = axis.magnitude_squared();
    //     let min_distance = p1.radius + p2.radius;
    //     if squared_distance < min_distance * min_distance {
    //         let total_mass = p1.mass + p2.mass;
    //         let mass_ratio1 = p1.mass / total_mass;
    //         let mass_ratio2 = p2.mass / total_mass;
    //         let distance = squared_distance.simd_sqrt();
    //         let depth = min_distance - distance;
    //         let normal = axis / distance;
    //         let delta = 0.5 * self.restitution * depth;
    //         p1.position -= normal * (mass_ratio1 * delta);
    //         p2.position += normal * (mass_ratio2 * delta);
    //     }
    // }
}
