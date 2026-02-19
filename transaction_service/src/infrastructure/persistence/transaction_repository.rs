use crate::domain::entities::Transaction;
use crate::domain::repository::TransactionRepository;
use crate::infrastructure::persistence::models::TransactionModel;
use async_trait::async_trait;
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

/// Repositorio de transacciones implementado para PostgreSQL.
///
/// Esta estructura actúa como el "Adaptador" en la Arquitectura Hexagonal/Clean Architecture.
/// Su responsabilidad es traducir las solicitudes del dominio (guardar, buscar)
/// en consultas SQL concretas utilizando la librería `sqlx`.
pub struct PostgresTransactionRepository {
    /// Pool de conexiones a la base de datos PostgreSQL.
    /// `sqlx::PgPool` maneja automáticamente la creación y reutilización de conexiones,
    /// lo cual es crítico para el rendimiento en aplicaciones de alta concurrencia.
    /// Es seguro de compartir entre hilos (implementa `Clone` de forma barata, apuntando al mismo pool interno).
    pool: PgPool,
}

impl PostgresTransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Implementación del puerto `TransactionRepository` definido en el dominio.
///
/// `#[async_trait]`:
/// Rust estándar (aún) no soporta funciones asíncronas en traits directamente de forma estable y ergonómica.
/// Esta macro transforma las funciones `async fn` en funciones que devuelven un `BoxFuture` (puntero en el heap),
/// permitiendo polimorfismo dinámico (`dyn TransactionRepository`).
#[async_trait]
impl TransactionRepository for PostgresTransactionRepository {
    /// Guarda una transacción en la base de datos.
    ///
    /// Flujo:
    /// 1. Recibe una entidad de dominio `Transaction`.
    /// 2. La convierte a un modelo de infraestructura `TransactionModel` (DTO) para adaptar tipos (ej. Enums a Strings).
    /// 3. Ejecuta una consulta INSERT mediante `sqlx`.
    async fn save(&self, transaction: Transaction) -> Result<Transaction, Box<dyn Error>> {
        // Convertimos la entidad de dominio a nuestro modelo de persistencia (Infrastructure Layer)
        // Esto desacopla cómo usamos los datos (Domain) de cómo los guardamos (DB).
        let model = TransactionModel::from(&transaction);

        // `sqlx::query_as`:
        // Permite ejecutar una consulta SQL pura y mapear el resultado automáticamente a una estructura (`TransactionModel`)
        // que implemente el trait `sqlx::FromRow`.
        // Usamos la variante sin macro (query_as vs query_as!) para evitar validaciones en tiempo de compilación
        // que requieren una BD activa, facilitando el desarrollo inicial.
        let saved_model = sqlx::query_as::<_, TransactionModel>(
            r#"
            INSERT INTO transactions (
                id, source_wallet_id, destination_wallet_id, amount, status, transaction_type, created_at, correlation_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(model.id)
        .bind(model.source_wallet_id)
        .bind(model.destination_wallet_id)
        .bind(model.amount)
        .bind(model.status)
        .bind(model.transaction_type)
        .bind(model.created_at)
        .bind(model.correlation_id)
        .fetch_one(&self.pool) // Ejecuta y espera exactamente una fila. Falla si no devuelve nada.
        .await?;

        // Convertimos de vuelta al dominio antes de retornar.
        // `try_into()` usa nuestra implementación de `TryFrom` que valida los strings de enums.
        Ok(saved_model.try_into()?)
    }

    /// Busca una transacción por su ID único (UUID).
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>> {
        // `fetch_optional`: Devuelve `Option<T>`, manejando el caso donde no existe la fila (Ok(None)).
        let model_opt =
            sqlx::query_as::<_, TransactionModel>(r#"SELECT * FROM transactions WHERE id = $1"#)
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;

        if let Some(model) = model_opt {
            Ok(Some(model.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// Busca todas las transacciones asociadas a una Wallet (como origen o destino).
    async fn find_by_wallet_id(&self, wallet_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>> {
        // Buscamos transacciones donde la wallet sea origen O destino.
        let models = sqlx::query_as::<_, TransactionModel>(
            r#"
            SELECT * FROM transactions 
            WHERE source_wallet_id = $1 OR destination_wallet_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(wallet_id)
        .fetch_all(&self.pool) // Devuelve un Vector con todas las coincidencias.
        .await?;

        // Mapeamos el Vector de modelos a un Vector de entidades de dominio.
        // Si falla alguna conversión (ej. status corrupto en DB), retornamos error.
        let mut transactions = Vec::new();
        for model in models {
            transactions.push(model.try_into()?);
        }

        Ok(transactions)
    }
}
