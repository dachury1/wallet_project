use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    PENDING,
    COMPLETED,
    FAILED,
    REVERSED,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    TRANSFER,
    DEPOSIT,
    WITHDRAWAL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub source_wallet_id: Option<Uuid>, // Nullable
    pub destination_wallet_id: Uuid,
    pub amount: Decimal,
    pub status: TransactionStatus,
    pub transaction_type: TransactionType,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
}
