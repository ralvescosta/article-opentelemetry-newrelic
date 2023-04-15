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
use opentelemetry::{global, Context};
use shared::{
    amqp::{EXCHANGE, ROUTING_KEY},
    models::todo::TodoCreatedMessage,
};
use sql_pool::postgres::conn_pool;
use std::{error::Error, sync::Arc};
use tracing::error;

pub const QUEUE: &str = "simple-queue";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = default_setup().await?;

    let (conn, channel, queue) = amqp_setup(&cfg).await?;
    let db_conn = Arc::new(conn_pool(&cfg.postgres)?);

    let handler = SimpleConsumer::new();

    let dispatcher =
        AmqpDispatcher::new(channel).register(&queue, &TodoCreatedMessage::default(), handler);

    let health_readiness = HealthReadinessServer::new(&cfg.health_readiness)
        .rabbitmq(conn)
        .postgres(db_conn.clone());

    declare_health_meter()?;

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
        .auth0()
        .otlp()
        .build::<Empty>()
        .await?;

    traces::otlp::setup(&configs)?;
    metrics::otlp::setup(&configs)?;

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

fn declare_health_meter() -> Result<(), Box<dyn Error>> {
    let meter = global::meter("http-meter-server");
    let health_counter = meter
        .i64_observable_up_down_counter("consumer.health")
        .with_description("AMQP Consumer Health")
        .init();
    let callback = move |ctx: &Context| {
        health_counter.observe(ctx, 1, &[]);
    };
    match meter.register_callback(callback) {
        Err(err) => {
            error!(error = err.to_string(), "error to register health counter");
            Err(err)
        }
        _ => Ok(()),
    }?;

    Ok(())
}
