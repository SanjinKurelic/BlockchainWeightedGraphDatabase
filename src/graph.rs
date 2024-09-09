use edge::Edge;
use error::DatabaseError;
use node::Node;
use rustc_hash::FxHashMap;
use std::vec;

mod edge;
mod error;
mod generator;
mod index;
mod node;

pub struct Graph {
    pub nodes: FxHashMap<String, Node>,
    pub indexes: FxHashMap<&'static str, String>,
}

impl Default for Graph {
    fn default() -> Self {
        Graph {
            nodes: FxHashMap::default(),
            indexes: FxHashMap::default(),
        }
    }
}

pub type GraphResults = Result<Vec<FxHashMap<String, String>>, DatabaseError>;

impl Graph {
    const ID_ATTRIBUTE: &'static str = "$id";
    const NAME_ATTRIBUTE: &'static str = "$name";

    /// Add node to the graph
    ///
    /// This method will add named node with given attributes to the graph database.
    /// Method will also check if attributes are valid and does not contain any internal attribute.
    pub fn add_node(&mut self, name: String, mut attributes: FxHashMap<String, String>) -> GraphResults {
        self.validate_attributes(&attributes, vec![])?;

        let identifier = generator::IdGenerator::generate();
        attributes.insert(Self::ID_ATTRIBUTE.to_string(), identifier.clone());
        attributes.insert(Self::NAME_ATTRIBUTE.to_string(), name.clone());

        self.nodes.insert(format!("{identifier}:{name}"), Node::new(attributes.clone(), vec![]));

        Ok(vec![attributes])
    }

    /// Update existing node with the new attributes
    ///
    /// This method will update existing node with the new attributes. In the list of the attributes, internal attribute
    /// $id must be present so specific node is found. Other internal attributes are not possible to set or change.
    /// If node was not found, appropriate error will be returned.
    pub fn update_node(&mut self, name: String, mut attributes: FxHashMap<String, String>) -> GraphResults {
        self.validate_attributes(&attributes, vec![Self::ID_ATTRIBUTE])?;

        let node = self.fetch_node(&name, &attributes)?;

        // New attributes map already contains $id, so only name is required to append
        attributes.insert(Self::NAME_ATTRIBUTE.to_string(), name);

        node.attributes = attributes.clone();

        Ok(vec![attributes])
    }

    /// Delete existing node from the graph
    ///
    /// This method will delete existing node from the graph. In the list of the attributes, internal attribute
    /// $id must be present so specific node is deleted.
    /// If node was not found, appropriate error will be returned.
    pub fn delete_node(&mut self, name: String, attributes: FxHashMap<String, String>) -> GraphResults {
        self.validate_attributes(&attributes, vec![Self::ID_ATTRIBUTE])?;

        let identifier = attributes.get(Self::ID_ATTRIBUTE).unwrap();

        Ok(vec![
            self.nodes
                .remove(format!("{identifier}:{name}").as_str())
                .ok_or(DatabaseError::NodeNotFound(name.clone(), identifier.clone()))?
                .attributes,
        ])
    }

    /// Connect two nodes with given weight
    ///
    /// This method will crete edge (connection) between two nodes (from/to name/identifier) with given weight.
    /// If from node or to node does not exist or edge already exist, appropriate error will be returned.
    pub fn add_edge(
        &mut self,
        (from_name, from_atr): (String, FxHashMap<String, String>),
        (to_name, to_atr): (String, FxHashMap<String, String>),
        weight: i8,
    ) -> GraphResults {
        self.validate_edge(&from_atr, &to_atr)?;

        let node = self.fetch_node(&from_name, &from_atr)?;
        let edge = Edge::new(to_name.clone(), to_atr.get(Self::ID_ATTRIBUTE).unwrap().clone(), weight);

        if node.edges.contains(&edge) {
            return Err(DatabaseError::EdgeAlreadyExists(from_name, to_name));
        }

        node.edges.push(edge);

        self.return_edge(from_name, to_name, weight)
    }

