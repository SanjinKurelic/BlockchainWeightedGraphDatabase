use crate::graph::Edge;
use derive_more::Constructor;
use rustc_hash::FxHashMap;

#[derive(Constructor, Clone)]
pub struct Node {
    pub attributes: FxHashMap<String, String>,
    pub edges: Vec<Edge>,
}
