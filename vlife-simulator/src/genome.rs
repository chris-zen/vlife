use rand::Rng;
use std::collections::BTreeSet;
use std::{cell::RefCell, collections::BTreeMap};

use crate::real::Real;
use crate::M;

pub trait BuildGenome {
    fn build_genome<'a>(&self, builder: GenomeBuilder);
}

pub trait ApplyGenome {
    fn apply_genome(&mut self, genome: &Genome);
}

#[derive(Debug, Clone)]
pub struct Genome {
    genes: BTreeMap<String, Gen>,
}

impl Genome {
    pub fn _get(&self, path: Option<String>, name: &str) -> Option<&Gen> {
        let id = Self::gen_id(path.as_deref(), name);
        self.genes.get(&id)
    }

    pub(crate) fn _mutate(&mut self, _num_mutations: usize, _probability: Real) {
        todo!()
    }

    pub(crate) fn cross(&self, other: &Genome) -> Genome {
        let mut rng = rand::thread_rng();
        let keys1 = self
            .genes
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let keys2 = other
            .genes
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let keys = keys1.union(&keys2).collect::<Vec<_>>();
        let num_genes = keys.len();
        let cross_index = rng.gen_range(1..num_genes - 1);
        let mut genes = BTreeMap::new();
        let (keys1, keys2) = keys.split_at(cross_index);
        for key in keys1 {
            let gen = self
                .genes
                .get(**key)
                .or_else(|| other.genes.get(**key))
                .expect("gen");
            genes.insert(key.to_string(), gen.clone());
        }
        for key in keys2 {
            let gen = other
                .genes
                .get(**key)
                .or_else(|| self.genes.get(**key))
                .expect("gen");
            genes.insert(key.to_string(), gen.clone());
        }
        Genome { genes }
    }

    fn gen_id(path: Option<&str>, name: &str) -> String {
        if let Some(path) = path {
            format!("{path}/{name}")
        } else {
            name.to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Gen {
    // This is used by the GenomeBuilder macro
    #[allow(dead_code)]
    pub(crate) value: Real,
}

#[derive(Clone)]
pub struct GenomeBuilder {
    path: Option<String>,
    genes: RefCell<BTreeMap<String, Gen>>,
}

impl GenomeBuilder {
    pub fn new() -> Self {
        Self {
            path: None,
            genes: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn nested(&self, name: &str) -> Self {
        let path = self
            .path
            .as_ref()
            .map(|path| format!("{path}/{name}"))
            .or_else(|| Some(name.to_string()));

        Self {
            path,
            genes: self.genes.clone(),
        }
    }

    pub fn add(&self, name: &str, gen: Gen) {
        let id = Genome::gen_id(self.path.as_deref(), name);
        self.genes.borrow_mut().insert(id, gen);
    }

    pub fn build(self) -> Genome {
        Genome {
            genes: self.genes.into_inner(),
        }
    }
}

// impl<const R: usize> BuildGenome for V<R> {
//     fn build_genome<'a>(&self, builder: GenomeBuilder) {
//         for (index, value) in self.iter().copied().enumerate() {
//             let name = format!("{index:03}");
//             builder.add(&name, Gen { value });
//         }
//     }
// }

impl<const R: usize, const C: usize> BuildGenome for M<R, C> {
    fn build_genome<'a>(&self, builder: GenomeBuilder) {
        for (row_index, row) in self.row_iter().enumerate() {
            let row_name = format!("{row_index:03}");
            let row_builder = builder.nested(&row_name);
            for (col_index, value) in row.iter().copied().enumerate() {
                let col_name = format!("{col_index:03}");
                row_builder.add(&col_name, Gen { value: value })
            }
        }
    }
}
