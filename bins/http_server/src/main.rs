mod controllers;
mod openapi;
mod routes;
mod viewmodels;

use configs::{Configs, Empty};
use configs_builder::ConfigBuilder;
use httpw::server::HTTPServer;
use openapi::ApiDoc;
use routes as todos_routes;
use std::error::Error;
use utoipa::OpenApi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = default_setup().await?;

    let doc = ApiDoc::openapi();

    let server = HTTPServer::new(&cfg.app)
        .register(todos_routes::routes())
        .openapi(&doc);

    server.start().await?;

    Ok(())
}

async fn default_setup<'cfg>() -> Result<Configs<Empty>, Box<dyn Error>> {
    let cfg = ConfigBuilder::new()
        .use_aws_secret_manager()
        .otlp()
        .auth0()
        .build::<Empty>()
        .await?;

    traces::otlp::setup(&cfg)?;
    metrics::otlp::setup(&cfg)?;

    Ok(cfg)
}
