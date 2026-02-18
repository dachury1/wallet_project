use crate::domain::entities::{User, Wallet};
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

// Interface (Port) for User persistence
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Box<dyn Error>>;
    async fn create(&self, user: User) -> Result<User, Box<dyn Error>>;
    async fn exists_by_username(&self, username: &str) -> Result<bool, Box<dyn Error>>;
}

// Interface (Port) for Wallet persistence
#[async_trait]
pub trait WalletRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Wallet>, Box<dyn Error>>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Wallet>, Box<dyn Error>>;
    async fn create(&self, wallet: Wallet) -> Result<Wallet, Box<dyn Error>>;
    async fn update_balance(
        &self,
        id: Uuid,
        amount: rust_decimal::Decimal,
    ) -> Result<(), Box<dyn Error>>;
}
