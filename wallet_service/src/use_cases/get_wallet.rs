use crate::domain::{entities::Wallet, error::WalletError, repository::WalletRepository};
use std::sync::Arc;
use uuid::Uuid;

/// Casos de uso para obtener los detalles de una billetera en particular.
///
/// Encapsula la lógica necesaria para consultar una única billetera y
/// devolver un error `WalletError::NotFound` estándar si la base de datos
/// no arroja resultados.
#[derive(Clone)]
pub struct GetWalletUseCase {
    wallet_repo: Arc<dyn WalletRepository>,
}

impl GetWalletUseCase {
    /// Construye una nueva instancia de `GetWalletUseCase`.
    ///
    /// Se le inyecta una implementación de `WalletRepository` utilizando `Arc<dyn ...>`
    /// para permitir su uso seguro en entornos multihilo según las reglas de Clean Architecture.
    pub fn new(wallet_repo: Arc<dyn WalletRepository>) -> Self {
        Self { wallet_repo }
    }

    /// Ejecuta el caso de uso para obtener una billetera específica.
    ///
    /// # Argumentos
    ///
    /// * `wallet_id` - El identificador único (`Uuid`) de la billetera.
    ///
    /// # Retornos
    ///
    /// Devuelve un `Result<Wallet, WalletError>`. Si la billetera existe,
    /// se retorna satisfactoriamente. Si no existe, lanza un `WalletError::NotFound`.
    #[tracing::instrument(name = "GetWalletUseCase::execute", skip(self))]
    pub async fn execute(&self, wallet_id: Uuid) -> Result<Wallet, WalletError> {
        self.wallet_repo
            .find_by_id(wallet_id)
            .await?
            .ok_or(WalletError::NotFound(wallet_id))
    }
}
