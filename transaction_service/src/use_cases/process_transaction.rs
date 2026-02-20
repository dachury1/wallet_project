use crate::domain::{
    entities::{Transaction, TransactionStatus},
    error::TransactionError,
    gateways::WalletGateway,
    repository::TransactionRepository,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

/// Caso de uso central para el procesamiento y orquestamiento de transacciones.
///
/// Coordina la persistencia en la base de datos de transacciones, chequea la
/// idempotencia e interactúa con el `WalletGateway` para actualizar saldos.
///
/// # Examples
/// ```ignore
/// use transaction_service::use_cases::process_transaction::ProcessTransactionUseCase;
/// use transaction_service::domain::repository::MockTransactionRepositoryImpl;
/// use transaction_service::domain::gateways::MockWalletGatewayImpl;
/// use std::sync::Arc;
///
/// let repo = Arc::new(MockTransactionRepositoryImpl::new());
/// let gateway = Arc::new(MockWalletGatewayImpl::new());
/// let use_case = ProcessTransactionUseCase::new(repo, gateway);
/// ```
pub struct ProcessTransactionUseCase {
    transaction_repo: Arc<dyn TransactionRepository>,
    wallet_gateway: Arc<dyn WalletGateway>,
}

impl ProcessTransactionUseCase {
    pub fn new(
        transaction_repo: Arc<dyn TransactionRepository>,
        wallet_gateway: Arc<dyn WalletGateway>,
    ) -> Self {
        Self {
            transaction_repo,
            wallet_gateway,
        }
    }

    /// Ejecuta el proceso de inicio y completitud de una transacción, manejando su estado transicional.
    ///
    /// Valida la idempotencia basándose en `correlation_id`, crea la entidad `Transaction`, la
    /// guarda inicialmente como "PENDING", hace el llamado por external gateway, y finaliza
    /// guardando el estado como "COMPLETED" o "FAILED".
    ///
    /// # Examples
    /// ```ignore
    /// use uuid::Uuid;
    /// use rust_decimal_macros::dec;
    /// let dest = Uuid::new_v4();
    /// let tx = use_case.execute(None, dest, dec!(100.0), Uuid::new_v4()).await.unwrap();
    /// ```
    pub async fn execute(
        &self,
        source_wallet: Option<Uuid>,
        dest_wallet: Uuid,
        amount: Decimal,
        correlation_id: Uuid, // Now mandatory
    ) -> Result<Transaction, TransactionError> {
        // 1. Idempotency Check (Verificación de Idempotencia)
        // Antes de iniciar cualquier proceso, verificamos si esta solicitud ya fue procesada anteriormente.
        // Esto previene cobros duplicados en caso de reintentos por fallos de red o errores del cliente.
        // Si el `correlation_id` existe, devolvemos la transacción previa sin re-ejecutar la lógica.
        if let Ok(Some(existing_transaction)) = self
            .transaction_repo
            .find_by_correlation_id(correlation_id)
            .await
        {
            return Ok(existing_transaction);
        }

        // 2. Create Entity (Creación de Entidad y Reglas de Negocio)
        // Delegamos la validación de la "forma" (monto positivo, wallets distintas) al constructor de la Entidad.
        // Esto asegura que nunca trabajemos con una estructura `Transaction` inválida en la capa de aplicación.
        let transaction = Transaction::new(source_wallet, dest_wallet, amount, correlation_id)?;

        // 3. Persist Initial Intent (Persistencia del Intento - Estado PENDING)
        // Guardamos la transacción con estado `PENDING` *antes* de contactar al servicio externo.
        // Esto actúa como un registro de auditoría (write-ahead log). Si el proceso muere aquí,
        // sabremos que hubo un intento fallido (o pendiente de conciliación).
        // Usamos `map_err` para envolver el error original de la BD con contexto útil.
        let saved_transaction = self
            .transaction_repo
            .save(transaction.clone())
            .await
            .map_err(|e| TransactionError::RepositoryError(format!("DB Save Error: {}", e)))?;

        // 4. Call Wallet Service (Ejecución de la Acción Distribuida)
        // Solicitamos al Wallet Service que mueva los fondos. Esta es la operación crítica ("Point of No Return").
        // El Gateway abstrae si es una llamada gRPC, HTTP o mensaje en cola.
        let result = self
            .wallet_gateway
            .process_movement(&saved_transaction)
            .await;

        // 5. Handle Result (Commit o Rollback - Consistencia Eventual)
        match result {
            Ok(true) => {
                // Happy Path: El Wallet Service confirmó el movimiento.
                // Actualizamos el estado local a `COMPLETED`.
                let success_transaction = Transaction {
                    status: TransactionStatus::COMPLETED,
                    ..saved_transaction
                };
                self.transaction_repo
                    .update(success_transaction)
                    .await
                    .map_err(|e| {
                        TransactionError::RepositoryError(format!("DB Commit Error: {}", e))
                    })
            }
            Ok(false) | Err(_) => {
                // Failure Path: El Wallet Service rechazó (fondos insuficientes) o falló la comunicación.
                // Debemos marcar la transacción como `FAILED` para cerrar el ciclo de vida.
                let failed_transaction = Transaction {
                    status: TransactionStatus::FAILED,
                    ..saved_transaction
                };

                // Best-effort rollback: Intentamos guardar el estado de fallo.
                // Ignoramos el resultado de este update (`let _`) porque nuestro objetivo principal
                // es retornar el error original que causó el fallo.
                let _ = self.transaction_repo.update(failed_transaction).await;

                // Retornamos el error específico para que el cliente sepa qué pasó.
                match result {
                    Err(e) => Err(TransactionError::GatewayError(e.to_string())),
                    Ok(false) => Err(TransactionError::GatewayError(
                        "Wallet rejected the transaction".to_string(),
                    )),
                    _ => unreachable!(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Transaction, TransactionStatus, TransactionType};
    use crate::domain::error::TransactionError;
    use crate::domain::gateways::WalletGateway;
    use crate::domain::repository::TransactionRepository;
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use mockall::mock;
    use mockall::predicate::*;
    use rust_decimal::Decimal;
    use std::sync::Arc;
    use uuid::Uuid;

    mock! {
        pub TransactionRepositoryImpl {}

        #[async_trait]
        impl TransactionRepository for TransactionRepositoryImpl {
            async fn save(&self, transaction: Transaction) -> Result<Transaction, TransactionError>;
            async fn update(&self, transaction: Transaction) -> Result<Transaction, TransactionError>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, TransactionError>;
            async fn find_by_wallet_id(&self, wallet_id: Uuid) -> Result<Vec<Transaction>, TransactionError>;
            async fn find_by_correlation_id(&self, correlation_id: Uuid) -> Result<Option<Transaction>, TransactionError>;
            async fn find_pending_older_than(&self, timestamp: DateTime<Utc>) -> Result<Vec<Transaction>, TransactionError>;
        }
    }

    mock! {
        pub WalletGatewayImpl {}

        #[async_trait]
        impl WalletGateway for WalletGatewayImpl {
            async fn process_movement(&self, transaction: &Transaction) -> Result<bool, TransactionError>;
        }
    }

    #[tokio::test]
    async fn test_process_transaction_idempotency() {
        // Arrange
        let mut mock_repo = MockTransactionRepositoryImpl::new();
        let mock_gateway = MockWalletGatewayImpl::new();

        let correlation_id = Uuid::new_v4();
        let existing_tx = Transaction {
            id: Uuid::new_v4(),
            source_wallet_id: Some(Uuid::new_v4()),
            destination_wallet_id: Uuid::new_v4(),
            amount: Decimal::from(100),
            status: TransactionStatus::COMPLETED,
            transaction_type: TransactionType::TRANSFER,
            created_at: Utc::now(),
            correlation_id,
        };
        let expected_tx = existing_tx.clone();

        mock_repo
            .expect_find_by_correlation_id()
            .with(eq(correlation_id))
            .times(1)
            .returning(move |_| Ok(Some(existing_tx.clone())));

        let use_case = ProcessTransactionUseCase::new(Arc::new(mock_repo), Arc::new(mock_gateway));

        // Act
        let result = use_case
            .execute(
                Some(Uuid::new_v4()),
                Uuid::new_v4(),
                Decimal::from(100),
                correlation_id,
            )
            .await;

        // Assert
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert_eq!(tx.id, expected_tx.id);
        assert_eq!(tx.status, TransactionStatus::COMPLETED);
    }

    #[tokio::test]
    async fn test_process_transaction_success() {
        // Arrange
        let mut mock_repo = MockTransactionRepositoryImpl::new();
        let mut mock_gateway = MockWalletGatewayImpl::new();

        let source_wallet = Uuid::new_v4();
        let dest_wallet = Uuid::new_v4();
        let amount = Decimal::from(100);
        let correlation_id = Uuid::new_v4();

        // 1. Check idempotency (returns None)
        mock_repo
            .expect_find_by_correlation_id()
            .with(eq(correlation_id))
            .times(1)
            .returning(|_| Ok(None));

        // 2. Save PENDING
        mock_repo.expect_save().times(1).returning(|tx| Ok(tx)); // Return what was passed (with generated ID)

        // 3. Call Wallet Gateway (returns true)
        mock_gateway
            .expect_process_movement()
            .times(1)
            .returning(|_| Ok(true));

        // 4. Update COMPLETED
        mock_repo
            .expect_update()
            .with(function(|tx: &Transaction| {
                tx.status == TransactionStatus::COMPLETED
            }))
            .times(1)
            .returning(|tx| Ok(tx));

        let use_case = ProcessTransactionUseCase::new(Arc::new(mock_repo), Arc::new(mock_gateway));

        // Act
        let result = use_case
            .execute(Some(source_wallet), dest_wallet, amount, correlation_id)
            .await;

        // Assert (verify result state, though mock expectations cover flow)
        assert!(result.is_ok());
        // Note: result will contain what expect_update returned.
    }

    #[tokio::test]
    async fn test_process_transaction_gateway_failure() {
        // Arrange
        let mut mock_repo = MockTransactionRepositoryImpl::new();
        let mut mock_gateway = MockWalletGatewayImpl::new();

        let source_wallet = Uuid::new_v4();
        let dest_wallet = Uuid::new_v4();
        let amount = Decimal::from(50);
        let correlation_id = Uuid::new_v4();

        mock_repo
            .expect_find_by_correlation_id()
            .returning(|_| Ok(None));
        mock_repo.expect_save().returning(|tx| Ok(tx));

        // Gateway returns false (e.g. insufficient funds)
        mock_gateway
            .expect_process_movement()
            .returning(|_| Ok(false));

        // Should update to FAILED
        mock_repo
            .expect_update()
            .with(function(|tx: &Transaction| {
                tx.status == TransactionStatus::FAILED
            }))
            .times(1)
            .returning(|tx| Ok(tx));

        let use_case = ProcessTransactionUseCase::new(Arc::new(mock_repo), Arc::new(mock_gateway));

        // Act
        let result = use_case
            .execute(Some(source_wallet), dest_wallet, amount, correlation_id)
            .await;

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            TransactionError::GatewayError("Wallet rejected the transaction".to_string())
        );
    }
}
