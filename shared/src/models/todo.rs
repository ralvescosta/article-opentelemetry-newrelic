use amqp::errors::AmqpError;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tracing::error;

pub struct CreateTodo {
    pub name: String,
    pub description: String,
}

#[derive(Default)]
pub struct Todo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TodoCreatedMessage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
}

impl Display for TodoCreatedMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SimpleAmqpMessage")
    }
}

impl From<&Todo> for TodoCreatedMessage {
    fn from(value: &Todo) -> Self {
        TodoCreatedMessage {
            id: value.id.clone(),
            name: value.name.clone(),
            description: value.description.clone(),
            created_at: value.created_at.clone(),
        }
    }
}

impl TryFrom<&[u8]> for TodoCreatedMessage {
    type Error = AmqpError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match serde_json::from_slice::<TodoCreatedMessage>(value) {
            Ok(v) => Ok(v),
            Err(err) => {
                error!(
                    error = err.to_string(),
                    payload = format!("{:?}", value),
                    "parsing error"
                );
                Err(AmqpError::AckMessageDeserializationError(err.to_string()))
            }
        }
    }
}
