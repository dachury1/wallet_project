use crate::domain::entities::{Transaction, TransactionStatus, TransactionType};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct TransactionModel {
    pub id: Uuid,
    pub source_wallet_id: Option<Uuid>,
    pub destination_wallet_id: Uuid,
    pub amount: Decimal,
    pub status: TransactionStatus,
    pub transaction_type: TransactionType,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
}

impl From<&Transaction> for TransactionModel {
    fn from(t: &Transaction) -> Self {
        Self {
            id: t.id,
            source_wallet_id: t.source_wallet_id,
            destination_wallet_id: t.destination_wallet_id,
            amount: t.amount,
            status: t.status,
            transaction_type: t.transaction_type,
            created_at: t.created_at,
            correlation_id: t.correlation_id,
        }
    }
}

impl TryFrom<TransactionModel> for Transaction {
    type Error = String;

    fn try_from(m: TransactionModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: m.id,
            source_wallet_id: m.source_wallet_id,
            destination_wallet_id: m.destination_wallet_id,
            amount: m.amount,
            status: m.status,
            transaction_type: m.transaction_type,
            created_at: m.created_at,
            correlation_id: m.correlation_id,
        })
    }
}
