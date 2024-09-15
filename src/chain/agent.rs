use crate::chain::error::ChainError;
use crate::graph::Graph;
use rustc_hash::FxHashMap;

// Note: This should be implemented as API call to graph db
pub struct AgentService {
    pub(crate) agents: FxHashMap<String, FxHashMap<String, String>>,
    pub(crate) accounts: FxHashMap<String, (String, usize)>,
}

impl Default for AgentService {
    fn default() -> Self {
        AgentService {
            agents: FxHashMap::default(),
            accounts: FxHashMap::default(),
        }
    }
}

impl AgentService {
    pub fn define_agent(&mut self, node_name: String, conditions: FxHashMap<String, String>) {
        self.agents.insert(node_name, conditions);
    }

    pub fn add_or_update_agent(&mut self, graph: &mut Graph, node_name: String, identifier: &String) -> Result<(String, usize), ChainError> {
        if let Ok(value) = self.validate_agent(graph, node_name, identifier) {
            self.accounts.insert(identifier.clone(), value.clone());

            Ok(value)
        } else {
            self.remove_agent(identifier);

            Err(ChainError::NotQualifiedForAgent(identifier.clone()))
        }
    }

    pub fn remove_agent(&mut self, identifier: &String) {
        self.accounts.remove(identifier);
    }

    fn validate_agent(&self, graph: &mut Graph, node_name: String, identifier: &String) -> Result<(String, usize), ChainError> {
        let agent = self.agents.get(&node_name).ok_or(ChainError::WrongAgentIdentifier(identifier.clone()))?;
        let node = graph
            .find_by_id(&node_name, identifier)
            .map_err(|_| ChainError::WrongAgentIdentifier(identifier.clone()))?;

        for (condition, condition_value) in agent {
            if node.attributes.get(condition) != Some(condition_value) {
                return Err(ChainError::WrongAgentIdentifier(identifier.clone()));
            }
        }

        let p_key = node.attributes.get("key").ok_or_else(|| ChainError::WrongAgentKey(node_name.clone()))?;
        Ok((p_key.clone(), node.edges.len()))
    }

    pub fn get_difficulty(&self, identifier: &String) -> usize {
        self.accounts.get(identifier).map_or(0, |(_, difficulty)| *difficulty)
    }

    pub fn get_validator_difficulty(&self, validator: &String) -> usize {
        self.accounts
            .iter()
            .filter(|(_, (p_key, _))| p_key == validator)
            .map(|(_, (_, difficulty))| *difficulty)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::attribute::InternalNodeAttribute;

    #[test]
    fn should_define_agent() {
        // Given
        let mut agent_service = AgentService::default();

        // When
        agent_service.define_agent("User".to_string(), FxHashMap::default());

        // Then
        assert_eq!(agent_service.agents.len(), 1);
    }

    #[test]
    fn should_add_or_update_agent() {
        // Given
        let mut agent_service = AgentService::default();
        let mut graph = Graph::default();
        let identifier = insert_agent(&mut graph);

        define_agent(&mut agent_service);

        // When
        let result = agent_service.add_or_update_agent(&mut graph, "User".to_string(), &identifier);

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ("1234567890".to_string(), 0));
        assert_eq!(agent_service.agents.len(), 1);
        assert_eq!(agent_service.accounts.len(), 1);
    }

    #[test]
    fn should_remove_agent() {
        // Given
        let mut agent_service = AgentService::default();
        let mut graph = Graph::default();
        let identifier = insert_agent(&mut graph);

        define_agent(&mut agent_service);
        let _ = agent_service.add_or_update_agent(&mut graph, "User".to_string(), &identifier);

        // Change user to non-premium
        let mut attributes = FxHashMap::default();
        attributes.insert(InternalNodeAttribute::ID_ATTRIBUTE.to_string(), identifier.clone());
        graph.update_node("User".to_string(), attributes).unwrap();

        // When
        let result = agent_service.add_or_update_agent(&mut graph, "User".to_string(), &identifier);

        // Then
        assert!(result.is_err());
        assert_eq!(agent_service.agents.len(), 1);
        assert_eq!(agent_service.accounts.len(), 0);
    }

    #[test]
    fn should_get_difficulty() {
        // Given
        let mut agent_service = AgentService::default();
        let mut graph = Graph::default();
        let identifier = insert_agent(&mut graph);

        // Insert edge
        let mut wrapped_identifier = FxHashMap::default();
        wrapped_identifier.insert(InternalNodeAttribute::ID_ATTRIBUTE.to_string(), identifier.clone());
        graph
            .add_edge(
                ("User".to_string(), wrapped_identifier.clone()),
                ("User".to_string(), wrapped_identifier),
                1,
            )
            .unwrap();

        define_agent(&mut agent_service);
        let _ = agent_service.add_or_update_agent(&mut graph, "User".to_string(), &identifier);

        // When
        let difficulty = agent_service.get_difficulty(&identifier);

        // Then
        assert_eq!(difficulty, 1);
    }

    fn insert_agent(graph: &mut Graph) -> String {
        let mut attributes = FxHashMap::default();

        attributes.insert("premium".to_string(), "true".to_string());
        attributes.insert("key".to_string(), "1234567890".to_string());

        graph.create_definition("User".to_string(), attributes.keys().cloned().collect()).unwrap();
        InternalNodeAttribute::get_identifier(&graph.add_node("User".to_string(), attributes).unwrap().first().unwrap())
    }

    fn define_agent(agent_service: &mut AgentService) {
        let mut conditions = FxHashMap::default();

        conditions.insert("premium".to_string(), "true".to_string());

        agent_service.define_agent("User".to_string(), FxHashMap::default());
    }
}
