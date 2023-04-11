use crate::controllers;
use actix_web::web::{self, ServiceConfig};

pub fn routes() -> impl FnMut(&mut web::ServiceConfig) + Send + Sync + 'static {
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
