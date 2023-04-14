mod controllers;
mod openapi;
mod routes;
mod viewmodels;

use actix_web::web::{Data, ServiceConfig};
use amqp::{
    channel::new_amqp_channel,
    publisher::{AmqpPublisher, Publisher},
};
use configs::{Configs, Empty};
use configs_builder::ConfigBuilder;
use deadpool_postgres::Pool;
use health_readiness::HealthReadinessServiceImpl;
use http_components::CustomServiceConfigure;
use httpw::server::HTTPServer;
use infra::repositories::TodoRepositoryImpl;
use lapin::Channel;
use openapi::ApiDoc;
use opentelemetry::{global, Context};
use routes as todos_routes;
use shared::repositories::TodoRepository;
use sql_pool::postgres::conn_pool;
use std::{error::Error, sync::Arc};
use tracing::error;
use utoipa::OpenApi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = default_setup().await?;

    let (connection, channel) = new_amqp_channel(&cfg).await?;
    let db_conn = Arc::new(conn_pool(&cfg.postgres)?);

    let health_checker = HealthReadinessServiceImpl::default()
        .amqp(connection.clone())
        .postgres(db_conn.clone());

    let doc = ApiDoc::openapi();
    let server = HTTPServer::new(&cfg.app)
        .custom_configure(container(channel.clone(), db_conn.clone()))
        .custom_configure(todos_routes::routes())
        .health_check(Arc::new(health_checker))
        .openapi(&doc);

    declare_health_meter()?;

    server.start().await?;

    Ok(())
}

async fn default_setup<'cfg>() -> Result<Configs<Empty>, Box<dyn Error>> {
    let cfg = ConfigBuilder::new()
        .use_aws_secret_manager()
        .otlp()
        .auth0()
        .amqp()
        .postgres()
        .build::<Empty>()
        .await?;

    traces::otlp::setup(&cfg)?;
    metrics::otlp::setup(&cfg)?;

    Ok(cfg)
}

fn container(channel: Arc<Channel>, db_pool: Arc<Pool>) -> CustomServiceConfigure {
    CustomServiceConfigure::new(move |cfg: &mut ServiceConfig| {
        let publisher = AmqpPublisher::new(channel.clone());
        let repository = TodoRepositoryImpl::new(db_pool.clone());

        cfg.app_data(Data::<Arc<dyn Publisher>>::new(publisher));
        cfg.app_data(Data::<Arc<dyn TodoRepository>>::new(repository));
    })
}

fn declare_health_meter() -> Result<(), Box<dyn Error>> {
    let meter = global::meter("http-server-meter");
    let health_counter = meter
        .u64_observable_counter("http.server.health")
        .with_description("HTTP Server Health Counter")
        .init();

    let callback = move |ctx: &Context| {
        health_counter.observe(ctx, 10, &[]);
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
