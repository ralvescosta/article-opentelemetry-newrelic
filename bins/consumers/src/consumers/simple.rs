use amqp::{dispatcher::ConsumerHandler, errors::AmqpError};
use async_trait::async_trait;
use opentelemetry::Context;
use shared::models::todo::TodoCreatedMessage;
use std::sync::Arc;
use tracing::info;
pub struct SimpleConsumer {}

impl SimpleConsumer {
    pub fn new() -> Arc<SimpleConsumer> {
        Arc::new(SimpleConsumer {})
    }
}

#[async_trait]
impl ConsumerHandler for SimpleConsumer {
    async fn exec(&self, _ctx: &Context, data: &[u8]) -> Result<(), AmqpError> {
        let received = TodoCreatedMessage::try_from(data)?;

        info!("amqp message received {:?}", received);

        Ok(())
    }
}
