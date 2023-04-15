use amqp::{dispatcher::ConsumerHandler, errors::AmqpError};
use async_trait::async_trait;
use opentelemetry::{
    global::{self, BoxedTracer},
    metrics::Counter,
    trace::{Span, Status, Tracer},
    Context,
};
use shared::models::todo::TodoCreatedMessage;
use std::{borrow::Cow, sync::Arc};
use tracing::{error, info};
pub struct SimpleConsumer {
    tracer: BoxedTracer,
    messages_processed: Counter<u64>,
    messages_failed: Counter<u64>,
}

impl SimpleConsumer {
    pub fn new() -> Arc<SimpleConsumer> {
        let meter = global::meter("consumers-handler-meter");
        let tracer = global::tracer("consumers-handler");

        let messages_processed = meter
            .u64_counter("consumers.messages.processed")
            .with_description("Consumer Messages Processed Successfully")
            .init();

        let messages_failed = meter
            .u64_counter("consumers.messages.failed")
            .with_description("Consumer Messages Failed to Processed")
            .init();

        Arc::new(SimpleConsumer {
            tracer,
            messages_processed,
            messages_failed,
        })
    }
}

#[async_trait]
impl ConsumerHandler for SimpleConsumer {
    async fn exec(&self, ctx: &Context, data: &[u8]) -> Result<(), AmqpError> {
        let mut span = self
            .tracer
            .start_with_context("simple_consumer_handler", ctx);

        let received = match TodoCreatedMessage::try_from(data) {
            Err(err) => {
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("failure to serialize message"),
                });

                error!(error = err.to_string(), "failure to serialize message");
                self.messages_failed.add(ctx, 1, &[]);

                Err(err)
            }
            Ok(r) => Ok(r),
        }?;

        self.messages_processed.add(ctx, 1, &[]);

        info!("amqp message received {:?}", received);

        Ok(())
    }
}
