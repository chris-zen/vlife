use std::fmt::Display;

use crate::cell::Cell;
use crate::object_set::ObjectHandle;
use crate::physics::{Particle, ParticleHandle, Physics, Spring, SpringHandle};
use crate::Vec2;

pub type CellHandle = ObjectHandle<CellBody>;

pub struct CellBody {
    pub(crate) cell: Cell,
    pub(crate) center: ParticleHandle,
    pub(crate) particles: Vec<ParticleHandle>,
    pub(crate) springs: Vec<SpringHandle>,
}

impl CellBody {
    pub fn view<'a>(&'a self, handle: CellHandle, physics: &'a Physics) -> CellView<'a> {
        let center = physics
            .get_particle(self.center)
            .expect("center-particle")
            .clone();
        let particles = self
            .particles
            .iter()
            .filter_map(|handle| physics.get_particle(*handle))
            .cloned()
            .collect();
        CellView::new(handle, &self.cell, center, particles)
    }

    pub fn view_mut<'a>(
        &'a mut self,
        handle: CellHandle,
        physics: &'a mut Physics,
    ) -> CellViewMut<'a> {
        CellViewMut::new(handle, &mut self.cell)
    }
}

pub struct CellView<'a> {
    handle: CellHandle,
    cell: &'a Cell,
    center: Particle,
    particles: Vec<Particle>,
}

impl<'a> CellView<'a> {
    pub fn new(
        handle: CellHandle,
        cell: &'a Cell,
        center: Particle,
        particles: Vec<Particle>,
    ) -> Self {
        Self {
            handle,
            cell,
            center,
            particles,
        }
    }

    pub fn handle(&self) -> CellHandle {
        self.handle
    }

    pub fn cell(&self) -> &Cell {
        self.cell
    }

    pub fn position(&self) -> Vec2 {
        self.center.position
    }

    pub fn membrane(&self) -> Vec<Vec2> {
        self.particles
            .iter()
            .map(|particle| particle.position)
            .collect()
    }
}

impl<'a> Display for CellView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{}", self.object)?;
        // write!(f, "{}", self.cell)?;
        Ok(())
    }
}

pub struct CellViewMut<'a> {
    handle: CellHandle,
    cell: &'a mut Cell,
}

impl<'a> CellViewMut<'a> {
    pub fn new(handle: CellHandle, cell: &'a mut Cell) -> Self {
        Self { handle, cell }
    }

    pub fn handle(&self) -> CellHandle {
        self.handle
    }

    pub fn cell(&mut self) -> &mut Cell {
        self.cell
    }
}
