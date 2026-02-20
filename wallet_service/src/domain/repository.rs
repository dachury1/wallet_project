use crate::domain::entities::{User, Wallet};
use crate::domain::error::{UserError, WalletError};
use crate::domain::types::{UserId, WalletId};
use async_trait::async_trait;

// Interface (Port) for User persistence
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, UserError>;
    async fn create(&self, user: User) -> Result<User, UserError>;
    async fn exists_by_username(&self, username: &str) -> Result<bool, UserError>;
}

// Interface (Port) for Wallet persistence
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait WalletRepository: Send + Sync {
    async fn find_by_id(&self, id: WalletId) -> Result<Option<Wallet>, WalletError>;
    async fn find_by_user_id(&self, user_id: UserId) -> Result<Vec<Wallet>, WalletError>;
    async fn create(&self, wallet: Wallet) -> Result<Wallet, WalletError>;
    async fn update_balance(
        &self,
        id: WalletId,
        amount: rust_decimal::Decimal,
    ) -> Result<(), WalletError>;
}
