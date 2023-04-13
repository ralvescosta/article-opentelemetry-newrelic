use async_trait::async_trait;
use opentelemetry::{
    global,
    metrics::{Counter, Unit, UpDownCounter},
    Context,
};
use shared::{
    models::todo::{CreateTodo, Todo},
    repositories::TodoRepository,
};

use std::sync::Arc;
use tracing::warn;

pub struct TodoRepositoryImpl {
    counter: Counter<u64>,
    up_down_counter: UpDownCounter<i64>,
}

impl TodoRepositoryImpl {
    pub fn new() -> Arc<TodoRepositoryImpl> {
        let meter = global::meter("todo-controller");
        let up_down_counter = meter
            .i64_up_down_counter("todo_controller_counter")
            .with_description("HTTP Server Controller")
            .with_unit(Unit::new("rpm"))
            .init();

        let counter = meter
            .u64_counter("todo_controller_counter2")
            .with_description("HTTP Server Controller")
            .with_unit(Unit::new("rpm"))
            .init();

        Arc::new(TodoRepositoryImpl {
            up_down_counter,
            counter,
        })
    }
}

#[async_trait]
impl TodoRepository for TodoRepositoryImpl {
    async fn create(&self, ctx: &Context, todo: &CreateTodo) -> Result<Todo, String> {
        warn!("TodoRepositoryImpl::print");

        self.up_down_counter.add(ctx, 1, &[]);
        self.counter.add(ctx, 1, &[]);

        Ok(Todo::default())
    }
}
