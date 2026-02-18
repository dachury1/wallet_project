use crate::domain::{
    entities::Wallet,
    repository::{UserRepository, WalletRepository},
};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateWalletUseCase {
    wallet_repo: Arc<dyn WalletRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl CreateWalletUseCase {
    pub fn new(wallet_repo: Arc<dyn WalletRepository>, user_repo: Arc<dyn UserRepository>) -> Self {
        Self {
            wallet_repo,
            user_repo,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        currency: String,
        label: String,
    ) -> Result<Wallet, Box<dyn Error>> {
        // TODO: Validar usuario existe, validar moneda, crear wallet
        todo!()
    }
}
