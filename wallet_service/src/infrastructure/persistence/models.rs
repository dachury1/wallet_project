use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::FromRow;
use uuid::Uuid;

// Modelo de Base de Datos para User (especifico de SQLx)
#[derive(Debug, FromRow)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

// Modelo de Base de Datos para Wallet (especifico de SQLx)
#[derive(Debug, FromRow)]
pub struct WalletModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub label: String,
    pub balance: Decimal,
    pub currency: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

// Implementaciones de conversion: Model -> Entity
// impl From<UserModel> for crate::domain::entities::User { ... }