    /// Update connection between two nodes
    ///
    /// This method will update weight of edge (connection) between two nodes (from/to name/identifier).
    /// If from node or to node does not exist or edge does not exist, appropriate error will be returned.
    pub fn update_edge(
        &mut self,
        (from_name, from_atr): (String, FxHashMap<String, String>),
        (to_name, to_atr): (String, FxHashMap<String, String>),
        weight: i8,
    ) -> GraphResults {
        self.validate_edge(&from_atr, &to_atr)?;

        let node = self.fetch_node(&from_name, &from_atr)?;

        let to_id = to_atr.get(Self::ID_ATTRIBUTE).unwrap();
        let edge = node
            .edges
            .iter_mut()
            .find(|edge| edge.to_node_id == *to_id)
            .ok_or(DatabaseError::EdgeNotFound(from_name.clone(), to_name.clone()))?;

        edge.weight = weight;

        self.return_edge(from_name, to_name, weight)
    }

    /// Delete connection between two nodes
    ///
    /// This method will delete edge (connection) between two nodes (from/to name/identifier).
    /// If edge/connection does not exist, appropriate error will be returned.
    pub fn delete_edge(
        &mut self,
        (from_name, from_atr): (String, FxHashMap<String, String>),
        (to_name, to_atr): (String, FxHashMap<String, String>),
    ) -> GraphResults {
        self.validate_edge(&from_atr, &to_atr)?;

        let node = self.fetch_node(&from_name, &from_atr)?;

        let to_id = to_atr.get(Self::ID_ATTRIBUTE).unwrap();
        let edge_position = node
            .edges
            .iter()
            .position(|edge| edge.to_node_id == *to_id)
            .ok_or(DatabaseError::EdgeNotFound(from_name.clone(), to_name.clone()))?;

        // Swap remove and get weight used for returning deleted element
        let weight = node.edges.swap_remove(edge_position).weight;

        self.return_edge(from_name, to_name, weight)
    }

    fn return_edge(&mut self, from: String, to: String, weight: i8) -> GraphResults {
        let mut edge_attributes = FxHashMap::default();

        edge_attributes.insert("$from".to_string(), from);
        edge_attributes.insert("$to".to_string(), to);
        edge_attributes.insert("$weight".to_string(), weight.to_string());

        Ok(vec![edge_attributes])
    }

    /// Check if internal attributes are present in attributes map. Other internal variables are not allowed to be used.
    fn validate_attributes(&mut self, check: &FxHashMap<String, String>, internal_attributes: Vec<&str>) -> Result<(), DatabaseError> {
        for (key, _) in check {
            if key.starts_with('$') && !internal_attributes.contains(&key.as_str()) {
                return Err(DatabaseError::AttributeNotAllowed(key.clone()));
            }
        }

        for attribute in internal_attributes {
            if !check.contains_key(attribute) {
                return Err(DatabaseError::AttributeIsRequired(attribute.to_string()));
            }
        }

        Ok(())
    }

    /// This method will validate edge attributes for bot from and to node.
    fn validate_edge(&mut self, from: &FxHashMap<String, String>, to: &FxHashMap<String, String>) -> Result<(), DatabaseError> {
        self.validate_attributes(from, vec![Self::ID_ATTRIBUTE])?;
        self.validate_attributes(to, vec![Self::ID_ATTRIBUTE])?;

        Ok(())
    }

    /// This method will find node and return mut reference.
    fn fetch_node(&mut self, name: &String, attributes: &FxHashMap<String, String>) -> Result<&mut Node, DatabaseError> {
        let identifier = attributes.get(Self::ID_ATTRIBUTE).unwrap();

        self.nodes
            .get_mut(format!("{identifier}:{name}").as_str())
            .ok_or(DatabaseError::NodeNotFound(name.clone(), identifier.clone()))
    }
}
