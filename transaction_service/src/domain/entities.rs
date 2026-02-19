use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::TransactionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    PENDING,
    COMPLETED,
    FAILED,
    REVERSED,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_type", rename_all = "SCREAMING_SNAKE_CASE")]
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
    pub correlation_id: Uuid, // Ya no es opcional
}

impl Transaction {
    pub fn new(
        source_wallet: Option<Uuid>,
        dest_wallet: Uuid,
        amount: Decimal,
        correlation_id: Uuid,
    ) -> Result<Self, TransactionError> {
        // 1. Validar Monto
        if amount <= Decimal::ZERO {
            return Err(TransactionError::InvalidAmount);
        }

        // 2. Determinar Tipo de Transacción y Validaciones
        let tx_type = match source_wallet {
            Some(src) => {
                if src == dest_wallet {
                    return Err(TransactionError::SameWallet);
                }
                TransactionType::TRANSFER
            }
            None => TransactionType::DEPOSIT,
        };

        // 3. Crear Entidad
        Ok(Self {
            id: Uuid::new_v4(),
            source_wallet_id: source_wallet,
            destination_wallet_id: dest_wallet,
            amount,
            status: TransactionStatus::PENDING,
            transaction_type: tx_type,
            created_at: Utc::now(),
            correlation_id,
        })
    }
}
