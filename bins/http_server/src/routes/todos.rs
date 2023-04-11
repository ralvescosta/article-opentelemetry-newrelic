use crate::controllers;
use actix_web::web::{self, ServiceConfig};
use httpw::server::ServiceConfigs;

pub fn routes() -> impl FnOnce(&mut ServiceConfig) {
    move |cfg: &mut ServiceConfig| {
        cfg.service(
            web::scope("/v1/todos")
                .service(controllers::post)
                .service(controllers::list)
                .service(controllers::get)
                .service(controllers::delete),
        );
    }
}
