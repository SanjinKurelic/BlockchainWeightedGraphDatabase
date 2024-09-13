use std::fmt::Display;

pub enum ProtocolError {
    NetworkError(String),
    PublishingError(String),
}

impl Display for ProtocolError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::NetworkError(error) => {
                write!(formatter, "There was an network issue: {error}")
            }
            ProtocolError::PublishingError(error) => {
                write!(formatter, "Error while publishing to the topic: {error}")
            }
        }
    }
}
