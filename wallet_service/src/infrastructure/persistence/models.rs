use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::FromRow;

use crate::domain::entities::{User, Wallet};
use crate::domain::types::{UserId, WalletId};

// Modelo de Base de Datos para User (especifico de SQLx)
// Representa la tabla 'users' en PostgreSQL.
#[derive(Debug, FromRow)]
pub struct UserModel {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

// Conversión Dominio -> Modelo (Eficiente: Move Semantics)
// Implementamos From<User> en lugar de From<&User> para consumir la entidad
// y mover los Strings (username, email) sin realizar clones costosos.
impl From<User> for UserModel {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            created_at: u.created_at,
        }
    }
}

// Conversión Modelo -> Dominio
// Permite reconstruir la entidad de dominio al leer de la base de datos.
impl From<UserModel> for User {
    fn from(m: UserModel) -> Self {
        Self {
            id: m.id,
            username: m.username,
            email: m.email,
            created_at: m.created_at,
        }
    }
}

// Modelo de Base de Datos para Wallet (especifico de SQLx)
// Representa la tabla 'wallets'. Incluye 'created_at' que no está en la entidad de dominio.
#[derive(Debug, FromRow)]
pub struct WalletModel {
    pub id: WalletId,
    pub user_id: UserId,
    pub label: String,
    pub balance: Decimal,
    pub currency: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

// Conversión Dominio -> Modelo
// Nota: La entidad Wallet no tiene fecha de creación, pero la DB la requiere.
impl From<Wallet> for WalletModel {
    fn from(w: Wallet) -> Self {
        Self {
            id: w.id,
            user_id: w.user_id,
            label: w.label,
            balance: w.balance,
            currency: w.currency,
            version: w.version,
            // Asignamos la fecha actual (UTC) al persistir.
            // IMPORTANTE: Esto asume que estamos creando el registro.
            // Para updates, se debería usar una query que ignore este campo o preserve el valor original.
            created_at: Utc::now(),
        }
    }
}

// Conversión Modelo -> Dominio
// Ignoramos el campo 'created_at' del modelo ya que la entidad no lo necesita.
impl From<WalletModel> for Wallet {
    fn from(w: WalletModel) -> Self {
        Self {
            id: w.id,
            user_id: w.user_id,
            label: w.label,
            balance: w.balance,
            currency: w.currency,
            version: w.version,
        }
    }
}
