use crate::domain::{entities::Transaction, error::TransactionError, gateways::WalletGateway};
use async_trait::async_trait;
use tracing::info;

/// Implementación Mock del Gateway de Wallet para desarrollo y testing.
///
/// Siempre retorna `true` (éxito) y loguea la operación.
/// Útil para probar el flujo de Transaction Service sin levantar Wallet Service.
pub struct FakeWalletGateway;

impl FakeWalletGateway {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WalletGateway for FakeWalletGateway {
    async fn process_movement(&self, transaction: &Transaction) -> Result<bool, TransactionError> {
        info!(
            " [FakeWalletGateway] Processing movement for Transaction ID: {}",
            transaction.id
        );
        info!(
            " [FakeWalletGateway] Amount: {}, Source: {:?}, Dest: {}",
            transaction.amount, transaction.source_wallet_id, transaction.destination_wallet_id
        );

        // Simulamos un pequeño delay de red
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        info!(" [FakeWalletGateway] Movement APPROVED");
        Ok(true)
    }
}
