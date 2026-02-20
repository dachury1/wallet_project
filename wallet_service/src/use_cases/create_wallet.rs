use crate::domain::{
    entities::Wallet,
    error::WalletError,
    repository::{UserRepository, WalletRepository},
};
use std::sync::Arc;
use uuid::Uuid;
/// Caso de uso que gestiona la creación segura de una Wallet para un Usuario.
pub struct CreateWalletUseCase {
    wallet_repo: Arc<dyn WalletRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl CreateWalletUseCase {
    /// Inicializa las dependencias (Clean Architecture)
    pub fn new(wallet_repo: Arc<dyn WalletRepository>, user_repo: Arc<dyn UserRepository>) -> Self {
        Self {
            wallet_repo,
            user_repo,
        }
    }

    /// Ejecuta el caso de uso para crear una nueva Wallet.
    /// Valida que el usuario exista en BD antes de proveer una nueva Wallet.
    /// Utiliza el patrón Builder de `Wallet` para asegurar que el estado contenga valores válidos.
    pub async fn execute(
        &self,
        user_id: Uuid,
        currency: String,
        label: String,
    ) -> Result<Wallet, WalletError> {
        let user_option = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| WalletError::RepositoryError(e.to_string()))?; // Propagamos errores de BD

        if user_option.is_none() {
            return Err(WalletError::NotFound(user_id)); // Solo retornamos NotFound si la opción es None
        }

        // Construimos usando el patrón Builder (valida interiormente)
        let wallet = Wallet::builder()
            .user_id(user_id)
            .label(label)
            .currency(currency)
            .build()?;

        self.wallet_repo.create(wallet).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        entities::User,
        repository::{MockUserRepository, MockWalletRepository},
    };

    #[tokio::test]
    async fn test_create_wallet_success() {
        let mut mock_user_repo = MockUserRepository::new();
        let mut mock_wallet_repo = MockWalletRepository::new();
        let user_id = Uuid::new_v4();

        // Espera que el usuario exista
        mock_user_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(user_id))
            .returning(move |_| {
                Ok(Some(User {
                    id: user_id,
                    username: "test_user".into(),
                    email: "test@example.com".into(),
                    created_at: chrono::Utc::now(),
                }))
            });

        // Espera que se cree el wallet y reciba de vuelta
        mock_wallet_repo.expect_create().returning(|w| Ok(w));

        let use_case =
            CreateWalletUseCase::new(Arc::new(mock_wallet_repo), Arc::new(mock_user_repo));

        let result = use_case
            .execute(user_id, "USD".to_string(), "Main Wallet".to_string())
            .await;

        assert!(result.is_ok());
        let wallet = result.unwrap();
        assert_eq!(wallet.user_id, user_id);
        assert_eq!(wallet.currency, "USD");
        assert_eq!(wallet.label, "Main Wallet");
    }

    #[tokio::test]
    async fn test_create_wallet_user_not_found() {
        let mut mock_user_repo = MockUserRepository::new();
        let mock_wallet_repo = MockWalletRepository::new();
        let user_id = Uuid::new_v4();

        // El usuario no existe
        mock_user_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(user_id))
            .returning(|_| Ok(None));

        let use_case =
            CreateWalletUseCase::new(Arc::new(mock_wallet_repo), Arc::new(mock_user_repo));

        let result = use_case
            .execute(user_id, "USD".to_string(), "Main Wallet".to_string())
            .await;

        assert!(matches!(result, Err(WalletError::NotFound(id)) if id == user_id));
    }

    #[tokio::test]
    async fn test_create_wallet_invalid_currency() {
        let mut mock_user_repo = MockUserRepository::new();
        let mock_wallet_repo = MockWalletRepository::new();
        let user_id = Uuid::new_v4();

        mock_user_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq(user_id))
            .returning(move |_| {
                Ok(Some(User {
                    id: user_id,
                    username: "test_user".into(),
                    email: "test@example.com".into(),
                    created_at: chrono::Utc::now(),
                }))
            });

        let use_case =
            CreateWalletUseCase::new(Arc::new(mock_wallet_repo), Arc::new(mock_user_repo));

        // Pasando un currency en blanco/inválido (inválido por el builder)
        let result = use_case
            .execute(user_id, "XX".to_string(), "Main Wallet".to_string())
            .await;

        assert!(matches!(result, Err(WalletError::InvalidData(_))));
    }
}
