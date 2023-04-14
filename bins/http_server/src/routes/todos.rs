use crate::controllers;
use actix_web::web::{self, ServiceConfig};
use http_components::CustomServiceConfigure;

pub fn routes() -> CustomServiceConfigure {
    CustomServiceConfigure::new(move |cfg: &mut ServiceConfig| {
        cfg.service(
            web::scope("/v1/todos")
                .service(controllers::post)
                .service(controllers::list)
                .service(controllers::get)
                .service(controllers::delete),
        );
    })
}
