use crate::viewmodels::{CreateTodoRequest, TodoResponse};
use actix_web::{
    delete, get,
    http::StatusCode,
    post,
    web::{Data, Json, Path},
    HttpRequest, HttpResponse, Responder, ResponseError,
};
use amqp::publisher::{Payload, Publisher};
use http_components::{
    extractors::JwtAuthenticateExtractor, middlewares::otel::HTTPExtractor, viewmodels::HTTPError,
};
use opentelemetry::global;
use shared::{
    amqp::{EXCHANGE, ROUTING_KEY},
    models::todo::TodoCreatedMessage,
    repositories::TodoRepository,
};
use std::sync::Arc;
use tracing::error;

/// Request to create a new ToDo.
///
/// If the request was registered correctly this endpoint will return 201 Accepted and 4xx/5xx if some error occur.
///
#[utoipa::path(
    post,
    path = "",
    context_path = "/v1/todos",
    tag = "todos",
    request_body = CreateTodoRequest,
    responses(
        (status = 202, description = "Todo requested successfully", body = ThingResponse),
        (status = 400, description = "Bad request", body = HTTPError),
        (status = 401, description = "Unauthorized", body = HTTPError),
        (status = 403, description = "Forbidden", body = HTTPError),
        (status = 500, description = "Internal error", body = HTTPError)
    ),
    security()
)]
#[post("")]
pub async fn post(
    req: HttpRequest,
    todo: Json<CreateTodoRequest>,
    repo: Data<Arc<dyn TodoRepository>>,
    publisher: Data<Arc<dyn Publisher>>,
) -> Result<impl Responder, impl ResponseError> {
    let ctx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HTTPExtractor::new(req.headers()))
    });

    let created = match repo.create(&ctx, &todo.0.into()).await {
        Err(err) => {
            error!(error = err.to_string(), "error to create todo");
            Err(HTTPError {
                status_code: StatusCode::BAD_REQUEST.into(),
                message: "error to create todo".to_owned(),
                details: "error to create todo".to_owned(),
            })
        }
        Ok(t) => Ok(t),
    }?;

    let payload = match Payload::new(&TodoCreatedMessage::from(&created)) {
        Err(err) => {
            error!(error = err.to_string(), "error to create todo");
            Err(HTTPError {
                status_code: StatusCode::BAD_REQUEST.into(),
                message: "error to create todo".to_owned(),
                details: "error to create todo".to_owned(),
            })
        }
        Ok(p) => Ok(p),
    }?;

    match publisher
        .publish(&ctx, EXCHANGE, ROUTING_KEY, &payload, None)
        .await
    {
        Err(err) => {
            error!(error = err.to_string(), "error to create todo");
            Err(HTTPError {
                status_code: StatusCode::BAD_REQUEST.into(),
                message: "error to create todo".to_owned(),
                details: "error to create todo".to_owned(),
            })
        }
        _ => Ok(HttpResponse::Ok().json(TodoResponse::from(&created))),
    }
}

/// Request to get all ToDo's that was created.
///
/// If the request was process correctly this endpoint will return 200 Ok and 4xx/5xx if some error occur.
///
#[utoipa::path(
    get,
    path = "",
    context_path = "/v1/todos",
    tag = "todos",
    responses(
        (status = 200, description = "Success", body = Vec<ThingResponse>),
        (status = 400, description = "Bad request", body = HTTPError),
        (status = 401, description = "Unauthorized", body = HTTPError),
        (status = 403, description = "Forbidden", body = HTTPError),
        (status = 500, description = "Internal error", body = HTTPError)
    ),
    security(
        ("auth" = [])
    )
)]
#[get("")]
pub async fn list(
    req: HttpRequest,
    _: JwtAuthenticateExtractor,
    repo: Data<Arc<dyn TodoRepository>>,
) -> Result<impl Responder, impl ResponseError> {
    let ctx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HTTPExtractor::new(req.headers()))
    });

    match repo.list_paginated(&ctx, 10, 0).await {
        Err(err) => {
            error!(error = err.to_string(), "error to list todo");
            Err(HTTPError {
                status_code: StatusCode::BAD_REQUEST.into(),
                message: "error to list todo".to_owned(),
                details: "error to list todo".to_owned(),
            })
        }
        Ok(todos) => Ok(HttpResponse::Ok().json(
            todos
                .iter()
                .map(|e| TodoResponse::from(e))
                .collect::<Vec<TodoResponse>>(),
        )),
    }
}

/// Request to get a specific ToDo by ID.
///
/// If the request was process correctly this endpoint will return 200 Ok and 4xx/5xx if some error occur.
///
#[utoipa::path(
    get,
    path = "/{id}",
    context_path = "/v1/todos",
    tag = "todos",
    responses(
        (status = 200, description = "Success", body = ThingResponse),
        (status = 400, description = "Bad request", body = HTTPError),
        (status = 401, description = "Unauthorized", body = HTTPError),
        (status = 403, description = "Forbidden", body = HTTPError),
        (status = 500, description = "Internal error", body = HTTPError)
    ),
    security(
        ("auth" = [])
    )
)]
#[get("/{id}")]
pub async fn get(
    req: HttpRequest,
    path: Path<(String,)>,
    _: JwtAuthenticateExtractor,
    repo: Data<Arc<dyn TodoRepository>>,
) -> Result<impl Responder, impl ResponseError> {
    let ctx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HTTPExtractor::new(req.headers()))
    });

    let (id,) = path.into_inner();

    match repo.get_by_id(&ctx, &id).await {
        Err(err) => {
            error!(error = err.to_string(), "error to get todo");
            Err(HTTPError {
                status_code: StatusCode::BAD_REQUEST.into(),
                message: "error to get todo".to_owned(),
                details: "error to get todo".to_owned(),
            })
        }
        Ok(todo) => {
            if let Some(t) = todo {
                return Ok(HttpResponse::Ok().json(TodoResponse::from(&t)));
            }

            Ok(HttpResponse::Ok().finish())
        }
    }
}

/// Request to delete a specific ToDo by ID.
///
/// If the request was process correctly this endpoint will return 200 Ok and 4xx/5xx if some error occur.
///
#[utoipa::path(
    delete,
    path = "/{id}",
    context_path = "/v1/todos",
    tag = "todos",
    responses(
        (status = 200, description = "Deleted"),
        (status = 400, description = "Bad request", body = HTTPError),
        (status = 401, description = "Unauthorized", body = HTTPError),
        (status = 403, description = "Forbidden", body = HTTPError),
        (status = 500, description = "Internal error", body = HTTPError)
    ),
    security(
        ("auth" = [])
    )
)]
#[delete("/{id}")]
pub async fn delete(
    req: HttpRequest,
    path: Path<(String,)>,
    _: JwtAuthenticateExtractor,
    repo: Data<Arc<dyn TodoRepository>>,
) -> Result<impl Responder, impl ResponseError> {
    let ctx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HTTPExtractor::new(req.headers()))
    });

    let (id,) = path.into_inner();

    match repo.delete(&ctx, &id).await {
        Err(err) => {
            error!(error = err.to_string(), "error to create todo");
            Err(HTTPError {
                status_code: StatusCode::BAD_REQUEST.into(),
                message: "error to create todo".to_owned(),
                details: "error to create todo".to_owned(),
            })
        }
        _ => Ok(HttpResponse::Ok().finish()),
    }
}
