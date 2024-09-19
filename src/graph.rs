use crate::graph::attribute::InternalNodeAttribute;
use edge::Edge;
use error::DatabaseError;
use node::Node;
use rustc_hash::FxHashMap;
use std::vec;

pub mod attribute;
mod edge;
pub(crate) mod error;
mod generator;
pub(crate) mod node;

pub struct Graph {
    pub definitions: FxHashMap<String, Vec<String>>,
    pub nodes: FxHashMap<String, Node>,
}

impl Default for Graph {
    fn default() -> Self {
        Graph {
            definitions: FxHashMap::default(),
            nodes: FxHashMap::default(),
        }
    }
}

pub type GraphResults = Result<Vec<FxHashMap<String, String>>, DatabaseError>;

impl Graph {
    /// Fetch node
    ///
    /// Fetch node with all joins by given attributes. If no node was found, error is returned.
    /// If node is found but joins does not meet given query, empty result is returned.
    /// This behaviour is currently ok, as we can only fetch nodes by id. Fetching by attributes
    /// would require adding searchable index tree.
    pub fn search(&mut self, name: String, attributes: FxHashMap<String, String>, joins: Vec<(String, i8)>) -> GraphResults {
        let node = self.fetch_node(&name, &attributes)?.clone();

        let mut result = node.attributes;

        // Collect edges
        for (join, weight) in &joins {
            let edge = node.edges.iter().find(|edge| edge.to_node == *join);

            if edge.is_none() || edge.unwrap().weight < *weight {
                return Ok(vec![]);
            }

            let edge = edge.unwrap();
            self.find_by_id(&edge.to_node, &edge.to_node_id)?
                .attributes
                .iter()
                .for_each(|(key, value)| {
                    result.insert(format!("{}.{key}", edge.to_node), value.clone());
                });
        }

        Ok(vec![result])
    }

    /// Create node definition
    ///
    /// Node definition is used to validate all queries against specific node, e.g. are all attributes defined.
    pub fn create_definition(&mut self, name: String, attributes: Vec<String>) -> GraphResults {
        if self.definitions.contains_key(&name) {
            return Err(DatabaseError::NodeAlreadyExists(name));
        }

        self.definitions.insert(name, attributes.clone());

        self.return_definition(attributes)
    }

    /// Add node to the graph
    ///
    /// This method will add named node with given attributes to the graph database.
    /// Method will also check if attributes are valid and does not contain any internal attribute.
    pub fn add_node(&mut self, name: String, mut attributes: FxHashMap<String, String>) -> GraphResults {
        self.validate_attributes(&name, &attributes, vec![])?;

        let identifier = generator::IdGenerator::generate();
        attributes.insert(InternalNodeAttribute::ID_ATTRIBUTE.to_string(), identifier.clone());
        attributes.insert(InternalNodeAttribute::NAME_ATTRIBUTE.to_string(), name.clone());
        attributes.insert(InternalNodeAttribute::EDGE_COUNT_ATTRIBUTE.to_string(), "0".to_string());

        self.nodes.insert(format!("{identifier}:{name}"), Node::new(attributes.clone(), vec![]));

        Ok(vec![attributes])
    }

    /// Update existing node with the new attributes
    ///
    /// This method will update existing node with the new attributes. In the list of the attributes, internal attribute
    /// $id must be present so specific node is found. Other internal attributes are not possible to set or change.
    /// If node was not found, appropriate error will be returned.
    pub fn update_node(&mut self, name: String, mut attributes: FxHashMap<String, String>) -> GraphResults {
        self.validate_attributes(&name, &attributes, vec![InternalNodeAttribute::ID_ATTRIBUTE])?;

        let node = self.fetch_node(&name, &attributes)?;

        // New attributes map already contains $id, so only other internal variables are required to append
        attributes.insert(InternalNodeAttribute::NAME_ATTRIBUTE.to_string(), name);
        attributes.insert(InternalNodeAttribute::EDGE_COUNT_ATTRIBUTE.to_string(), node.edges.len().to_string());

        node.attributes = attributes.clone();

        Ok(vec![attributes])
    }

