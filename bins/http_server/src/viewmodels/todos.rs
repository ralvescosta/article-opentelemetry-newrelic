use serde::{Deserialize, Serialize};
use shared::models::todo::{CreateTodo, Todo};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateTodoRequest {
    pub(crate) name: String,
    pub(crate) description: String,
}

impl Into<CreateTodo> for CreateTodoRequest {
    fn into(self) -> CreateTodo {
        CreateTodo {
            name: self.name,
            description: self.description,
        }
    }
}

#[derive(Default, Serialize, Deserialize, ToSchema)]
pub struct TodoResponse {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) created_at: String,
}

impl From<&Todo> for TodoResponse {
    fn from(value: &Todo) -> Self {
        TodoResponse {
            id: value.id.clone(),
            name: value.name.clone(),
            description: value.description.clone(),
            created_at: value.created_at.clone(),
        }
    }
}
