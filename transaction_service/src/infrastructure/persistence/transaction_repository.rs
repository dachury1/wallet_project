use crate::domain::entities::Transaction;
use crate::domain::repository::TransactionRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

pub struct PostgresTransactionRepository {
    pool: PgPool,
}

impl PostgresTransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionRepository for PostgresTransactionRepository {
    async fn save(&self, transaction: Transaction) -> Result<Transaction, Box<dyn Error>> {
        todo!()
    }
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>> {
        todo!()
    }
    async fn find_by_wallet_id(&self, wallet_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>> {
        todo!()
    }
}
