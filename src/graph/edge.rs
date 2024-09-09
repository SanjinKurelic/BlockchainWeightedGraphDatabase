use derive_more::Constructor;

#[derive(Constructor, Clone)]
pub struct Edge {
    pub to_node: String,
    pub to_node_id: String,
    pub weight: i8,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.to_node_id == other.to_node_id
    }
}
