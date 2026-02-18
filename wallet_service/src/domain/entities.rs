use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Modelo de Entidad: User
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String, // Unique
    pub email: String,    // Unique
    pub created_at: DateTime<Utc>,
}

// Modelo de Entidad: Wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid, // FK -> User.id
    pub label: String,
    pub balance: Decimal, // Precisión fija
    pub currency: String, // ISO code
    pub version: i32,     // Optimistic Locking
}
