extern crate peg;

use crate::chain::Chain;
use crate::graph::Graph;
use crate::graph::GraphResults;
use peg::error::ParseError;
use peg::str::LineCol;
use rustc_hash::FxHashMap;

peg::parser! {
    grammar query_parser(graph: &mut Graph, chain: &mut Chain) for str {
        use crate::graph::attribute::InternalNodeAttribute;

        pub rule command() -> GraphResults = define_node() / add_node() / update_node() / delete_node() / add_edge() / update_edge() / delete_edge() / fetch_node() / fetch_connection()

        rule define_node() -> GraphResults = _ "define" _ "node" _ name:name() _ attributes:attribute_definitions() _ conditions:agent()? {
            let result = graph.create_definition(name.to_string(), attributes.iter().map(|attribute| attribute.to_string()).collect());

            if result.is_ok() && conditions.is_some() {
                chain.define_agent(name.to_string(), conditions.unwrap())
            }

            result
        }

        rule fetch_node() -> GraphResults = _ "fetch" _ "node" _ name:name() _ attributes:attributes() _ joins:joins() {
            graph.search(name.to_string(), attributes, joins)
        }

        rule fetch_connection() -> GraphResults = _ "fetch" _ "connection" _ "chain" {
            chain.as_graph_result()
        }

        rule add_node() -> GraphResults = _ "add" _ "node" _ name:name() _ attributes:attributes()? {
            let result = graph.add_node(name.to_string(), attributes.clone().unwrap_or_else(FxHashMap::default));

            // Attributes are required for agent registration
            if result.is_ok() && attributes.is_some() {
                chain.add_or_update_agent(graph, name.to_string(), InternalNodeAttribute::get_identifier(result.clone().unwrap().first().unwrap()));
            }

            result
        }

        rule add_edge() -> GraphResults = _ "add" _ "connection" _ "from" _ from_name:name() _ from_attributes:attributes() _ "to" _ to_name:name() _ to_attributes:attributes() _ "with" _ "weight" _ weight:weight()  {
            let result = graph.add_edge((from_name.to_string(), from_attributes.clone()), (to_name.to_string(), to_attributes.clone()), weight);

            if result.is_ok() {
                  if let Err(error) = chain.add_edge_change(InternalNodeAttribute::get_identifier(&from_attributes),InternalNodeAttribute::get_identifier(&to_attributes), weight) {
                    eprintln!("Chain error: {error}");
                }
            }

            result
        }

        rule update_node() -> GraphResults = _ "update" _ "node" _ name:name() _ attributes:attributes() {
            let result = graph.update_node(name.to_string(), attributes.clone());

            // Handle case where user does not meet conditions anymore
            if result.is_ok() {
               chain.add_or_update_agent(graph, name.to_string(), InternalNodeAttribute::get_identifier(&attributes));
            }

            result
        }

        rule update_edge() -> GraphResults = _ "update" _ "connection" _ "from" _ from_name:name() _ from_attributes:attributes() _ "to" _ to_name:name() _ to_attributes:attributes() _ "with" _ "weight" _ weight:weight()  {
            let result = graph.update_edge((from_name.to_string(), from_attributes.clone()), (to_name.to_string(), to_attributes.clone()), weight);

            if result.is_ok() {
                if let Err(error) = chain.add_edge_change(InternalNodeAttribute::get_identifier(&from_attributes),InternalNodeAttribute::get_identifier(&to_attributes), weight) {
                    eprintln!("Chain error: {error}");
                }
            }

            result
        }

        rule delete_node() -> GraphResults = _ "delete" _ "node" _ name:name() _ attributes:attributes() {
            let result = graph.delete_node(name.to_string(), attributes.clone());

            if result.is_ok() {
                chain.remove_agent(InternalNodeAttribute::get_identifier(&attributes));
            }

            result
        }

        rule delete_edge() -> GraphResults = _ "delete" _ "connection" _ "from" _ from_name:name() _ from_attributes:attributes() _ "to" _ to_name:name() _ to_attributes:attributes() {
            let result = graph.delete_edge((from_name.to_string(), from_attributes.clone()), (to_name.to_string(), to_attributes.clone()));

            if result.is_ok() {
                if let Err(error) = chain.add_edge_change(InternalNodeAttribute::get_identifier(&from_attributes),InternalNodeAttribute::get_identifier(&to_attributes), 0) {
                    eprintln!("Chain error: {error}");
                }
            }

            result
        }

        rule agent() -> FxHashMap<String, String> = _ "with" _ "agent" _ conditions:attributes() { conditions }

        rule joins() -> Vec<(String, i8)> = joins:join() ** _ { joins }

        rule join() -> (String, i8) = _ "join" _ name:name() _ "($weight>\"" weight:weight() "\")" { (name.to_string(), weight) }

        rule attributes() -> FxHashMap<String, String> = "(" attributes:attribute() ** "," ")" {
            attributes.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<FxHashMap<String, String>>()
        }

        rule attribute() -> (&'input str, &'input str) = name:attribute_name() "=" value:attribute_value() { (name, value) }

        rule attribute_name() -> &'input str = $(['a'..='z' | 'A'..='Z' | '0'..='9' | '$' | '*']+)

        rule attribute_value() -> &'input str = "\"" value:__ "\"" { value }

        rule attribute_definitions() -> Vec<&'input str> = "(" names:attribute_definition() ** "," ")" { names }

        rule attribute_definition() -> &'input str = $(['a'..='z' | 'A'..='Z' | '0'..='9' | '*']+)

        rule name() -> &'input str = $(['a'..='z' | 'A'..='Z']+)

        rule weight() -> i8 = n:$(['0'..='9']+) { n.parse().unwrap() }

        rule __ -> &'input str = $([^'"']*)

        rule _ -> &'input str = $([' ']*)
    }
}

