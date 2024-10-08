use crate::bootstrap::Bootstrap;
use crate::chain::Chain;
use crate::graph::Graph;
use crate::protocol::Protocol;
use query_processor::QueryProcessor;
use tokio::{io, io::AsyncBufReadExt, select};

mod bootstrap;
mod chain;
mod graph;
mod protocol;
mod query_processor;

#[tokio::main]
async fn main() {
    let mut graph = Graph::default();
    let mut chain = Chain::default();

    let mut protocol = Protocol::init().map_err(|error| eprintln!("{error}")).unwrap();

    let mut input = io::BufReader::new(io::stdin()).lines();

    // Initialization for testing
    if let Err(error) = Bootstrap::init(&mut graph, &mut chain) {
        eprintln!("{error}");
    }

    loop {
        select! {
            Ok(Some(line)) = input.next_line() => {
                match QueryProcessor::parse_command(&mut graph, &mut chain, &line) {
                    Err(error) => eprintln!("{error}"),
                    Ok(result) => match result {
                        Ok(items) => {
                                match serde_json::to_string(&items) {
                                    Ok(json) => println!("{json}"),
                                    Err(error) => eprintln!("{error}"),
                                }
                            }
                        Err(error) => eprintln!("{error}"),
                    }
                }
            },
            event = protocol.fetch_network_event() => {
                match protocol.handle_network_event(&mut chain, event) {
                    Err(error) => eprintln!("{error}"),
                    Ok(message) =>if message != "NOP" { println!("{message}") },
                }
            },
        }

        if let Err(error) = protocol.publish_changes(&chain) {
            eprintln!("{error}");
        }
    }
}
