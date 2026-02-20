use crate::domain::error::WalletError;
use crate::domain::repository::WalletRepository;
use std::sync::Arc;
use uuid::Uuid;

/// Casos de uso para procesar movimientos (depósitos/retiros) en una billetera.
///
/// Encapsula la lógica de llamar al repositorio para realizar la actualización
/// atómica del balance en la base de datos.
#[derive(Clone)]
pub struct ProcessMovementUseCase {
    wallet_repo: Arc<dyn WalletRepository>,
}

impl ProcessMovementUseCase {
    /// Construye una nueva instancia de `ProcessMovementUseCase`.
    ///
    /// Se le inyecta una implementación de `WalletRepository` utilizando `Arc<dyn ...>`
    /// para permitir su uso seguro en entornos multihilo según las reglas de Clean Architecture.
    pub fn new(wallet_repo: Arc<dyn WalletRepository>) -> Self {
        Self { wallet_repo }
    }

    /// Ejecuta el caso de uso para procesar un movimiento financiero.
    ///
    /// El proceso se apoya en el repositorio subyacente que asegura la atomicidad de la
    /// transacción y que el balance no sea menor a cero (en caso de que la implementación
    /// lo valide mediante constricciones en BD).
    ///
    /// # Argumentos
    ///
    /// * `wallet_id` - El identificador único de la billetera destino u origen.
    /// * `amount`    - La cantidad como logaritmo decimal continuo. Puede ser positivo (depósito) o negativo (retiro).
    ///
    /// # Retornos
    ///
    /// Devuelve un `Result<(), WalletError>`. Falla con `WalletError::NotFound` si la billetera no existe
    /// o `WalletError::InsufficientFunds` en caso de un balance no viable.
    #[tracing::instrument(name = "ProcessMovementUseCase::execute", skip(self))]
    pub async fn execute(
        &self,
        wallet_id: Uuid,
        amount: rust_decimal::Decimal,
    ) -> Result<(), WalletError> {
        self.wallet_repo.update_balance(wallet_id, amount).await
    }
}
