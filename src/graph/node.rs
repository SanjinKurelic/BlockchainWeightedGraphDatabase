use derive_more::Constructor;
use rustc_hash::FxHashMap;
use crate::graph::Edge;

#[derive(Constructor, Clone)]
pub struct Node {
    pub attributes: FxHashMap<String, String>,
    pub edges: Vec<Edge>
}