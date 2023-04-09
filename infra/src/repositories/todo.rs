use shared::repositories::TodoRepository;
use std::sync::Arc;

pub struct TodoRepositoryImpl {}

impl TodoRepositoryImpl {
    pub fn new() -> Arc<TodoRepositoryImpl> {
        Arc::new(TodoRepositoryImpl {})
    }
}

impl TodoRepository for TodoRepositoryImpl {}
