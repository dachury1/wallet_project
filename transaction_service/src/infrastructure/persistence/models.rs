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
    pub status: TransactionStatus, // Necesita SQLx Type override o ser String/Int en DB real
    pub transaction_type: TransactionType,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
}