    /// Delete existing node from the graph
    ///
    /// This method will delete existing node from the graph. In the list of the attributes, internal attribute
    /// $id must be present so specific node is deleted.
    /// If node was not found, appropriate error will be returned.
    pub fn delete_node(&mut self, name: String, attributes: FxHashMap<String, String>) -> GraphResults {
        self.validate_attributes(&name, &attributes, vec![InternalNodeAttribute::ID_ATTRIBUTE])?;

        let identifier = InternalNodeAttribute::get_identifier(&attributes);

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
        self.validate_edge((&from_name, &from_atr), (&to_name, &to_atr))?;

        let node = self.fetch_node(&from_name, &from_atr)?;
        let edge = Edge::new(to_name.clone(), InternalNodeAttribute::get_identifier(&to_atr), weight);

        if node.edges.contains(&edge) {
            return Err(DatabaseError::EdgeAlreadyExists(from_name, to_name));
        }

        node.edges.push(edge);
        node.attributes
            .insert(InternalNodeAttribute::EDGE_COUNT_ATTRIBUTE.to_string(), node.edges.len().to_string());

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
        self.validate_edge((&from_name, &from_atr), (&to_name, &to_atr))?;

        let node = self.fetch_node(&from_name, &from_atr)?;

        let to_id = InternalNodeAttribute::get_identifier(&to_atr);
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
        self.validate_edge((&from_name, &from_atr), (&to_name, &to_atr))?;

        let node = self.fetch_node(&from_name, &from_atr)?;

        let to_id = InternalNodeAttribute::get_identifier(&to_atr);
        let edge_position = node
            .edges
            .iter()
            .position(|edge| edge.to_node_id == *to_id)
            .ok_or(DatabaseError::EdgeNotFound(from_name.clone(), to_name.clone()))?;

        // Swap remove and get weight used for returning deleted element
        let weight = node.edges.swap_remove(edge_position).weight;

        // Update edge counter
        node.attributes
            .insert(InternalNodeAttribute::EDGE_COUNT_ATTRIBUTE.to_string(), node.edges.len().to_string());

        self.return_edge(from_name, to_name, weight)
    }

    fn return_definition(&self, attributes: Vec<String>) -> GraphResults {
        let mut result = FxHashMap::default();

        attributes.iter().for_each(|attribute| {
            result.insert(attribute.clone(), "*".to_string());
        });

        Ok(vec![result])
    }

    fn return_edge(&mut self, from: String, to: String, weight: i8) -> GraphResults {
        let mut edge_attributes = FxHashMap::default();

        edge_attributes.insert(InternalNodeAttribute::FROM_ATTRIBUTE.to_string(), from);
        edge_attributes.insert(InternalNodeAttribute::TO_ATTRIBUTE.to_string(), to);
        edge_attributes.insert(InternalNodeAttribute::WEIGHT_ATTRIBUTE.to_string(), weight.to_string());

        Ok(vec![edge_attributes])
    }

    /// Check if attributes are same as defined in node definition.
    /// Also, check if only allowed internal attributes are present in attributes map.
    fn validate_attributes(
        &mut self,
        node_name: &String,
        check: &FxHashMap<String, String>,
        internal_attributes: Vec<&str>,
    ) -> Result<(), DatabaseError> {
        let allowed_attributes = self.definitions.get(node_name).ok_or(DatabaseError::NodeNotDefined(node_name.clone()))?;

        for (key, _) in check {
            if key.starts_with('$') && !internal_attributes.contains(&key.as_str()) {
                return Err(DatabaseError::AttributeNotAllowed(key.clone()));
            } else if !key.starts_with('$') && !allowed_attributes.contains(key) {
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
    fn validate_edge(
        &mut self,
        (from_name, from_atr): (&String, &FxHashMap<String, String>),
        (to_name, to_atr): (&String, &FxHashMap<String, String>),
    ) -> Result<(), DatabaseError> {
        self.validate_attributes(from_name, from_atr, vec![InternalNodeAttribute::ID_ATTRIBUTE])?;
        self.validate_attributes(to_name, to_atr, vec![InternalNodeAttribute::ID_ATTRIBUTE])?;

        Ok(())
    }

    /// This method will find node and return mut reference.
    fn fetch_node(&mut self, name: &String, attributes: &FxHashMap<String, String>) -> Result<&mut Node, DatabaseError> {
        let identifier = InternalNodeAttribute::get_identifier(attributes);

        self.find_by_id(name, &identifier)
    }

    pub fn find_by_id(&mut self, name: &String, identifier: &String) -> Result<&mut Node, DatabaseError> {
        self.nodes
            .get_mut(format!("{identifier}:{name}").as_str())
            .ok_or(DatabaseError::NodeNotFound(name.clone(), identifier.clone()))
    }
}

// There are no test cases for this module as it is tested though query processor integration test cases.
