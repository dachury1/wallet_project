use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::types::{TransactionId, WalletId};

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
/// use transaction_service::domain::types::WalletId;
/// use uuid::Uuid;
/// use rust_decimal::Decimal;
///
/// let tx = Transaction::new(None, WalletId::new(), Decimal::from(100), Uuid::new_v4()).unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    id: TransactionId,
    source_wallet_id: Option<WalletId>, // Nullable
    destination_wallet_id: WalletId,
    amount: Decimal,
    status: TransactionStatus,
    transaction_type: TransactionType,
    created_at: DateTime<Utc>,
    correlation_id: Uuid, // Ya no es opcional
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
    /// use transaction_service::domain::types::WalletId;
    /// use uuid::Uuid;
    /// use rust_decimal::Decimal;
    ///
    /// let correlation_id = Uuid::new_v4();
    /// let tx = Transaction::new(None, WalletId::new(), Decimal::from(50), correlation_id);
    /// assert!(tx.is_ok());
    /// ```
    pub fn new(
        source_wallet: Option<WalletId>,
        dest_wallet: WalletId,
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
            id: TransactionId::new(),
            source_wallet_id: source_wallet,
            destination_wallet_id: dest_wallet,
            amount,
            status: TransactionStatus::PENDING,
            transaction_type: tx_type,
            created_at: Utc::now(),
            correlation_id,
        })
    }

    /// Reconstruye una instancia de `Transaction` desde los datos persistidos.
    pub fn reconstitute(
        id: TransactionId,
        source_wallet_id: Option<WalletId>,
        destination_wallet_id: WalletId,
        amount: Decimal,
        status: TransactionStatus,
        transaction_type: TransactionType,
        created_at: DateTime<Utc>,
        correlation_id: Uuid,
    ) -> Result<Self, TransactionError> {
        if amount <= Decimal::ZERO {
            return Err(TransactionError::InvalidAmount);
        }
        if let Some(src) = source_wallet_id {
            if src == destination_wallet_id {
                return Err(TransactionError::SameWallet);
            }
        }
        Ok(Self {
            id,
            source_wallet_id,
            destination_wallet_id,
            amount,
            status,
            transaction_type,
            created_at,
            correlation_id,
        })
    }

    pub fn id(&self) -> TransactionId {
        self.id
    }

    pub fn source_wallet_id(&self) -> Option<WalletId> {
        self.source_wallet_id
    }

    pub fn destination_wallet_id(&self) -> WalletId {
        self.destination_wallet_id
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn status(&self) -> TransactionStatus {
        self.status
    }

    pub fn transaction_type(&self) -> TransactionType {
        self.transaction_type
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    pub fn update_status(&mut self, new_status: TransactionStatus) {
        self.status = new_status;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use rust_decimal::Decimal;

    #[test]
    fn test_create_transfer_success() {
        let source = WalletId::new();
        let dest = WalletId::new();
        let amount = Decimal::from(100);
        let correlation_id = Uuid::new_v4();

        let tx = Transaction::new(Some(source), dest, amount, correlation_id).unwrap();

        assert_eq!(tx.transaction_type(), TransactionType::TRANSFER);
        assert_eq!(tx.status(), TransactionStatus::PENDING);
        assert_eq!(tx.amount(), amount);
        assert_eq!(tx.source_wallet_id(), Some(source));
        assert_eq!(tx.destination_wallet_id(), dest);
    }

    #[test]
    fn test_create_deposit_success() {
        let dest = WalletId::new();
        let amount = Decimal::from(50);
        let correlation_id = Uuid::new_v4();

        let tx = Transaction::new(None, dest, amount, correlation_id).unwrap();

        assert_eq!(tx.transaction_type(), TransactionType::DEPOSIT);
        assert_eq!(tx.status(), TransactionStatus::PENDING);
        assert_eq!(tx.source_wallet_id(), None);
    }

    #[rstest]
    #[case(0)]
    #[case(-10)]
    fn test_create_invalid_amount(#[case] amount_val: i64) {
        let source = WalletId::new();
        let dest = WalletId::new();
        let correlation_id = Uuid::new_v4();
        let amount = Decimal::from(amount_val);

        let result = Transaction::new(Some(source), dest, amount, correlation_id);

        assert_eq!(result.unwrap_err(), TransactionError::InvalidAmount);
    }

    #[test]
    fn test_create_same_wallet_error() {
        let wallet_id = WalletId::new();
        let amount = Decimal::from(100);
        let correlation_id = Uuid::new_v4();

        let result = Transaction::new(Some(wallet_id), wallet_id, amount, correlation_id);

        assert_eq!(result.unwrap_err(), TransactionError::SameWallet);
    }
}
