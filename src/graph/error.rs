use std::fmt::{Debug, Formatter};

pub enum DatabaseError {
    AttributeNotAllowed(String),
    AttributeIsRequired(String),
    EdgeAlreadyExists(String, String),
    EdgeNotFound(String, String),
    NodeAlreadyExists(String),
    NodeNotDefined(String),
    NodeNotFound(String, String),
}

impl Debug for DatabaseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
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
}
