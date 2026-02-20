use crate::domain::entities::Transaction;
use crate::domain::error::TransactionError;
use crate::domain::types::{TransactionId, WalletId};
use async_trait::async_trait;
use uuid::Uuid;

/// Puerto de Persistencia de Transacciones (Domain Layer).
///
/// Define el contrato que cualquier adaptador de almacenamiento debe cumplir.
/// Permite desacoplar la lógica de negocio de la implementación técnica (PostgreSQL, In-Memory, etc.).
#[async_trait]
pub trait TransactionRepository: Send + Sync {
    /// Persiste inicialmente una nueva transacción.
    ///
    /// Se debe usar al inicio del flujo (Saga) para registrar la intención de pago
    /// con estado `PENDING`. Esto garantiza auditoría incluso si el proceso falla después.
    async fn save(&self, transaction: Transaction) -> Result<Transaction, TransactionError>;

    /// Actualiza el estado de una transacción existente.
    ///
    /// Crítico para la consistencia eventual. Se llama después de recibir respuesta del Wallet Service.
    /// - `PENDING` -> `COMPLETED`: Si el Wallet debitó los fondos correctamente.
    /// - `PENDING` -> `FAILED`: Si el Wallet rechazó por saldo insuficiente.
    async fn update(&self, transaction: Transaction) -> Result<Transaction, TransactionError>;

    /// Busca una transacción por su ID único.
    async fn find_by_id(&self, id: TransactionId) -> Result<Option<Transaction>, TransactionError>;

    /// Recupera el historial de transacciones de una billetera.
    ///
    /// Debe devolver tanto transacciones donde la wallet es `source` (débitos)
    /// como donde es `destination` (créditos).
    async fn find_by_wallet_id(
        &self,
        wallet_id: WalletId,
    ) -> Result<Vec<Transaction>, TransactionError>;

    /// Busca una transacción por su ID de Correlación (Idempotency Key).
    ///
    /// **Esencial para evitar duplicidad de pagos.**
    /// Antes de procesar cualquier solicitud, se debe verificar si este ID ya existe.
    /// Si existe, se devuelve la transacción previa sin volver a ejecutar la lógica de cobro.
    async fn find_by_correlation_id(
        &self,
        correlation_id: Uuid,
    ) -> Result<Option<Transaction>, TransactionError>;

    /// Busca transacciones que han quedado en estado PENDING por más de cierto tiempo.
    ///
    /// Utilizado por jobs en segundo plano para reintentar o revertir transacciones atascadas.
    async fn find_pending_older_than(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Transaction>, TransactionError>;
}
