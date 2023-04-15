use async_trait::async_trait;
use chrono::{DateTime, Utc};
use deadpool_postgres::{
    tokio_postgres::{types::ToSql, Row},
    Object, Pool,
};
use opentelemetry::{
    global::{self, BoxedSpan, BoxedTracer},
    trace::{Span, Status, Tracer},
    Context, KeyValue,
};
use postgres::Statement;
use shared::{
    models::todo::{CreateTodo, Todo},
    repositories::TodoRepository,
};
use std::{borrow::Cow, sync::Arc};
use tracing::error;
use uuid::Uuid;

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
            .await?
            .unwrap();

        Ok(Todo {
            id: row.get::<usize, Uuid>(0).to_string(),
            name: row.get(1),
            description: row.get(2),
            created_at: row.get::<usize, DateTime<Utc>>(3).to_rfc3339(),
            updated_at: row.get::<usize, DateTime<Utc>>(4).to_rfc3339(),
            deleted_at: None,
        })
    }

    async fn get_by_id(&self, ctx: &Context, id: &str) -> Result<Option<Todo>, String> {
        let query = "SELECT * FROM todos WHERE id = $1 AND deleted_at IS NULL";

        let uid: Uuid = match Uuid::parse_str(id) {
            Err(err) => {
                error!(error = err.to_string(), "invalid uuid");
                Err(String::from("invalid uuid"))
            }
            Ok(u) => Ok(u),
        }?;

        match self.query_one(ctx, query.to_owned(), &[&uid]).await? {
            None => Ok(None),
            Some(row) => Ok(Some(Todo {
                id: row.get::<usize, Uuid>(0).to_string(),
                name: row.get(1),
                description: row.get(2),
                created_at: row.get::<usize, DateTime<Utc>>(3).to_rfc3339(),
                updated_at: row.get::<usize, DateTime<Utc>>(4).to_rfc3339(),
                deleted_at: None,
            })),
        }
    }

    async fn list_paginated(
        &self,
        ctx: &Context,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Todo>, String> {
        let query = "SELECT * FROM todos WHERE deleted_at IS NOT NULL OFFSET = $1 LIMIT = $2";

        let rows = self
            .query(&ctx, query.to_owned(), &[&offset, &limit])
            .await?;

        Ok(rows
            .iter()
            .map(|row| Todo {
                id: row.get::<usize, Uuid>(0).to_string(),
                name: row.get(1),
                description: row.get(2),
                created_at: row.get::<usize, DateTime<Utc>>(3).to_rfc3339(),
                updated_at: row.get::<usize, DateTime<Utc>>(4).to_rfc3339(),
                deleted_at: None,
            })
            .collect::<Vec<Todo>>())
    }

    async fn delete(&self, ctx: &Context, id: &str) -> Result<(), String> {
        let query = "UPDATE todos SET deleted_at = $1 WHERE id = $2";

        let mut span = self.tracer.start_with_context("query_one", ctx);
        span.set_attributes(vec![KeyValue::new("sql.query", query.clone())]);

        let uid: Uuid = match Uuid::parse_str(id) {
            Err(err) => {
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("invalid uuid"),
                });

                error!(error = err.to_string(), "invalid uuid");
                Err(String::from("invalid uuid"))
            }
            Ok(u) => Ok(u),
        }?;

        let conn = self.get_conn(&mut span).await?;
        let statement = self.statement(&conn, &query, &mut span).await?;

        match conn.query(&statement, &[&"", &uid]).await {
            Err(err) => {
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error to execute query"),
                });

                error!(error = err.to_string(), "error to execute query");
                Err(String::from("error to execute query"))
            }
            _ => Ok(()),
        }
    }
}

impl TodoRepositoryImpl {
    async fn query_one(
        &self,
        ctx: &Context,
        query: String,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<Row>, String> {
        let mut span = self.tracer.start_with_context("query_one", ctx);
        span.set_attributes(vec![KeyValue::new("sql.query", query.clone())]);

        let conn = self.get_conn(&mut span).await?;
        let statement = self.statement(&conn, &query, &mut span).await?;

        match conn.query_one(&statement, params).await {
            Err(err) => {
                if err.code().is_none() {
                    return Ok(None);
                }

                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error to execute query"),
                });

                error!(error = err.to_string(), "error to execute query");
                Err(String::from("error to execute query"))
            }
            Ok(r) => Ok(Some(r)),
        }
    }

    async fn query(
        &self,
        ctx: &Context,
        query: String,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, String> {
        let mut span = self.tracer.start_with_context("query", ctx);
        span.set_attributes(vec![KeyValue::new("sql.query", query.clone())]);

        let conn = self.get_conn(&mut span).await?;
        let statement = self.statement(&conn, &query, &mut span).await?;

        match conn.query(&statement, params).await {
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

    async fn get_conn(&self, span: &mut BoxedSpan) -> Result<Object, String> {
        match self.pool.get().await {
            Err(err) => {
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error to get connection from poll"),
                });

                error!(error = err.to_string(), "error to get connection from poll");
                Err(String::from("error to get connection from poll"))
            }
            Ok(c) => Ok(c),
        }
    }

    async fn statement(
        &self,
        conn: &Object,
        query: &str,
        span: &mut BoxedSpan,
    ) -> Result<Statement, String> {
        match conn.prepare(query).await {
            Err(err) => {
                span.record_error(&err);
                span.set_status(Status::Error {
                    description: Cow::from("error to prepare statement"),
                });

                error!(error = err.to_string(), "error to prepare statement");
                Err(String::from("error to prepare statement"))
            }
            Ok(s) => Ok(s),
        }
    }
}
