use tonic::async_trait;

use crate::domain::{entities::Transaction, error::TransactionError};

#[async_trait]
pub trait WalletGateway: Send + Sync {
    // Retorna true si fue exitoso, o un error si falló (saldo insuficiente, usuario no existe, etc.)
    async fn process_movement(&self, transaction: &Transaction) -> Result<bool, TransactionError>;
}
