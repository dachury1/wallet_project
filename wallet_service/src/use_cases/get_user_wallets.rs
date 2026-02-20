use crate::domain::{
    entities::Wallet, error::WalletError, repository::WalletRepository, types::UserId,
};
use std::sync::Arc;

/// Casos de uso para obtener todas las billeteras asociadas a un usuario específico.
///
/// Este struct encapsula la lógica de buscar billeteras utilizando el repositorio,
/// abstrayendo detalles de implementación (como la base de datos).
///
/// # Examples
/// ```ignore
/// use wallet_service::use_cases::get_user_wallets::GetWalletsUseCase;
/// use wallet_service::domain::repository::MockWalletRepository;
/// use std::sync::Arc;
///
/// let repo = Arc::new(MockWalletRepository::new());
/// let use_case = GetWalletsUseCase::new(repo);
/// ```
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
    ///
    /// # Examples
    /// ```ignore
    /// use uuid::Uuid;
    /// let user_id = Uuid::new_v4();
    /// let wallets = use_case.execute(user_id).await.unwrap();
    /// ```
    #[tracing::instrument(name = "GetWalletsUseCase::execute", skip(self))]
    pub async fn execute(&self, user_id: UserId) -> Result<Vec<Wallet>, WalletError> {
        self.wallet_repo.find_by_user_id(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::MockWalletRepository;

    #[tokio::test]
    async fn test_get_wallets_success_empty() {
        let mut mock_repo = MockWalletRepository::new();
        let user_id = UserId::new();

        mock_repo
            .expect_find_by_user_id()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .returning(|_| Ok(vec![]));

        let use_case = GetWalletsUseCase::new(Arc::new(mock_repo));
        let result = use_case.execute(user_id).await;

        assert!(result.is_ok());
        let wallets = result.unwrap();
        assert!(wallets.is_empty());
    }

    #[tokio::test]
    async fn test_get_wallets_success_with_items() {
        let mut mock_repo = MockWalletRepository::new();
        let user_id = UserId::new();

        mock_repo
            .expect_find_by_user_id()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .returning(move |_| {
                Ok(vec![
                    Wallet::builder()
                        .user_id(user_id)
                        .currency("USD".to_string())
                        .label("Main Wallet".to_string())
                        .build()
                        .unwrap(),
                    Wallet::builder()
                        .user_id(user_id)
                        .currency("EUR".to_string())
                        .label("Euro Trip".to_string())
                        .build()
                        .unwrap(),
                ])
            });

        let use_case = GetWalletsUseCase::new(Arc::new(mock_repo));
        let result = use_case.execute(user_id).await;

        assert!(result.is_ok());
        let wallets = result.unwrap();
        assert_eq!(wallets.len(), 2);
        assert_eq!(wallets[0].currency, "USD");
        assert_eq!(wallets[1].currency, "EUR");
    }

    #[tokio::test]
    async fn test_get_wallets_repository_error() {
        let mut mock_repo = MockWalletRepository::new();
        let user_id = UserId::new();

        mock_repo
            .expect_find_by_user_id()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .returning(|_| Err(WalletError::RepositoryError("DB disconnected".to_string())));

        let use_case = GetWalletsUseCase::new(Arc::new(mock_repo));
        let result = use_case.execute(user_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            WalletError::RepositoryError(_) => (),
            _ => panic!("Expected RepositoryError"),
        }
    }
}
