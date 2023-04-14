use async_trait::async_trait;
use deadpool_postgres::{
    tokio_postgres::{types::ToSql, Row},
    Pool,
};
use opentelemetry::{
    global::{self, BoxedTracer},
    trace::{Span, Status, Tracer},
    Context, KeyValue,
};
use shared::{
    models::todo::{CreateTodo, Todo},
    repositories::TodoRepository,
};
use std::{borrow::Cow, sync::Arc};
use tracing::error;

pub struct TodoRepositoryImpl {
    tracer: BoxedTracer,
    pool: Arc<Pool>,
}

impl TodoRepositoryImpl {
    pub fn new(pool: Arc<Pool>) -> Arc<TodoRepositoryImpl> {
        let tracer = global::tracer("todo-repository");

        Arc::new(TodoRepositoryImpl { tracer, pool })
    }
}

#[async_trait]
impl TodoRepository for TodoRepositoryImpl {
    async fn create(&self, ctx: &Context, todo: &CreateTodo) -> Result<Todo, String> {
        let query = "INSERT INTO todos (name, description) values ($1, $2) RETURNING *";

        let row = self
            .query_one(ctx, query.to_owned(), &[&todo.name, &todo.description])
            .await?;

        Ok(Todo {
            id: row.get(0),
            name: row.get(1),
            description: row.get(2),
            created_at: row.get(3),
            updated_at: row.get(4),
            deleted_at: None,
        })
    }

    async fn get_by_id(&self, ctx: &Context, id: &str) -> Result<Option<Todo>, String> {
        let query = "SELECT * FROM todos WHERE id = $1 and deleted_at IS NOT NULL";

        let row = self.query_one(ctx, query.to_owned(), &[&id]).await?;

        Ok(Some(Todo {
            id: row.get(0),
            name: row.get(1),
            description: row.get(2),
            created_at: row.get(3),
            updated_at: row.get(4),
            deleted_at: None,
        }))
    }

    async fn list_paginated(
        &self,
        ctx: Context,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Todo>, String> {
        todo!()
    }
}

impl TodoRepositoryImpl {
    async fn query_one(
        &self,
        ctx: &Context,
        query: String,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Row, String> {
        let mut span = self.tracer.start_with_context("query_one", ctx);

        let conn = match self.pool.get().await {
            Err(err) => {
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error to get connection from poll"),
                });

                error!(error = err.to_string(), "error to get connection from poll");
                Err(String::from("error to get connection from poll"))
            }
            Ok(c) => Ok(c),
        }?;

        span.set_attributes(vec![KeyValue::new("sql.query", query.clone())]);

        let statement = match conn.prepare(&query).await {
            Err(err) => {
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error to prepare statement"),
                });

                error!(error = err.to_string(), "error to prepare statement");
                Err(String::from("error to prepare statement"))
            }
            Ok(s) => Ok(s),
        }?;

        match conn.query_one(&statement, params).await {
            Err(err) => {
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error to execute query"),
                });

                error!(error = err.to_string(), "error to execute query");
                Err(String::from("error to execute query"))
            }
            Ok(r) => Ok(r),
        }
    }
}
