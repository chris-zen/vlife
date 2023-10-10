use crate::object_set::ObjectSet;
use crate::physics::collisions::resolver::CollisionResolver;
use crate::physics::collisions::CollisionsContext;
use crate::physics::{Particle, PolygonCollider};

pub mod polygon;

pub enum Collider {
    Polygon(PolygonCollider),
}

impl Collider {
    pub(crate) fn intersects(&self, other: &Collider) -> bool {
        match (self, other) {
            (Self::Polygon(collider), Self::Polygon(other)) => {
                collider.intersects_bounding_box(other)
            }
        }
    }

    pub(crate) fn update<'a>(
        &mut self,
        resolver: &CollisionResolver,
        context: &mut CollisionsContext<'a>,
    ) {
        match self {
            Self::Polygon(collider) => collider.update(resolver, context),
        }
    }

    pub(crate) fn resolve_collisions<'a>(
        &self,
        other: &Collider,
        resolver: &CollisionResolver,
        context: &mut CollisionsContext<'a>,
    ) {
        match (self, other) {
            (Self::Polygon(collider), Self::Polygon(other)) => {
                collider.resolve_collisions_with_polygon(other, resolver, context)
            }
        }
    }
}

impl From<PolygonCollider> for Collider {
    fn from(collider: PolygonCollider) -> Self {
        Self::Polygon(collider)
    }
}
