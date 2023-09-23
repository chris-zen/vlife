use ordered_float::NotNan;
use rand::Rng;
use std::collections::BTreeMap;

use crate::cell::Cell;
use crate::genome::{BuildGenome, Genome, GenomeBuilder};
use crate::Scalar;

pub struct CellRank {
    cells: BTreeMap<NotNan<Scalar>, Cell>,
    max_size: usize,
}

impl CellRank {
    pub fn new(max_size: usize) -> Self {
        Self {
            cells: BTreeMap::default(),
            max_size,
        }
    }

    pub fn choose_random_genome(&self) -> Option<Genome> {
        let mut rng = rand::thread_rng();
        if !self.cells.is_empty() {
            let drop = rng.gen_range(0..self.cells.len());
            if let Some(cell) = self.cells.values().skip(drop).next() {
                let builder = GenomeBuilder::new();
                cell.build_genome(builder.clone());
                let genome = builder.build();
                Some(genome)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn insert(&mut self, score: Scalar, cell: Cell) {
        let score = NotNan::new(score).expect("non-nan-score");
        self.cells.insert(score, cell);
        if self.cells.len() > self.max_size {
            self.cells.pop_first();
        }
    }
}
