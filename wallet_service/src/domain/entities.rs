use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::error::{UserError, WalletError};
use crate::domain::types::{UserId, WalletId};

/// Modelo de Entidad: User.
/// Representa a un usuario dentro del sistema, con su información básica de identidad.
///
/// # Examples
/// ```
/// use wallet_service::domain::entities::User;
///
/// let user = User::new("johndoe".to_string(), "john@example.com".to_string()).unwrap();
/// assert_eq!(user.username(), "johndoe");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    username: String, // Unique
    email: String,    // Unique
    created_at: DateTime<Utc>,
}

impl User {
    /// Inicializa una nueva instancia válida de `User`.
    ///
    /// Valida que el nombre de usuario y el correo no estén vacíos.
    ///
    /// # Examples
    /// ```
    /// use wallet_service::domain::entities::User;
    ///
    /// let user = User::new("user1".to_string(), "user1@test.com".to_string());
    /// assert!(user.is_ok());
    /// ```
    pub fn new(username: String, email: String) -> Result<Self, UserError> {
        if username.trim().is_empty() || email.trim().is_empty() {
            return Err(UserError::InvalidData(
                "Username and email cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            id: UserId::new(),
            username,
            email,
            created_at: Utc::now(),
        })
    }

    /// Reconstruye una instancia de `User` desde los datos persistidos.
    /// Esto es un constructor cerrado (`new(...)`) para uso de la capa de persistencia
    /// que regresa errores de validación si existen datos inválidos en la BD.
    pub fn reconstitute(
        id: UserId,
        username: String,
        email: String,
        created_at: DateTime<Utc>,
    ) -> Result<Self, UserError> {
        if username.trim().is_empty() || email.trim().is_empty() {
            return Err(UserError::InvalidData(
                "Username and email cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            id,
            username,
            email,
            created_at,
        })
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

/// Modelo de Entidad: Wallet.
/// Representa una billetera de un usuario, que alinea fondos en una divisa específica e implementa optimistic locking.
///
/// # Examples
/// ```
/// use wallet_service::domain::entities::Wallet;
/// use wallet_service::domain::types::UserId;
/// use uuid::Uuid;
///
/// let wallet_builder = Wallet::builder();
/// let wallet = wallet_builder
///     .user_id(UserId::new())
///     .label("My Wallet".to_string())
///     .currency("USD".to_string())
///     .build();
/// assert!(wallet.is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    id: WalletId,
    user_id: UserId, // FK -> User.id
    label: String,
    balance: Decimal, // Precisión fija
    currency: String, // ISO code
    version: i32,     // Optimistic Locking
}

impl Wallet {
    /// Inicia la construcción de una instancia `Wallet` con el patrón Builder.
    ///
    /// # Examples
    /// ```
    /// use wallet_service::domain::entities::Wallet;
    ///
    /// let builder = Wallet::builder();
    /// ```
    pub fn builder() -> WalletBuilder {
        WalletBuilder::default()
    }

    /// Reconstruye una billetera cargada desde la persistencia o en memoria.
    /// Valida los datos esenciales siguiendo reglas de dominio básicas.
    pub fn reconstitute(
        id: WalletId,
        user_id: UserId,
        label: String,
        balance: Decimal,
        currency: String,
        version: i32,
    ) -> Result<Self, WalletError> {
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

        Ok(Self {
            id,
            user_id,
            label,
            balance,
            currency,
            version,
        })
    }

    pub fn id(&self) -> WalletId {
        self.id
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn balance(&self) -> Decimal {
        self.balance
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn version(&self) -> i32 {
        self.version
    }
}

/// Builder para asegurar que al instanciarse la entidad Wallet, todas las reglas de negocio base aplican (como validación de campos).
///
/// # Examples
/// ```
/// use wallet_service::domain::entities::WalletBuilder;
/// use wallet_service::domain::types::UserId;
/// use uuid::Uuid;
///
/// let builder = WalletBuilder::default()
///     .user_id(UserId::new())
///     .label("Savings".to_string())
///     .currency("EUR".to_string());
/// ```
#[derive(Default)]
pub struct WalletBuilder {
    user_id: Option<UserId>,
    label: Option<String>,
    currency: Option<String>,
}

impl WalletBuilder {
    pub fn user_id(mut self, user_id: UserId) -> Self {
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

    /// Construye y valida la entidad instanciada.
    ///
    /// # Examples
    /// ```
    /// use wallet_service::domain::entities::Wallet;
    /// use wallet_service::domain::types::UserId;
    /// use uuid::Uuid;
    ///
    /// let wallet = Wallet::builder()
    ///     .user_id(UserId::new())
    ///     .label("Main".to_string())
    ///     .currency("USD".to_string())
    ///     .build();
    /// assert!(wallet.is_ok());
    /// ```
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
            id: WalletId::new(),
            user_id,
            label,
            balance: Decimal::from(0),
            currency,
            version: 0,
        })
    }
}
