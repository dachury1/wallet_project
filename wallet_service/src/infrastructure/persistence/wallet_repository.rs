use crate::domain::entities::Wallet;
use crate::domain::error::WalletError;
use crate::domain::repository::WalletRepository;
use crate::infrastructure::persistence::models::WalletModel;
use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// Repositorio de Billeteras basado en PostgreSQL.
pub struct PostgresWalletRepository {
    pool: PgPool,
}

impl PostgresWalletRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WalletRepository for PostgresWalletRepository {
    /// Busca una billetera por su ID.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Wallet>, WalletError> {
        let model_opt = sqlx::query_as::<_, WalletModel>(
            r#"
            SELECT * FROM wallets
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WalletError::RepositoryError(e.to_string()))?;

        Ok(model_opt.map(|m| m.into()))
    }

    /// Busca todas las billeteras asociadas a un usuario.
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Wallet>, WalletError> {
        let models = sqlx::query_as::<_, WalletModel>(
            r#"
            SELECT * FROM wallets
            WHERE user_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WalletError::RepositoryError(e.to_string()))?;

        // Convertimos Vec<WalletModel> -> Vec<Wallet>
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    /// Crea una nueva billetera.
    async fn create(&self, wallet: Wallet) -> Result<Wallet, WalletError> {
        let model = WalletModel::from(wallet);

        let saved_model = sqlx::query_as::<_, WalletModel>(
            r#"
            INSERT INTO wallets (
                id, user_id, label, balance, currency, version, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(model.id)
        .bind(model.user_id)
        .bind(model.label)
        .bind(model.balance)
        .bind(model.currency)
        .bind(model.version)
        .bind(model.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WalletError::RepositoryError(e.to_string()))?;

        Ok(saved_model.into())
    }

    /// Actualiza el balance de forma atómica.
    ///
    /// Se suma (o resta si es negativo) el `amount` al balance actual.
    async fn update_balance(&self, id: Uuid, amount: Decimal) -> Result<(), WalletError> {
        // Ejecutamos UPDATE directo para atomicidad.
        // Incrementamos la versión para optimistic locking implícito.
        let result = sqlx::query(
            r#"
            UPDATE wallets 
            SET balance = balance + $1,
                version = version + 1
            WHERE id = $2 
            "#,
        )
        .bind(amount)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            // Check constraint violation (e.g. balance < 0 constraint)
            if e.to_string().contains("balance_chk") || e.to_string().contains("positive_balance") {
                return WalletError::InsufficientFunds(id);
            }
            WalletError::RepositoryError(e.to_string())
        })?;

        if result.rows_affected() == 0 {
            return Err(WalletError::NotFound(id));
        }

        Ok(())
    }
}
