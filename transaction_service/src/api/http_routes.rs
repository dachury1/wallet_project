use axum::{
    routing::{get, post},
    Router,
};

pub fn routes() -> Router {
    Router::new()
        .route("/transactions", post(initiate_transaction))
        .route("/transactions/:id", get(get_transaction_details))
        .route("/transactions/wallet/:wallet_id", get(get_wallet_history))
}

// Handler: Iniciar un movimiento entre billeteras
// POST /transactions
pub async fn initiate_transaction() {
    // TODO: Implementar logica de inicio de transaccion
}

// Handler: Ver detalle de una transaccion
// GET /transactions/{id}
// Nota: Aqui ocurre la magia: Busca la transaccion y llama por gRPC a Wallet
pub async fn get_transaction_details() {
    // TODO: Implementar logica para obtener detalles y llamar a Wallet Service via gRPC
}

// Handler: Historial de movimientos de una billetera especifica (paginado)
// GET /transactions/wallet/{wallet_id}
pub async fn get_wallet_history() {
    // TODO: Implementar logica de historial paginado
}
