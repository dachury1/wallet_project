use axum::{
    Router,
    routing::{get, post},
};

// Definicion de rutas para la API HTTP
pub fn routes() -> Router {
    Router::new()
        .route("/users", post(create_user))
        .route("/wallets", post(create_wallet).get(list_user_wallets))
        .route("/wallets/:id", get(get_wallet_details))
}

// Handler: Crear un usuario base
// POST /users
pub async fn create_user() {
    // TODO: Implementar logica de creacion de usuario
}

// Handler: Crear una nueva billetera para un usuario
// POST /wallets
// Header: x-user-id requerido
pub async fn create_wallet() {
    // TODO: Implementar logica de creacion de billetera
}

// Handler: Listar todas las billeteras del usuario actual
// GET /wallets
pub async fn list_user_wallets() {
    // TODO: Implementar logica para listar billeteras
}

// Handler: Ver saldo y detalles de una billetera especifica
// GET /wallets/{id}
pub async fn get_wallet_details() {
    // TODO: Implementar logica para obtener detalles de billetera
}
