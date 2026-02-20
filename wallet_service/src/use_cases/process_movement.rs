use crate::domain::error::WalletError;
use crate::domain::repository::WalletRepository;
use crate::domain::types::WalletId;
use std::sync::Arc;

/// Casos de uso para procesar movimientos (depósitos/retiros) en una billetera.
///
/// Encapsula la lógica de llamar al repositorio para realizar la actualización
/// atómica del balance en la base de datos.
///
/// # Examples
/// ```ignore
/// use wallet_service::use_cases::process_movement::ProcessMovementUseCase;
/// use wallet_service::domain::repository::MockWalletRepository;
/// use std::sync::Arc;
///
/// let repo = Arc::new(MockWalletRepository::new());
/// let use_case = ProcessMovementUseCase::new(repo);
/// ```
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
    ///
    /// # Examples
    /// ```ignore
    /// use uuid::Uuid;
    /// use rust_decimal_macros::dec;
    ///
    /// let wallet_id = Uuid::new_v4();
    /// let amount = dec!(50.0);
    /// use_case.execute(wallet_id, amount).await.unwrap();
    /// ```
    #[tracing::instrument(name = "ProcessMovementUseCase::execute", skip(self))]
    pub async fn execute(
        &self,
        wallet_id: WalletId,
        amount: rust_decimal::Decimal,
    ) -> Result<(), WalletError> {
        self.wallet_repo.update_balance(wallet_id, amount).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::MockWalletRepository;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_process_movement_success() {
        let mut mock_repo = MockWalletRepository::new();
        let wallet_id = WalletId::new();
        let amount = Decimal::from_str("150.50").unwrap();

        mock_repo
            .expect_update_balance()
            .with(
                mockall::predicate::eq(wallet_id),
                mockall::predicate::eq(amount),
            )
            .times(1)
            .returning(|_, _| Ok(()));

        let use_case = ProcessMovementUseCase::new(Arc::new(mock_repo));
        let result = use_case.execute(wallet_id, amount).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_movement_not_found() {
        let mut mock_repo = MockWalletRepository::new();
        let wallet_id = WalletId::new();
        let amount = Decimal::from_str("150.50").unwrap();

        mock_repo
            .expect_update_balance()
            .with(
                mockall::predicate::eq(wallet_id),
                mockall::predicate::eq(amount),
            )
            .times(1)
            .returning(|id, _| Err(WalletError::NotFound(id)));

        let use_case = ProcessMovementUseCase::new(Arc::new(mock_repo));
        let result = use_case.execute(wallet_id, amount).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            WalletError::NotFound(id) => assert_eq!(id, wallet_id),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_process_movement_insufficient_funds() {
        let mut mock_repo = MockWalletRepository::new();
        let wallet_id = WalletId::new();
        let amount = Decimal::from_str("-150.50").unwrap();

        mock_repo
            .expect_update_balance()
            .with(
                mockall::predicate::eq(wallet_id),
                mockall::predicate::eq(amount),
            )
            .times(1)
            .returning(move |id, _| Err(WalletError::InsufficientFunds(id)));

        let use_case = ProcessMovementUseCase::new(Arc::new(mock_repo));
        let result = use_case.execute(wallet_id, amount).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            WalletError::InsufficientFunds(id) => assert_eq!(id, wallet_id),
            _ => panic!("Expected InsufficientFunds error"),
        }
    }
}
