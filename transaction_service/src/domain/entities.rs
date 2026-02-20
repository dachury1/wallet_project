use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::TransactionError;

/// Estado de una transacción en el sistema.
///
/// # Examples
/// ```
/// use transaction_service::domain::entities::TransactionStatus;
/// let status = TransactionStatus::PENDING;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    PENDING,
    COMPLETED,
    FAILED,
    REVERSED,
}

/// Tipo de transacción financiera.
///
/// # Examples
/// ```
/// use transaction_service::domain::entities::TransactionType;
/// let tx_type = TransactionType::DEPOSIT;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    TRANSFER,
    DEPOSIT,
    WITHDRAWAL,
}

/// Entidad que representa una transacción financiera entre billeteras.
///
/// # Examples
/// ```
/// use transaction_service::domain::entities::Transaction;
/// use uuid::Uuid;
/// use rust_decimal_macros::dec;
///
/// let tx = Transaction::new(None, Uuid::new_v4(), dec!(100.0), Uuid::new_v4()).unwrap();
/// ```
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
    /// Inicializa una nueva instancia de `Transaction`.
    ///
    /// Valida que el monto sea positivo, que la billetera origen (si existe) sea diferente
    /// de la destino, y determina el tipo de transacción en base a la existencia de la billetera origen.
    ///
    /// # Examples
    /// ```
    /// use transaction_service::domain::entities::Transaction;
    /// use uuid::Uuid;
    /// use rust_decimal_macros::dec;
    ///
    /// let correlation_id = Uuid::new_v4();
    /// let tx = Transaction::new(None, Uuid::new_v4(), dec!(50.0), correlation_id);
    /// assert!(tx.is_ok());
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use rust_decimal::Decimal;

    #[test]
    fn test_create_transfer_success() {
        let source = Uuid::new_v4();
        let dest = Uuid::new_v4();
        let amount = Decimal::from(100);
        let correlation_id = Uuid::new_v4();

        let tx = Transaction::new(Some(source), dest, amount, correlation_id).unwrap();

        assert_eq!(tx.transaction_type, TransactionType::TRANSFER);
        assert_eq!(tx.status, TransactionStatus::PENDING);
        assert_eq!(tx.amount, amount);
        assert_eq!(tx.source_wallet_id, Some(source));
        assert_eq!(tx.destination_wallet_id, dest);
    }

    #[test]
    fn test_create_deposit_success() {
        let dest = Uuid::new_v4();
        let amount = Decimal::from(50);
        let correlation_id = Uuid::new_v4();

        let tx = Transaction::new(None, dest, amount, correlation_id).unwrap();

        assert_eq!(tx.transaction_type, TransactionType::DEPOSIT);
        assert_eq!(tx.status, TransactionStatus::PENDING);
        assert_eq!(tx.source_wallet_id, None);
    }

    #[rstest]
    #[case(0)]
    #[case(-10)]
    fn test_create_invalid_amount(#[case] amount_val: i64) {
        let source = Uuid::new_v4();
        let dest = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let amount = Decimal::from(amount_val);

        let result = Transaction::new(Some(source), dest, amount, correlation_id);

        assert_eq!(result.unwrap_err(), TransactionError::InvalidAmount);
    }

    #[test]
    fn test_create_same_wallet_error() {
        let wallet_id = Uuid::new_v4();
        let amount = Decimal::from(100);
        let correlation_id = Uuid::new_v4();

        let result = Transaction::new(Some(wallet_id), wallet_id, amount, correlation_id);

        assert_eq!(result.unwrap_err(), TransactionError::SameWallet);
    }
}
