use crate::domain::{
    entities::{Transaction, TransactionStatus},
    error::TransactionError,
    gateways::WalletGateway,
    repository::TransactionRepository,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

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
