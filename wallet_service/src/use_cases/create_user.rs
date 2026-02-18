use crate::domain::{entities::User, repository::UserRepository};
use std::error::Error;
use std::sync::Arc;

pub struct CreateUserUseCase {
    user_repo: Arc<dyn UserRepository>,
}

impl CreateUserUseCase {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, username: String, email: String) -> Result<User, Box<dyn Error>> {
        // TODO: Validaciones de dominio y llamada al repositorio
        todo!()
    }
}
