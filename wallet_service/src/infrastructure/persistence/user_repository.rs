use crate::domain::entities::User;
use crate::domain::error::UserError;
use crate::domain::repository::UserRepository;
use crate::domain::types::UserId;
use crate::infrastructure::persistence::models::UserModel;
use async_trait::async_trait;
use sqlx::PgPool;

/// Repositorio de Usuarios basado en PostgreSQL.
///
/// Implementa la interfaz de dominio `UserRepository` utilizando `sqlx` para
/// interactuar con la base de datos de manera asíncrona y segura (params binding).
///
/// Esta capa actúa como un adaptador:
/// - Recibe entidades de dominio (`User`).
/// - Las convierte a modelos de persistencia (`UserModel`).
/// - Ejecuta consultas SQL.
/// - Devuelve entidades de dominio, aislando al núcleo de la lógica de la DB.
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    /// Crea una nueva instancia inyectando el pool de conexiones.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    /// Busca un usuario por su ID único.
    ///
    /// Retorna `None` si el usuario no existe, en lugar de un error.
    /// Utiliza `fetch_optional` para manejar elegantemente el caso de "no encontrado".
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, UserError> {
        // Consultamos el modelo de base de datos
        let model_opt = sqlx::query_as::<_, UserModel>(
            r#"
            SELECT * FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserError::RepositoryError(e.to_string()))?;

        // Mapeamos Option<UserModel> -> Option<User>
        // Si hay dato (Some), usamos `.into()` para convertir UserModel a User
        // gracias a la implementación de `From` en models.rs.
        Ok(model_opt.map(|m| m.into()))
    }

    /// Persiste un nuevo usuario en la base de datos.
    ///
    /// Realiza un INSERT y retorna la entidad creada (incluyendo datos generados por DB si los hubiera,
    /// aunque en este caso el ID y fechas ya vienen pre-calculados o se manejan aquí).
    async fn create(&self, user: User) -> Result<User, UserError> {
        // Convertimos Entidad -> Modelo
        // Usamos `UserModel::from(user)` que consume la entidad (move semantics)
        // evitando clonar los Strings (username, email) por eficiencia.
        let model = UserModel::from(user);

        // Clonamos datos necesarios para manejo de errores antes de mover 'model' al query
        let username_for_error = model.username.clone();
        let email_for_error = model.email.clone();

        let saved_model = sqlx::query_as::<_, UserModel>(
            r#"
            INSERT INTO users (
                id, username, email, created_at
            )
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(model.id)
        .bind(model.username)
        .bind(model.email)
        .bind(model.created_at) // Importante: persistir la fecha de creación
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            // Manejo básico de duplicados (Postgres error 23505)
            // En un caso real, inspeccionaríamos e.code() para ser más precisos
            let error_msg = e.to_string();
            if error_msg.contains("users_username_key") {
                UserError::UsernameTaken(username_for_error)
            } else if error_msg.contains("users_email_key") {
                UserError::EmailTaken(email_for_error)
            } else {
                UserError::RepositoryError(error_msg)
            }
        })?;

        // Convertimos Modelo -> Entidad
        // El `.into()` funciona automáticamente porque implementamos `From<UserModel> for User`.
        Ok(saved_model.into())
    }

    /// Verifica eficientemente si un nombre de usuario ya está ocupado.
    ///
    /// Optimizado para NO traer datos de la red (`SELECT 1`), solo verifica existencia.
    /// Útil para validaciones rápidas antes de intentar crear un usuario.
    async fn exists_by_username(&self, username: &str) -> Result<bool, UserError> {
        let result = sqlx::query(
            r#"
            SELECT 1 FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserError::RepositoryError(e.to_string()))?;

        // Si fetch_optional retorna Some(...), significa que encontró una fila -> existe.
        Ok(result.is_some())
    }
}
