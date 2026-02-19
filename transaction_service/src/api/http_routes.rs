use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::error::TransactionError;
use crate::use_cases::process_transaction::ProcessTransactionUseCase;

// Estado compartido de la aplicación
pub struct AppState {
    pub process_transaction_use_case: ProcessTransactionUseCase,
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/transactions", post(initiate_transaction))
        .route("/transactions/{id}", get(get_transaction_details))
        .route("/transactions/wallet/{wallet_id}", get(get_wallet_history))
        .with_state(state) // Inyectamos el estado (Casos de Uso)
}

// DTO de entrada para crear transacción
#[derive(Deserialize)]
pub struct CreateTransactionRequest {
    pub source_wallet_id: Option<Uuid>,
    pub dest_wallet_id: Uuid,
    pub amount: Decimal,
    pub correlation_id: Uuid,
}

// Handler: Iniciar un movimiento entre billeteras
// POST /transactions
pub async fn initiate_transaction(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = state
        .process_transaction_use_case
        .execute(
            payload.source_wallet_id,
            payload.dest_wallet_id,
            payload.amount,
            payload.correlation_id,
        )
        .await;

    match result {
        Ok(transaction) => Ok(Json(serde_json::json!({
            "status": "success",
            "data": transaction
        }))),
        Err(err) => {
            let status = match err {
                TransactionError::ValidationError(_) => StatusCode::BAD_REQUEST,
                TransactionError::InvalidAmount => StatusCode::BAD_REQUEST,
                TransactionError::SameWallet => StatusCode::BAD_REQUEST,
                TransactionError::NotFound(_) => StatusCode::NOT_FOUND,
                TransactionError::IdempotencyError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, err.to_string()))
        }
    }
}

// Handler: Ver detalle de una transaccion
// GET /transactions/{id}
// Nota: Aqui ocurre la magia: Busca la transaccion y llama por gRPC a Wallet
pub async fn get_transaction_details() -> impl axum::response::IntoResponse {
    // TODO: Implementar logica para obtener detalles y llamar a Wallet Service via gRPC
    (StatusCode::NOT_IMPLEMENTED, "Not Implemented Yet")
}

// Handler: Historial de movimientos de una billetera especifica (paginado)
// GET /transactions/wallet/{wallet_id}
pub async fn get_wallet_history() -> impl axum::response::IntoResponse {
    // TODO: Implementar logica de historial paginado
    (StatusCode::NOT_IMPLEMENTED, "Not Implemented Yet")
}
