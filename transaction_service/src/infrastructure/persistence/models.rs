use crate::domain::entities::{Transaction, TransactionStatus, TransactionType};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::FromRow;
use uuid::Uuid;

// Modelo de Base de Datos para Transaction (especifico de SQLx)
// Representa la tabla 'transactions' en PostgreSQL.
// Diseñado para ser un reflejo directo (1:1) de la estructura de la tabla.
#[derive(Debug, FromRow)]
pub struct TransactionModel {
    pub id: Uuid,
    pub source_wallet_id: Option<Uuid>,
    pub destination_wallet_id: Uuid,
    pub amount: Decimal,
    pub status: TransactionStatus,
    pub transaction_type: TransactionType,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Uuid,
}

// Conversión Dominio -> Modelo (Eficiente: Copy Semantics)
// Aquí usamos From<&Transaction> porque todos los campos de Transaction
// son tipos "ligeros" (Copy o baratos de clonar) como Uuid, Decimal, Enums.
// No hay ganancia significativa en consumir la entidad (From<Transaction>),
// y esto nos permite reutilizar la entidad original si fuera necesario.
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

// Conversión Modelo -> Dominio (Infallible)
// Usamos From (siempre exitoso) en lugar de TryFrom porque
// la validación de tipos la garantiza SQLx al leer de la DB.
impl From<TransactionModel> for Transaction {
    fn from(m: TransactionModel) -> Self {
        Self {
            id: m.id,
            source_wallet_id: m.source_wallet_id,
            destination_wallet_id: m.destination_wallet_id,
            amount: m.amount,
            status: m.status,
            transaction_type: m.transaction_type,
            created_at: m.created_at,
            correlation_id: m.correlation_id,
        }
    }
}
