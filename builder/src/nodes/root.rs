use std::cell::UnsafeCell;

use crate::cache::NodeCache;
use crate::error::Result;
use crate::externs::PythonScript;
use crate::n3_std;
use crate::seed::Seed;
use crate::tensor::TensorNode;

pub struct NodeRoot {
    pub(crate) seed: Seed,
    sources: NodeCache<TensorNode>,
    externs: NodeCache<PythonScript>,
    pub(crate) parser: crate::Parser,
    _thread_unsafe: UnsafeCell<()>,
}

impl NodeRoot {
    pub fn new() -> Self {
        Self {
            seed: Seed::default(),
            sources: NodeCache::new(n3_std::get_sources()),
            externs: NodeCache::new(n3_std::get_externs()),
            parser: crate::Parser::new(),
            _thread_unsafe: UnsafeCell::new(()),
        }
    }

    pub fn add_source(&self, name: String, source: String) {
        self.sources.add_source(name, source);
    }

    pub fn get(&self, name: &str) -> Result<TensorNode> {
        self.sources.get(name, self)
    }

    pub fn get_extern(&self, name: &str) -> Result<PythonScript> {
        self.externs.get(name, self)
    }
}
