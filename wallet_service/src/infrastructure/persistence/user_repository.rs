use crate::domain::entities::User;
use crate::domain::repository::UserRepository;
use crate::infrastructure::persistence::models::UserModel;
use async_trait::async_trait;
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Box<dyn Error>> {
        // TODO: Implementar consulta SQLx
        todo!()
    }

    async fn create(&self, user: User) -> Result<User, Box<dyn Error>> {
        // TODO: INSERT INTO users ...
        todo!()
    }

    async fn exists_by_username(&self, username: &str) -> Result<bool, Box<dyn Error>> {
        todo!()
    }
}
