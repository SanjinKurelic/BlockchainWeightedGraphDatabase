use crate::chain::Chain;
use crate::graph::attribute::InternalNodeAttribute;
use crate::graph::error::DatabaseError;
use crate::graph::node::Node;
use crate::graph::Graph;
use crate::query_processor::QueryProcessor;
use rand::Rng;
use rustc_hash::FxHashMap;
use std::env;

pub struct Bootstrap;

impl Bootstrap {
    pub fn init(mut graph: &mut Graph, mut chain: &mut Chain) -> Result<(), DatabaseError> {
        let username: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let user = format!("add node User(name=\"{username}\",premium=\"true\",key=\"{}\")",  chain.wallet.get_public_key().clone());

        let commands = vec![
            "define node User(name,premium,key) with agent(premium=\"true\")",
            "define node Playlist(name)",
            "add node Playlist(name=\"Party Mix\")",
            user.as_str(),
        ];

        let mut commands_iter = commands.iter().peekable();

        while let Some(command) = commands_iter.next() {
            let result = QueryProcessor::parse_command(&mut graph, &mut chain, command)
                .expect("BOOTSTRAP :: Failed to parse command")
                .expect("BOOTSTRAP :: Failed to parse command")
                .first()
                .expect("BOOTSTRAP :: Failed to parse command")
                .clone();

            println!("BOOTSTRAP :: {command} :: {:#?}", result);
        }

        let (_, users) = argmap::parse(env::args());
        for n in 1..4 {
            if users.contains_key(format!("username{n}").as_str()) && users.contains_key(format!("key{n}").as_str()) {
                Self::insert_node(
                    &mut graph,
                    users.get(format!("username{n}").as_str()).unwrap().first().unwrap(),
                    users.get(format!("key{n}").as_str()).unwrap().first().unwrap(),
                );
            }
        }

        Ok(())
    }

    fn insert_node(graph: &mut Graph, username: &String, key: &String) {
        let mut attributes = FxHashMap::default();

        attributes.insert(InternalNodeAttribute::ID_ATTRIBUTE.to_string(), username.to_string());
        attributes.insert(InternalNodeAttribute::NAME_ATTRIBUTE.to_string(), "User".to_string());
        attributes.insert(InternalNodeAttribute::EDGE_COUNT_ATTRIBUTE.to_string(), "0".to_string());
        attributes.insert("premium".to_string(), "true".to_string());
        attributes.insert("key".to_string(), key.to_string());

        graph.nodes.insert(format!("{username}:User"), Node::new(attributes, vec![]));
    }
}
