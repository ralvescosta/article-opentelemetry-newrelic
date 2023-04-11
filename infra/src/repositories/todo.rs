use shared::repositories::TodoRepository;
use std::sync::Arc;
use tracing::info;

pub struct TodoRepositoryImpl {}

impl TodoRepositoryImpl {
    pub fn new() -> Arc<TodoRepositoryImpl> {
        Arc::new(TodoRepositoryImpl {})
    }
}

impl TodoRepository for TodoRepositoryImpl {
    fn print(&self) {
        info!("TodoRepositoryImpl::print");
    }
}
