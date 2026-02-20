use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::{UserError, WalletError};

// Modelo de Entidad: User
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String, // Unique
    pub email: String,    // Unique
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, email: String) -> Result<Self, UserError> {
        if username.trim().is_empty() || email.trim().is_empty() {
            return Err(UserError::InvalidData(
                "Username and email cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            username,
            email,
            created_at: Utc::now(),
        })
    }
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

impl Wallet {
    /// Inicia la construcción de una instancia `Wallet` con el patrón Builder
    pub fn builder() -> WalletBuilder {
        WalletBuilder::default()
    }
}

/// Builder para asegurar que al instanciarse la entidad Wallet, todas las reglas de negocio base aplican (como validación de campos)
#[derive(Default)]
pub struct WalletBuilder {
    user_id: Option<Uuid>,
    label: Option<String>,
    currency: Option<String>,
}

impl WalletBuilder {
    pub fn user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn currency(mut self, currency: String) -> Self {
        self.currency = Some(currency);
        self
    }

    /// Construye y valida la entidad instanciada
    pub fn build(self) -> Result<Wallet, WalletError> {
        let user_id = self
            .user_id
            .ok_or_else(|| WalletError::InvalidData("El campo user_id es obligatorio".into()))?;
        let label = self
            .label
            .ok_or_else(|| WalletError::InvalidData("El campo label es obligatorio".into()))?;
        let currency = self
            .currency
            .ok_or_else(|| WalletError::InvalidData("El campo currency es obligatorio".into()))?;

        if label.trim().is_empty() {
            return Err(WalletError::InvalidData(
                "La etiqueta de la wallet no puede estar en blanco".into(),
            ));
        }

        let currency = currency.trim().to_uppercase();
        if currency.len() != 3 {
            return Err(WalletError::InvalidData(
                "La divisa debe ser un código ISO de 3 letras".into(),
            ));
        }

        Ok(Wallet {
            id: Uuid::new_v4(),
            user_id,
            label,
            balance: Decimal::from(0),
            currency,
            version: 0,
        })
    }
}
