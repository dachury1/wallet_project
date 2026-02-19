use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Transaction not found with ID: {0}")]
    NotFound(Uuid),

    #[error("Invalid transaction state: {0}")]
    InvalidState(String),

    #[error("Transaction repository error: {0}")]
    RepositoryError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),
}
