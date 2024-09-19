use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[derive(Serialize, Deserialize, Clone)]
pub enum DatabaseError {
    AttributeNotAllowed(String),
    AttributeIsRequired(String),
    EdgeAlreadyExists(String, String),
    EdgeNotFound(String, String),
    NodeAlreadyExists(String),
    NodeNotDefined(String),
    NodeNotFound(String, String),
}

fn error_message(error: &DatabaseError, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    match error {
        DatabaseError::AttributeNotAllowed(name) => {
            write!(
                formatter,
                "Attribute {name} is not allowed. It's either not defined or used for internal purposes."
            )
        }
        DatabaseError::AttributeIsRequired(name) => {
            write!(formatter, "Attribute {name} is required.")
        }
        DatabaseError::EdgeAlreadyExists(from, to) => {
            write!(formatter, "Edge from node {from} to node {to} already exists.")
        }
        DatabaseError::EdgeNotFound(from, to) => {
            write!(formatter, "Edge from node {from} to node {to} was not found.")
        }
        DatabaseError::NodeAlreadyExists(name) => {
            write!(formatter, "Node definition for name {name} already exists.")
        }
        DatabaseError::NodeNotDefined(name) => {
            write!(
                formatter,
                "Node definition for name {name} not found. Please define node before adding items."
            )
        }
        DatabaseError::NodeNotFound(name, identifier) => {
            write!(formatter, "Node with given name {name} and identifier {identifier} was not found.")
        }
    }
}

impl Display for DatabaseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        error_message(self, formatter)
    }
}

impl Debug for DatabaseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        error_message(self, formatter)
    }
}
