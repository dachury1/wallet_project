use crate::domain::{entities::Transaction, repository::TransactionRepository};
use rust_decimal::Decimal;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

pub struct ProcessTransactionUseCase {
    transaction_repo: Arc<dyn TransactionRepository>,
    // TODO: Cliente gRPC de Wallet Service para validar/reservar fondos
}

impl ProcessTransactionUseCase {
    pub fn new(transaction_repo: Arc<dyn TransactionRepository>) -> Self {
        Self { transaction_repo }
    }

    pub async fn execute(
        &self,
        source_wallet: Option<Uuid>,
        dest_wallet: Uuid,
        amount: Decimal,
    ) -> Result<Transaction, Box<dyn Error>> {
        // TODO: Orquestacion de la transaccion (Saga o 2PC simplificado)
        todo!()
    }
}
