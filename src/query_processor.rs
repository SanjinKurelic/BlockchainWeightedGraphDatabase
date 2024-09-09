extern crate peg;

use crate::graph::Graph;
use crate::graph::GraphResults;
use peg::error::ParseError;
use peg::str::LineCol;
use rustc_hash::FxHashMap;

peg::parser! {
    grammar query_parser(graph: &mut Graph) for str {
        pub rule command() -> GraphResults = add_node() / update_node() / delete_node() / add_edge() / update_edge() / delete_edge()

        rule add_node() -> GraphResults = _ "add" _ "node" _ name:name() _ attributes:attributes()? {
            graph.add_node(name.to_string(), attributes.unwrap_or_else(FxHashMap::default))
        }

        rule add_edge() -> GraphResults = _ "add" _ "connection" _ "from" _ from_name:name() _ from_attributes:attributes() _ "to" _ to_name:name() _ to_attributes:attributes() _ "with" _ "weight" _ weight:weight()  {
            graph.add_edge((from_name.to_string(), from_attributes), (to_name.to_string(), to_attributes), weight)
        }

        rule update_node() -> GraphResults = _ "update" _ "node" _ name:name() _ attributes:attributes() {
            graph.update_node(name.to_string(), attributes)
        }

        rule update_edge() -> GraphResults = _ "update" _ "connection" _ "from" _ from_name:name() _ from_attributes:attributes() _ "to" _ to_name:name() _ to_attributes:attributes() _ "with" _ "weight" _ weight:weight()  {
            graph.update_edge((from_name.to_string(), from_attributes), (to_name.to_string(), to_attributes), weight)
        }

        rule delete_node() -> GraphResults = _ "delete" _ "node" _ name:name() _ attributes:attributes() {
            graph.delete_node(name.to_string(), attributes)
        }

        rule delete_edge() -> GraphResults = _ "delete" _ "connection" _ "from" _ from_name:name() _ from_attributes:attributes() _ "to" _ to_name:name() _ to_attributes:attributes() {
            graph.delete_edge((from_name.to_string(), from_attributes), (to_name.to_string(), to_attributes))
        }

        rule attributes() -> FxHashMap<String, String> = "(" attributes:attribute() ** "," ")" {
            attributes.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<FxHashMap<String, String>>()
        }

        rule attribute() -> (&'input str, &'input str) = name:attribute_name() "=" value:attribute_value() { (name, value) }

        rule attribute_name() -> &'input str = $(['a'..='z' | 'A'..='Z' | '0'..='9' | '$' | '*']+)

        rule attribute_value() -> &'input str = "\"" value:__ "\"" { value }

        rule name() -> &'input str = $(['a'..='z' | 'A'..='Z']+)

        rule weight() -> i8 = n:$(['0'..='9']+) { n.parse().unwrap() }

        rule __ -> &'input str = $([^'"']*)

        rule _ -> &'input str = $([' ']*)
    }
}

pub struct QueryProcessor;

impl QueryProcessor {
    pub fn parse_command(graph: &mut Graph, command: &str) -> Result<GraphResults, ParseError<LineCol>> {
        query_parser::command(command, graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_add_node() {
        // Given
        let mut graph = Graph::default();
        let command = "add node Person(name=\"Janne\")";

        // When
        let result = query_parser::command(command, &mut graph);

        // Then
        assert_graph_result(result, vec![("$id", "_"), ("$name", "Person"), ("name", "Janne")]);
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn should_update_node() {
        // Given
        let mut graph = Graph::default();
        let identifier = insert_new_node(&mut graph, "Person");

        let command = format!("update node Person($id=\"{}\",name=\"Janne\")", identifier);

        // When
        let result = query_parser::command(command.as_str(), &mut graph);

        // Then
        assert_graph_result(result, vec![("$id", identifier.as_str()), ("$name", "Person"), ("name", "Janne")]);
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn should_delete_node() {
        // Given
        let mut graph = Graph::default();
        let identifier = insert_new_node(&mut graph, "Person");

        let command = format!("delete node Person($id=\"{}\")", identifier);

        // When
        let result = query_parser::command(command.as_str(), &mut graph);

        // Then
        assert_graph_result(result, vec![("$id", identifier.as_str()), ("$name", "Person")]);
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn should_add_edge() {
        // Given
        let mut graph = Graph::default();
        let from_id = insert_new_node(&mut graph, "From");
        let to_id = insert_new_node(&mut graph, "To");

        let cmd = format!("add connection from From($id=\"{}\") to To($id=\"{}\") with weight 50", from_id, to_id);

        // When
        let result = query_parser::command(cmd.as_str(), &mut graph);

        // Then
        assert_graph_result(result, vec![("$from", "From"), ("$to", "To"), ("$weight", "50")]);
        assert_edge(&mut graph, from_id, to_id, 50);
    }

    #[test]
    fn should_update_edge() {
        // Given
        let mut graph = Graph::default();
        let from_id = insert_new_node(&mut graph, "From");
        let to_id = insert_new_node(&mut graph, "To");

        insert_new_edge(&mut graph, from_id.clone(), to_id.clone(), 50);

        let cmd = format!("update connection from From($id=\"{}\") to To($id=\"{}\") with weight 80", from_id, to_id);

        // When
        let result = query_parser::command(cmd.as_str(), &mut graph);

        // Then
        assert_graph_result(result, vec![("$from", "From"), ("$to", "To"), ("$weight", "80")]);
        assert_edge(&mut graph, from_id, to_id, 80);
    }

    #[test]
    fn should_delete_edge() {
        // Given
        let mut graph = Graph::default();
        let from_id = insert_new_node(&mut graph, "From");
        let to_id = insert_new_node(&mut graph, "To");

        insert_new_edge(&mut graph, from_id.clone(), to_id.clone(), 50);

        let cmd = format!("delete connection from From($id=\"{}\") to To($id=\"{}\")", from_id, to_id);

        // When
        let result = query_parser::command(cmd.as_str(), &mut graph);

        // Then
        assert_graph_result(result, vec![("$from", "From"), ("$to", "To"), ("$weight", "50")]);

        for (_, node) in &graph.nodes {
            assert!(node.edges.is_empty());
        }
    }

    fn insert_new_node(graph: &mut Graph, name: &str) -> String {
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
