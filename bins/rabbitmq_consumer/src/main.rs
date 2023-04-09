mod consumers;

use amqp::{
    channel,
    dispatcher::{AmqpDispatcher, Dispatcher},
    exchange::ExchangeDefinition,
    queue::{QueueBinding, QueueDefinition},
    topology::{AmqpTopology, Topology},
};
use configs::{Configs, Empty};
use configs_builder::ConfigBuilder;
use consumers::SimpleConsumer;
use health_readiness::HealthReadinessServer;
use lapin::{Channel, Connection};
use shared::viewmodels::SimpleAmqpMessage;
use std::{error::Error, sync::Arc};
use tracing::error;

pub const QUEUE: &str = "simple-queue";
pub const EXCHANGE: &str = "simple-exchange";
pub const ROUTING_KEY: &str = "simple-exchange-key";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = default_setup().await?;

    let (conn, channel, queue) = amqp_setup(&cfg).await?;

    let handler = SimpleConsumer::new();

    let dispatcher =
        AmqpDispatcher::new(channel).register(&queue, &SimpleAmqpMessage::default(), handler);

    let health_readiness = HealthReadinessServer::new(&cfg.health_readiness).rabbitmq(conn);

    match tokio::join!(health_readiness.run(), dispatcher.consume_blocking()) {
        (Err(e), _) => {
            error!(error = e.to_string(), "error");
            panic!("{:?}", e)
        }
        (Ok(_), errors) => {
            for err in errors {
                if err.is_err() {
                    error!("error");
                    panic!("{:?}", err)
                }
            }
        }
    }

    Ok(())
}

async fn default_setup() -> Result<Configs<Empty>, Box<dyn Error>> {
    let configs = ConfigBuilder::new()
        .amqp()
        .postgres()
        .otlp()
        .build::<Empty>()
        .await?;

    traces::otlp::setup(&configs)?;

    Ok(configs)
}

async fn amqp_setup(
    cfg: &Configs<Empty>,
) -> Result<(Arc<Connection>, Arc<Channel>, QueueDefinition), Box<dyn Error>> {
    let (conn, channel) = channel::new_amqp_channel(cfg).await?;

    let queue = QueueDefinition::new(QUEUE)
        .durable()
        .with_dlq()
        .with_retry(18000, 3);

    AmqpTopology::new(channel.clone())
        .exchange(&ExchangeDefinition::new(EXCHANGE).direct().durable())
        .queue(&queue)
        .queue_binding(
            &QueueBinding::new(QUEUE)
                .exchange(EXCHANGE)
                .routing_key(ROUTING_KEY),
        )
        .install()
        .await?;

    Ok((conn, channel, queue))
}
