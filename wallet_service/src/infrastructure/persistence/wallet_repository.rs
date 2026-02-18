use crate::domain::entities::Wallet;
use crate::domain::repository::WalletRepository;
use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

pub struct PostgresWalletRepository {
    pool: PgPool,
}

impl PostgresWalletRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WalletRepository for PostgresWalletRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Wallet>, Box<dyn Error>> {
        todo!()
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Wallet>, Box<dyn Error>> {
        todo!()
    }

    async fn create(&self, wallet: Wallet) -> Result<Wallet, Box<dyn Error>> {
        todo!()
    }

    async fn update_balance(&self, id: Uuid, amount: Decimal) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