pub struct QueryProcessor;

impl QueryProcessor {
    pub fn parse_command(mut graph: &mut Graph, mut chain: &mut Chain, command: &str) -> Result<GraphResults, ParseError<LineCol>> {
        query_parser::command(command, &mut graph, &mut chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::attribute::InternalNodeAttribute;

    #[test]
    fn should_fetch_node() {
        // Given
        let mut graph = Graph::default();
        let mut chain = Chain::default();

        let from = insert_new_node(&mut graph, "From");
        let to = insert_new_node(&mut graph, "To");

        insert_new_edge(&mut graph, from.clone(), to.clone(), 50);

        let cmd = format!("fetch node From($id=\"{from}\") join To($weight>\"0\")");

        // When
        let result = query_parser::command(cmd.as_str(), &mut graph, &mut chain);

        // Then
        assert_graph_result(
            result,
            vec![
                ("$name", "From"),
                ("$id", from.as_str()),
                ("$edges", "1"),
                ("To.$id", to.as_str()),
                ("To.$name", "To"),
                ("To.$edges", "0"),
            ],
        );
    }

    #[test]
    fn should_add_node_definition() {
        // Given
        let mut graph = Graph::default();
        let mut chain = Chain::default();
        let cmd = "define node Person(name,premium) with agent (premium=\"true\")";

        // When
        let result = query_parser::command(cmd, &mut graph, &mut chain);

        // Then
        assert_graph_result(result, vec![("name", "*"), ("premium", "*")]);

        assert!(graph.nodes.is_empty());
        assert_eq!(graph.definitions.len(), 1);
        assert!(graph.definitions.contains_key("Person"));

        let conditions = graph.definitions.get("Person").unwrap();
        assert_eq!(*conditions, vec!["name", "premium"]);

        assert_eq!(chain.agent_service.agents.len(), 1);
    }

    #[test]
    fn should_add_node() {
        // Given
        let mut graph = Graph::default();
        let mut chain = Chain::default();
        graph
            .create_definition("Person".to_string(), vec!["name".to_string()])
            .expect("Inserting definition failed");

        let command = "add node Person(name=\"Janne\")";

        // When
        let result = query_parser::command(command, &mut graph, &mut chain);

        // Then
        assert_graph_result(
            result,
            vec![
                (InternalNodeAttribute::ID_ATTRIBUTE, "_"),
                (InternalNodeAttribute::NAME_ATTRIBUTE, "Person"),
                ("name", "Janne"),
                (InternalNodeAttribute::EDGE_COUNT_ATTRIBUTE, "0"),
            ],
        );
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn should_update_node() {
        // Given
        let mut graph = Graph::default();
        let mut chain = Chain::default();
        let identifier = insert_new_node_with_attributes(&mut graph, "Person", vec!["name"]);

        let command = format!("update node Person($id=\"{}\",name=\"Janne\")", identifier);

        // When
        let result = query_parser::command(command.as_str(), &mut graph, &mut chain);

        // Then
        assert_graph_result(
            result,
            vec![
                (InternalNodeAttribute::ID_ATTRIBUTE, identifier.as_str()),
                (InternalNodeAttribute::NAME_ATTRIBUTE, "Person"),
                ("name", "Janne"),
                (InternalNodeAttribute::EDGE_COUNT_ATTRIBUTE, "0"),
            ],
        );
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn should_delete_node() {
        // Given
        let mut graph = Graph::default();
        let mut chain = Chain::default();
        let identifier = insert_new_node(&mut graph, "Person");

        let command = format!("delete node Person($id=\"{}\")", identifier);

        // When
        let result = query_parser::command(command.as_str(), &mut graph, &mut chain);

        // Then
        assert_graph_result(
            result,
            vec![
                (InternalNodeAttribute::ID_ATTRIBUTE, identifier.as_str()),
                (InternalNodeAttribute::NAME_ATTRIBUTE, "Person"),
                (InternalNodeAttribute::EDGE_COUNT_ATTRIBUTE, "0"),
            ],
        );
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn should_add_edge() {
        // Given
        let mut graph = Graph::default();
        let mut chain = Chain::default();
        let from_id = insert_new_node(&mut graph, "From");
        let to_id = insert_new_node(&mut graph, "To");

        let cmd = format!("add connection from From($id=\"{}\") to To($id=\"{}\") with weight 50", from_id, to_id);

        // When
        let result = query_parser::command(cmd.as_str(), &mut graph, &mut chain);

        // Then
        assert_graph_result(
            result,
            vec![
                (InternalNodeAttribute::FROM_ATTRIBUTE, "From"),
                (InternalNodeAttribute::TO_ATTRIBUTE, "To"),
                (InternalNodeAttribute::WEIGHT_ATTRIBUTE, "50"),
            ],
        );
        assert_edge(&mut graph, from_id, to_id, 50);
    }

    #[test]
    fn should_update_edge() {
        // Given
        let mut graph = Graph::default();
        let mut chain = Chain::default();
        let from_id = insert_new_node(&mut graph, "From");
        let to_id = insert_new_node(&mut graph, "To");

        insert_new_edge(&mut graph, from_id.clone(), to_id.clone(), 50);

        let cmd = format!("update connection from From($id=\"{}\") to To($id=\"{}\") with weight 80", from_id, to_id);

        // When
        let result = query_parser::command(cmd.as_str(), &mut graph, &mut chain);

        // Then
        assert_graph_result(
            result,
            vec![
                (InternalNodeAttribute::FROM_ATTRIBUTE, "From"),
                (InternalNodeAttribute::TO_ATTRIBUTE, "To"),
                (InternalNodeAttribute::WEIGHT_ATTRIBUTE, "80"),
            ],
        );
        assert_edge(&mut graph, from_id, to_id, 80);
    }

    #[test]
    fn should_delete_edge() {
        // Given
        let mut graph = Graph::default();
        let mut chain = Chain::default();
        let from_id = insert_new_node(&mut graph, "From");
        let to_id = insert_new_node(&mut graph, "To");

        insert_new_edge(&mut graph, from_id.clone(), to_id.clone(), 50);

        let cmd = format!("delete connection from From($id=\"{}\") to To($id=\"{}\")", from_id, to_id);

        // When
        let result = query_parser::command(cmd.as_str(), &mut graph, &mut chain);

        // Then
        assert_graph_result(
            result,
            vec![
                (InternalNodeAttribute::FROM_ATTRIBUTE, "From"),
                (InternalNodeAttribute::TO_ATTRIBUTE, "To"),
                (InternalNodeAttribute::WEIGHT_ATTRIBUTE, "50"),
            ],
        );

        for (_, node) in &graph.nodes {
            assert!(node.edges.is_empty());
        }
    }

    fn insert_new_node(graph: &mut Graph, name: &str) -> String {
        insert_new_node_with_attributes(graph, name, vec![])
    }

    fn insert_new_node_with_attributes(graph: &mut Graph, name: &str, attributes: Vec<&str>) -> String {
        graph
            .create_definition(name.to_string(), attributes.iter().map(|attribute| attribute.to_string()).collect())
            .expect("Inserting definition failed");

        graph
            .add_node(name.to_string(), FxHashMap::default())
            .unwrap()
            .first()
            .unwrap()
            .get("$id")
            .unwrap()
            .to_string()
    }

    fn insert_new_edge(graph: &mut Graph, from: String, to: String, weight: i8) {
        let mut from_attributes = FxHashMap::default();
        from_attributes.insert("$id".to_string(), from);

        let mut to_attributes = FxHashMap::default();
        to_attributes.insert("$id".to_string(), to);

        assert!(graph
            .add_edge(("From".to_string(), from_attributes), ("To".to_string(), to_attributes), weight)
            .is_ok());
    }

    fn assert_graph_result(result: Result<GraphResults, ParseError<LineCol>>, expected: Vec<(&str, &str)>) {
        assert!(result.is_ok()); // No parsing errors

        let graph_result = result.unwrap();
        assert!(graph_result.is_ok()); // No graph constraint errors

        let items = graph_result.unwrap();
        assert_eq!(items.len(), 1); // Only one result fetched

        let actual = items.first().unwrap();
        expected.clone().into_iter().for_each(|(key, value)| {
            assert!(actual.contains_key(key));
            // For random values we can set value as "_" to check only if present, not its content
            if value != "_" {
                assert_eq!(actual.get(key).unwrap(), value);
            } else {
                assert!(!actual.get(key).unwrap().is_empty());
            }
        });

        // There are some extra keys not sent in expected vector
        assert_eq!(actual.len(), expected.len());
    }

    fn assert_edge(graph: &Graph, from_id: String, to_id: String, weight: i8) {
        for (id, node) in &graph.nodes {
            if *id == format!("{from_id}:From") {
                assert_eq!(node.edges.len(), 1);

                let edge = node.edges.first().unwrap();
                assert_eq!(edge.to_node, "To");
                assert_eq!(edge.to_node_id, to_id);
                assert_eq!(edge.weight, weight);
            } else if *id == format!("{to_id}:To") {
                assert!(node.edges.is_empty())
            } else {
                assert!(false)
            }
        }
    }
}
