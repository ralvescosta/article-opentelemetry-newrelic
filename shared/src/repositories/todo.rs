use crate::models::todo::{CreateTodo, Todo};
use async_trait::async_trait;
use opentelemetry::Context;

#[async_trait]
pub trait TodoRepository: Send + Sync + 'static {
    async fn create(&self, ctx: &Context, todo: &CreateTodo) -> Result<Todo, String>;
    async fn get_by_id(&self, ctx: &Context, id: &str) -> Result<Option<Todo>, String>;
    async fn list_paginated(
        &self,
        ctx: Context,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Todo>, String>;
}
