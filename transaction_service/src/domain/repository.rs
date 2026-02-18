use crate::domain::entities::Transaction;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

// Port for Transaction Persistence
#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn save(&self, transaction: Transaction) -> Result<Transaction, Box<dyn Error>>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>>;
    async fn find_by_wallet_id(&self, wallet_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>>;
}
