use query_processor::QueryProcessor;

mod query_processor;
mod graph;
mod chain;

fn main() {
    let mut graph = graph::Graph::default();

    let result = QueryProcessor::parse_command(&mut graph, "add node Person(name=\"Janne\")");

    result.is_ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should() {
        assert!(true)
    }
}