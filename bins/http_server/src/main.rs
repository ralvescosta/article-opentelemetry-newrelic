mod controllers;
mod openapi;
mod routes;
mod viewmodels;

use actix_web::web::{Data, ServiceConfig};
use amqp::channel::new_amqp_channel;
use configs::{Configs, Empty};
use configs_builder::ConfigBuilder;
use httpw::server::{HTTPServer, ServiceConfigs};
use infra::repositories::TodoRepositoryImpl;
use lapin::Channel;
use openapi::ApiDoc;
use routes as todos_routes;
use shared::repositories::TodoRepository;
use std::{error::Error, sync::Arc};
use utoipa::OpenApi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = default_setup().await?;

    let (_, channel) = new_amqp_channel(&cfg).await?;

    let container_args = ContainerArgs {
        channel: channel.clone(),
    };
    let service_config_fn = move |cfg: &mut ServiceConfig| {
        container(cfg, &container_args);
    };

    let doc = ApiDoc::openapi();
    let server = HTTPServer::new(&cfg.app)
        // .register(service_config_fn)
        .register(todos_routes::routes())
        .openapi(&doc);

    server.start().await?;

    Ok(())
}

struct ContainerArgs {
    channel: Arc<Channel>,
}

fn container(cfg: &mut ServiceConfig, args: &ContainerArgs) {
    let ch = args.channel.clone();
    cfg.app_data(Data::<Arc<dyn TodoRepository>>::new(
        TodoRepositoryImpl::new(),
    ));
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
