use crate::domain::entities::Transaction;
use crate::domain::error::TransactionError;
use crate::domain::repository::TransactionRepository;
use crate::infrastructure::persistence::models::TransactionModel;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repositorio de transacciones implementado para PostgreSQL.
///
/// Utiliza consultas SQL parametrizadas directas con `sqlx` (Runtime-checked).
/// Esta implementación favorece la legibilidad y simplicidad, utilizando `sqlx::query_as`
/// para mapear automáticamente los resultados a `TransactionModel`.
pub struct PostgresTransactionRepository {
    /// Pool de conexiones a la base de datos PostgreSQL.
    pool: PgPool,
}

impl PostgresTransactionRepository {
    /// Crea una nueva instancia del repositorio.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionRepository for PostgresTransactionRepository {
    /// Guarda una NUEVA transacción en la base de datos (INSERT).
    ///
    /// Se utiliza al inicio del flujo (Saga) para registrar la intención de pago
    /// antes de ejecutar cualquier lógica de negocio en otros servicios.
    ///
    /// # Errores
    /// Retorna `TransactionError::RepositoryError` si falla la conexión o la query SQL (ej. constraints).
    async fn save(&self, transaction: Transaction) -> Result<Transaction, TransactionError> {
        // Convertimos la entidad de dominio a nuestro modelo de persistencia (Infrastructure Layer)
        let model = TransactionModel::from(&transaction);

        let saved_model = sqlx::query_as::<_, TransactionModel>(
            r#"
            INSERT INTO transactions (
                id, source_wallet_id, destination_wallet_id, amount, status, transaction_type, created_at, correlation_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        // Bind directo de valores para prevenir SQL Injection.
        // El orden de los binds debe coincidir estrictamente con $1, $2, etc.
        .bind(model.id)
        .bind(model.source_wallet_id)
        .bind(model.destination_wallet_id)
        .bind(model.amount)
        .bind(model.status)
        .bind(model.transaction_type)
        .bind(model.created_at)
        .bind(model.correlation_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| TransactionError::RepositoryError(e.to_string()))?;

        // Mapeamos de vuelta al dominio
        Ok(saved_model.into())
    }

    /// Actualiza el estado de una transacción existente (UPDATE).
    ///
    /// Se utiliza para finalizar el proceso de pago (COMPLETED/FAILED/REVERSED).
    /// Solo actualizamos campos mutables, los detalles financieros (monto, wallets) son inmutables.
    async fn update(&self, transaction: Transaction) -> Result<Transaction, TransactionError> {
        let model = TransactionModel::from(&transaction);

        let updated_model = sqlx::query_as::<_, TransactionModel>(
            r#"
            UPDATE transactions 
            SET status = $1, transaction_type = $2 
            WHERE id = $3 
            RETURNING *
            "#,
        )
        .bind(model.status)
        .bind(model.transaction_type)
        .bind(model.id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| TransactionError::RepositoryError(e.to_string()))?;

        match updated_model {
            Some(m) => Ok(m.into()),
            None => Err(TransactionError::NotFound(transaction.id)),
        }
    }

    /// Busca una transacción por su ID único (UUID).
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, TransactionError> {
        let model_opt =
            sqlx::query_as::<_, TransactionModel>(r#"SELECT * FROM transactions WHERE id = $1"#)
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| TransactionError::RepositoryError(e.to_string()))?;

        if let Some(model) = model_opt {
            Ok(Some(model.into()))
        } else {
            Ok(None)
        }
    }

    /// Recupera historial de transacciones para una Wallet específica.
    ///
    /// Retorna una lista ordenada por fecha de creación descendente (lo más reciente primero).
    /// Incluye transacciones donde la wallet actúa como origen O destino.
    async fn find_by_wallet_id(
        &self,
        wallet_id: Uuid,
    ) -> Result<Vec<Transaction>, TransactionError> {
        let models = sqlx::query_as::<_, TransactionModel>(
            r#"
            SELECT * FROM transactions 
            WHERE source_wallet_id = $1 OR destination_wallet_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(wallet_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TransactionError::RepositoryError(e.to_string()))?;

        let mut transactions = Vec::new();
        for model in models {
            transactions.push(model.into());
        }

        Ok(transactions)
    }

    /// Busca por Correlation ID (ID de Idempotencia).
    ///
    /// Permite verificar si una solicitud ya fue procesada anteriormente para evitar duplicados.
    async fn find_by_correlation_id(
        &self,
        correlation_id: Uuid,
    ) -> Result<Option<Transaction>, TransactionError> {
        let model_opt = sqlx::query_as::<_, TransactionModel>(
            r#"SELECT * FROM transactions WHERE correlation_id = $1"#,
        )
        .bind(correlation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| TransactionError::RepositoryError(e.to_string()))?;

        if let Some(model) = model_opt {
            Ok(Some(model.into()))
        } else {
            Ok(None)
        }
    }

    /// Busca transacciones pendientes antiguas.
    async fn find_pending_older_than(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Transaction>, TransactionError> {
        let models = sqlx::query_as::<_, TransactionModel>(
            r#"
            SELECT * FROM transactions 
            WHERE status = 'PENDING' AND created_at < $1
            ORDER BY created_at ASC
            LIMIT 50
            "#,
        )
        .bind(timestamp)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TransactionError::RepositoryError(e.to_string()))?;

        let mut transactions = Vec::new();
        for model in models {
            transactions.push(model.into());
        }

        Ok(transactions)
    }
}
