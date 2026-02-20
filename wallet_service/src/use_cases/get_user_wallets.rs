use crate::domain::{entities::Wallet, error::WalletError, repository::WalletRepository};
use std::sync::Arc;
use uuid::Uuid;

/// Casos de uso para obtener todas las billeteras asociadas a un usuario específico.
///
/// Este struct encapsula la lógica de buscar billeteras utilizando el repositorio,
/// abstrayendo detalles de implementación (como la base de datos).
#[derive(Clone)]
pub struct GetWalletsUseCase {
    wallet_repo: Arc<dyn WalletRepository>,
}

impl GetWalletsUseCase {
    /// Construye una nueva instancia de `GetWalletsUseCase`.
    ///
    /// Se le inyecta una implementación de `WalletRepository` utilizando `Arc<dyn ...>`
    /// para permitir su uso seguro en entornos multihilo según las reglas de Clean Architecture.
    pub fn new(wallet_repo: Arc<dyn WalletRepository>) -> Self {
        Self { wallet_repo }
    }

    /// Ejecuta el caso de uso para buscar billeteras por ID de usuario.
    ///
    /// # Argumentos
    ///
    /// * `user_id` - El identificador único (`Uuid`) del usuario.
    ///
    /// # Retornos
    ///
    /// Devuelve un `Result<Vec<Wallet>, WalletError>`. Retorna el listado de
    /// billeteras en caso de éxito o un error de billetera.
    #[tracing::instrument(name = "GetWalletsUseCase::execute", skip(self))]
    pub async fn execute(&self, user_id: Uuid) -> Result<Vec<Wallet>, WalletError> {
        self.wallet_repo.find_by_user_id(user_id).await
    }
}
