use thiserror::Error;
use uuid::Uuid;

use crate::domain::types::TransactionId;

#[derive(Error, Debug, PartialEq)]
pub enum TransactionError {
    #[error("Transaction not found with ID: {0}")]
    NotFound(TransactionId),

    #[error("Invalid transaction state: {0}")]
    InvalidState(String),

    #[error("Transaction repository error: {0}")]
    RepositoryError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Amount must be positive")]
    InvalidAmount,

    #[error("Source and destination wallets must be different")]
    SameWallet,

    #[error("Idempotency conflict: Transaction with correlation_id {0} already exists")]
    IdempotencyError(Uuid),

    #[error("Wallet Gateway error: {0}")]
    GatewayError(String),
}
