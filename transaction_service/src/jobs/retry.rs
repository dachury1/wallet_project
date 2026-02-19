use crate::domain::entities::TransactionStatus;
use crate::domain::gateways::WalletGateway;
use crate::domain::repository::TransactionRepository;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Job en segundo plano para reintentar transacciones que quedaron en estado PENDING.
///
/// Esto puede ocurrir si el Transaction Service se reinició antes de recibir respuesta
/// del Wallet Service, o si hubo un timeout en la comunicación.
pub struct RetryFailedTransactionJob {
    transaction_repo: Arc<dyn TransactionRepository>,
    wallet_gateway: Arc<dyn WalletGateway>,
}

impl RetryFailedTransactionJob {
    pub fn new(
        transaction_repo: Arc<dyn TransactionRepository>,
        wallet_gateway: Arc<dyn WalletGateway>,
    ) -> Self {
        Self {
            transaction_repo,
            wallet_gateway,
        }
    }

    /// Ejecuta el proceso de recuperación.
    pub async fn run(&self) {
        info!("Starting RetryFailedTransactionJob...");

        // Definimos el umbral: transacciones pendientes hace más de 1 minuto.
        // Esto da tiempo suficiente para que una petición normal termine.
        let cutoff_time = Utc::now() - Duration::minutes(1);

        match self
            .transaction_repo
            .find_pending_older_than(cutoff_time)
            .await
        {
            Ok(transactions) => {
                if transactions.is_empty() {
                    return;
                }

                info!(
                    "Found {} stuck transactions. Processing...",
                    transactions.len()
                );

                for mut tx in transactions {
                    info!(
                        "Retrying transaction {} (Created at: {})",
                        tx.correlation_id, tx.created_at
                    );

                    // Reintentamos la comunicación con el Wallet Service
                    match self.wallet_gateway.process_movement(&tx).await {
                        Ok(true) => {
                            info!("Transaction {} approved by Wallet Service on retry.", tx.id);
                            tx.status = TransactionStatus::COMPLETED;
                        }
                        Ok(false) => {
                            warn!("Transaction {} rejected by Wallet Service on retry.", tx.id);
                            tx.status = TransactionStatus::FAILED;
                        }
                        Err(e) => {
                            // Si falla la comunicación, logueamos y seguimos (se reintentará en la próxima ejecución)
                            error!(
                                "Communication error with Wallet Service for tx {}: {:?}. Keeping as PENDING.",
                                tx.id, e
                            );
                            continue;
                        }
                    }

                    // Actualizamos el estado final en base de datos
                    if let Err(e) = self.transaction_repo.update(tx.clone()).await {
                        error!(
                            "FATAL: Failed to update status for tx {} after retry: {:?}",
                            tx.id, e
                        );
                    } else {
                        info!("Transaction {} status updated to {:?}", tx.id, tx.status);
                    }
                }
            }
            Err(e) => {
                error!("Failed to fetch pending transactions: {:?}", e);
            }
        }
    }
}
