use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User not found with ID: {0}")]
    NotFound(Uuid),

    #[error("User already exists with username: {0}")]
    UsernameTaken(String),

    #[error("User already exists with email: {0}")]
    EmailTaken(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Invalid user data: {0}")]
    InvalidData(String),
}

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Wallet not found with ID: {0}")]
    NotFound(Uuid),

    #[error("Invalid wallet data: {0}")]
    InvalidData(String),

    #[error("Insufficient funds in wallet: {0}")]
    InsufficientFunds(Uuid),

    #[error("Optimistic locking conversion error: {0}")]
    ConcurrencyError(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}
