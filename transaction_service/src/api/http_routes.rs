use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::error::TransactionError;
use crate::use_cases::get_transaction_details::GetTransactionDetailsUseCase;
use crate::use_cases::get_wallet_history::GetWalletHistoryUseCase;
use crate::use_cases::process_transaction::ProcessTransactionUseCase;

// Estado compartido de la aplicación
pub struct AppState {
    pub process_transaction_use_case: ProcessTransactionUseCase,
    pub get_transaction_details_use_case: GetTransactionDetailsUseCase,
    pub get_wallet_history_use_case: GetWalletHistoryUseCase,
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/transactions", post(initiate_transaction))
        .route("/transactions/:id", get(get_transaction_details))
        .route("/transactions/wallet/:wallet_id", get(get_wallet_history))
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
// GET /transactions/:id
pub async fn get_transaction_details(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = state.get_transaction_details_use_case.execute(id).await;

    match result {
        Ok(transaction) => Ok(Json(serde_json::json!({
            "status": "success",
            "data": transaction
        }))),
        Err(err) => {
            let status = match err {
                TransactionError::NotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, err.to_string()))
        }
    }
}

// Handler: Historial de movimientos de una billetera especifica
// GET /transactions/wallet/:wallet_id
pub async fn get_wallet_history(
    State(state): State<Arc<AppState>>,
    Path(wallet_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = state.get_wallet_history_use_case.execute(wallet_id).await;

    match result {
        Ok(transactions) => Ok(Json(serde_json::json!({
            "status": "success",
            "data": transactions
        }))),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}
