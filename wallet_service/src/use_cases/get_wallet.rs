use crate::domain::{
    entities::Wallet, error::WalletError, repository::WalletRepository, types::WalletId,
};
use std::sync::Arc;

/// Casos de uso para obtener los detalles de una billetera en particular.
///
/// Encapsula la lógica necesaria para consultar una única billetera y
/// devolver un error `WalletError::NotFound` estándar si la base de datos
/// no arroja resultados.
///
/// # Examples
/// ```ignore
/// use wallet_service::use_cases::get_wallet::GetWalletUseCase;
/// use wallet_service::domain::repository::MockWalletRepository;
/// use std::sync::Arc;
///
/// let repo = Arc::new(MockWalletRepository::new());
/// let use_case = GetWalletUseCase::new(repo);
/// ```
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
    ///
    /// # Examples
    /// ```ignore
    /// use uuid::Uuid;
    /// let wallet_id = Uuid::new_v4();
    /// let wallet = use_case.execute(wallet_id).await.unwrap();
    /// ```
    #[tracing::instrument(name = "GetWalletUseCase::execute", skip(self))]
    pub async fn execute(&self, wallet_id: WalletId) -> Result<Wallet, WalletError> {
        self.wallet_repo
            .find_by_id(wallet_id)
            .await?
            .ok_or(WalletError::NotFound(wallet_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{repository::MockWalletRepository, types::UserId};

    #[tokio::test]
    async fn test_get_wallet_success() {
        let mut mock_repo = MockWalletRepository::new();
        let wallet_id = WalletId::new();
        let user_id = UserId::new();

        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(wallet_id))
            .times(1)
            .returning(move |_| {
                Ok(Some(
                    Wallet::builder()
                        .user_id(user_id)
                        .currency("USD".to_string())
                        .label("Main Wallet".to_string())
                        .build()
                        .unwrap(),
                ))
            });

        let use_case = GetWalletUseCase::new(Arc::new(mock_repo));
        let result = use_case.execute(wallet_id).await;

        assert!(result.is_ok());
        let wallet = result.unwrap();
        assert_eq!(wallet.user_id, user_id);
        assert_eq!(wallet.currency, "USD");
    }

    #[tokio::test]
    async fn test_get_wallet_not_found() {
        let mut mock_repo = MockWalletRepository::new();
        let wallet_id = WalletId::new();

        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(wallet_id))
            .times(1)
            .returning(|_| Ok(None));

        let use_case = GetWalletUseCase::new(Arc::new(mock_repo));
        let result = use_case.execute(wallet_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            WalletError::NotFound(id) => assert_eq!(id, wallet_id),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_wallet_repository_error() {
        let mut mock_repo = MockWalletRepository::new();
        let wallet_id = WalletId::new();

        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(wallet_id))
            .times(1)
            .returning(|_| Err(WalletError::RepositoryError("DB disconnected".to_string())));

        let use_case = GetWalletUseCase::new(Arc::new(mock_repo));
        let result = use_case.execute(wallet_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            WalletError::RepositoryError(_) => (),
            _ => panic!("Expected RepositoryError"),
        }
    }
}
