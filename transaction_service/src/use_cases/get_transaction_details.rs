use crate::domain::{
    entities::Transaction, error::TransactionError, repository::TransactionRepository,
};
use std::sync::Arc;
use uuid::Uuid;

/// Caso de uso para obtener los detalles de una única transacción.
///
/// Encapsula la lógica necesaria para consultar una transacción específica
/// por su ID delegando la tarea de acceso a datos al `TransactionRepository`.
///
/// # Examples
/// ```ignore
/// use transaction_service::use_cases::get_transaction_details::GetTransactionDetailsUseCase;
/// use transaction_service::domain::repository::MockTransactionRepositoryImpl;
/// use std::sync::Arc;
///
/// let repo = Arc::new(MockTransactionRepositoryImpl::new());
/// let use_case = GetTransactionDetailsUseCase::new(repo);
/// ```
#[derive(Clone)]
pub struct GetTransactionDetailsUseCase {
    transaction_repo: Arc<dyn TransactionRepository>,
}

impl GetTransactionDetailsUseCase {
    /// Construye una nueva instancia de `GetTransactionDetailsUseCase`.
    ///
    /// Se inyecta una implementación de `TransactionRepository` utilizando `Arc<dyn ...>`
    /// para permitir inyección de dependencias seguras para la concurrencia.
    pub fn new(transaction_repo: Arc<dyn TransactionRepository>) -> Self {
        Self { transaction_repo }
    }

    /// Ejecuta el caso de uso para buscar los detalles de una transacción por su ID.
    ///
    /// # Argumentos
    ///
    /// * `transaction_id` - El identificador único (`Uuid`) de la transacción.
    ///
    /// # Retornos
    ///
    /// Devuelve un `Result<Transaction, TransactionError>`. Retorna la
    /// transacción en caso de éxito o un `TransactionError::NotFound` si no existe la transacción.
    ///
    /// # Examples
    /// ```ignore
    /// use uuid::Uuid;
    /// let tx_id = Uuid::new_v4();
    /// let details = use_case.execute(tx_id).await.unwrap();
    /// ```
    #[tracing::instrument(name = "GetTransactionDetailsUseCase::execute", skip(self))]
    pub async fn execute(&self, transaction_id: Uuid) -> Result<Transaction, TransactionError> {
        self.transaction_repo
            .find_by_id(transaction_id)
            .await?
            .ok_or(TransactionError::NotFound(transaction_id))
    }
}
