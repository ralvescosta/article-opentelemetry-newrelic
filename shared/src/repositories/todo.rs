use crate::models::todo::{CreateTodo, Todo};
use async_trait::async_trait;
use opentelemetry::Context;

#[async_trait]
pub trait TodoRepository: Send + Sync + 'static {
    async fn create(&self, ctx: &Context, todo: &CreateTodo) -> Result<Todo, String>;
}
