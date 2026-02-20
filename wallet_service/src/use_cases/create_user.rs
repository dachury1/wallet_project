use crate::domain::{entities::User, error::UserError, repository::UserRepository};
use std::sync::Arc;

/// Caso de Uso: Crear un nuevo Usuario.
///
/// Este struct orquesta el flujo para registrar un nuevo usuario en el sistema.
/// Implementa las reglas de negocio necesarias antes de interactuar con la capa
/// de persistencia (repositorio).
pub struct CreateUserUseCase {
    /// Dependencia inyectada del repositorio de usuarios.
    /// Se utiliza `Arc` para permitir que el caso de uso sea compartido (clonado
    /// a bajo costo) entre múltiples hilos u operaciones asíncronas con seguridad.
    user_repo: Arc<dyn UserRepository>,
}

impl CreateUserUseCase {
    /// Inicializa una nueva instancia del caso de uso inyectando el repositorio
    /// que encapsula el acceso a datos.
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    /// Ejecuta el proceso de creación de usuario.
    ///
    /// # Flujo de negocio:
    /// 1. Verifica si el `username` solicitado ya existe en la base de datos para prevenir duplicados.
    /// 2. Invoca la lógica de construcción del dominio (`User::new()`) que valida
    ///    reglas fundamentales (ej: strings no vacíos) y genera el ID temporal.
    /// 3. Delega al repositorio la persistencia de la entidad válida creada.
    ///
    /// # Retorna
    /// - `Ok(User)`: Si el flujo tuvo éxito y se persistió el usuario correctamente.
    /// - `Err(UserError)`: Contiene el contexto detallado de la falla.
    ///     * `UsernameTaken` si el nombre ya está registrado.
    ///     * `InvalidData` si el usuario no pasó las reglas del dominio al crearse.
    ///     * `RepositoryError` ante fallos de conexión o consultas en la base de datos.
    pub async fn execute(&self, username: String, email: String) -> Result<User, UserError> {
        // Valida que no existan duplicados antes de instanciar un nuevo usuario.
        if self.user_repo.exists_by_username(&username).await? {
            return Err(UserError::UsernameTaken(username));
        }

        // Constructor de la Entidad: Generación de IDs y validación básica
        let user = User::new(username, email)?;

        // Persistencia
        self.user_repo.create(user).await
    }
}
