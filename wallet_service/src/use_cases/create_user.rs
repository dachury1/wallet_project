use crate::domain::{entities::User, error::UserError, repository::UserRepository};
use std::sync::Arc;

/// Caso de Uso: Crear un nuevo Usuario.
///
/// Este struct orquesta el flujo para registrar un nuevo usuario en el sistema.
/// Implementa las reglas de negocio necesarias antes de interactuar con la capa
/// de persistencia (repositorio).
///
/// # Examples
/// ```ignore
/// use wallet_service::use_cases::create_user::CreateUserUseCase;
/// use wallet_service::domain::repository::MockUserRepository;
/// use std::sync::Arc;
///
/// let repo = Arc::new(MockUserRepository::new());
/// let use_case = CreateUserUseCase::new(repo);
/// ```
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
    ///
    /// # Examples
    /// ```ignore
    /// let result = use_case.execute("alice".to_string(), "alice@example.com".to_string()).await;
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::MockUserRepository;

    #[tokio::test]
    async fn test_create_user_success() {
        let mut mock_repo = MockUserRepository::new();

        let username = "new_user".to_string();
        let email = "test@example.com".to_string();

        // Expect the username not to be taken
        mock_repo
            .expect_exists_by_username()
            .with(mockall::predicate::eq(username.clone()))
            .times(1)
            .returning(|_| Ok(false));

        // Expect the creation of the user to succeed
        mock_repo.expect_create().times(1).returning(|u| Ok(u));

        let use_case = CreateUserUseCase::new(Arc::new(mock_repo));

        let result = use_case.execute(username.clone(), email.clone()).await;

        assert!(result.is_ok());
        let created_user = result.unwrap();
        assert_eq!(created_user.username, username);
        assert_eq!(created_user.email, email);
    }

    #[tokio::test]
    async fn test_create_user_username_taken() {
        let mut mock_repo = MockUserRepository::new();

        let username = "existing_user".to_string();
        let email = "test@example.com".to_string();

        // Expect the username to be taken
        mock_repo
            .expect_exists_by_username()
            .with(mockall::predicate::eq(username.clone()))
            .times(1)
            .returning(|_| Ok(true));

        let use_case = CreateUserUseCase::new(Arc::new(mock_repo));

        let result = use_case.execute(username.clone(), email.clone()).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            UserError::UsernameTaken(taken_name) => assert_eq!(taken_name, username),
            _ => panic!("Expected UsernameTaken error"),
        }
    }

    #[tokio::test]
    async fn test_create_user_invalid_data() {
        let mut mock_repo = MockUserRepository::new();

        let username = "".to_string(); // Invalid empty username
        let email = "test@example.com".to_string();

        // Expect the username not to be taken
        mock_repo
            .expect_exists_by_username()
            .with(mockall::predicate::eq(username.clone()))
            .times(1)
            .returning(|_| Ok(false));

        let use_case = CreateUserUseCase::new(Arc::new(mock_repo));

        let result = use_case.execute(username.clone(), email.clone()).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            UserError::InvalidData(_) => (),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[tokio::test]
    async fn test_create_user_repository_error_on_exists() {
        let mut mock_repo = MockUserRepository::new();

        let username = "new_user".to_string();
        let email = "test@example.com".to_string();

        // Simulate DB failure
        mock_repo
            .expect_exists_by_username()
            .with(mockall::predicate::eq(username.clone()))
            .times(1)
            .returning(|_| Err(UserError::RepositoryError("DB disconnected".to_string())));

        let use_case = CreateUserUseCase::new(Arc::new(mock_repo));

        let result = use_case.execute(username.clone(), email.clone()).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            UserError::RepositoryError(_) => (),
            _ => panic!("Expected RepositoryError"),
        }
    }
}
