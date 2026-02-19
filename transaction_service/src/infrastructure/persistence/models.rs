use crate::domain::entities::{TransactionStatus, TransactionType};
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
    pub status: String,           // Stored as VARCHAR in DB
    pub transaction_type: String, // Stored as VARCHAR in DB
    pub created_at: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
}

impl From<&crate::domain::entities::Transaction> for TransactionModel {
    fn from(t: &crate::domain::entities::Transaction) -> Self {
        Self {
            id: t.id,
            source_wallet_id: t.source_wallet_id,
            destination_wallet_id: t.destination_wallet_id,
            amount: t.amount,
            status: format!("{:?}", t.status), // Defines usage of derived Debug implementation
            transaction_type: format!("{:?}", t.transaction_type),
            created_at: t.created_at,
            correlation_id: t.correlation_id,
        }
    }
}

impl TryFrom<TransactionModel> for crate::domain::entities::Transaction {
    type Error = String;

    fn try_from(m: TransactionModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: m.id,
            source_wallet_id: m.source_wallet_id,
            destination_wallet_id: m.destination_wallet_id,
            amount: m.amount,
            status: match m.status.as_str() {
                "PENDING" => TransactionStatus::PENDING,
                "COMPLETED" => TransactionStatus::COMPLETED,
                "FAILED" => TransactionStatus::FAILED,
                "REVERSED" => TransactionStatus::REVERSED,
                _ => return Err(format!("Invalid status: {}", m.status)),
            },
            transaction_type: match m.transaction_type.as_str() {
                "TRANSFER" => TransactionType::TRANSFER,
                "DEPOSIT" => TransactionType::DEPOSIT,
                "WITHDRAWAL" => TransactionType::WITHDRAWAL,
                _ => return Err(format!("Invalid type: {}", m.transaction_type)),
            },
            created_at: m.created_at,
            correlation_id: m.correlation_id,
        })
    }
}
