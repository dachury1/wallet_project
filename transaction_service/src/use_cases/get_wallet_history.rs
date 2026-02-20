use crate::domain::{
    entities::Transaction, error::TransactionError, repository::TransactionRepository,
};
use std::sync::Arc;
use uuid::Uuid;

/// Caso de uso para obtener el historial de transacciones de una billetera.
///
/// Encapsula la lógica de buscar transacciones (tanto depósitos, como retiros y transferencias)
/// utilizando el repositorio de persistencia `TransactionRepository`.
#[derive(Clone)]
pub struct GetWalletHistoryUseCase {
    transaction_repo: Arc<dyn TransactionRepository>,
}

impl GetWalletHistoryUseCase {
    /// Construye una nueva instancia de `GetWalletHistoryUseCase`.
    ///
    /// Se le inyecta una implementación de `TransactionRepository` envuelta en un `Arc`
    /// para permitir invocaciones seguras entre múltiples subprocesos (concurrency-safe / thread-safe),
    /// cumpliendo con las bases de la inyección de dependencias de la Arquitectura Limpia (Clean Architecture).
    pub fn new(transaction_repo: Arc<dyn TransactionRepository>) -> Self {
        Self { transaction_repo }
    }

    /// Ejecuta el caso de uso para buscar el listado de movimientos por ID de billetera.
    ///
    /// # Argumentos
    ///
    /// * `wallet_id` - El identificador único (`Uuid`) de la billetera involucrada en las transacciones.
    ///
    /// # Retornos
    ///
    /// Devuelve un `Result<Vec<Transaction>, TransactionError>`. Retorna una colección
    /// (quizás vacía si no hay transacciones) de `Transaction`s al tener una operación en BD exitosa.
    #[tracing::instrument(name = "GetWalletHistoryUseCase::execute", skip(self))]
    pub async fn execute(&self, wallet_id: Uuid) -> Result<Vec<Transaction>, TransactionError> {
        self.transaction_repo.find_by_wallet_id(wallet_id).await
    }
}
