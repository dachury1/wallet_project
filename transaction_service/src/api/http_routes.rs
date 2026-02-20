use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::response::ApiResponse;

use crate::use_cases::get_transaction_details::GetTransactionDetailsUseCase;
use crate::use_cases::get_wallet_history::GetWalletHistoryUseCase;
use crate::use_cases::process_transaction::ProcessTransactionUseCase;

use crate::domain::types::{TransactionId, WalletId};

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
#[derive(Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    pub source_wallet_id: Option<Uuid>,
    pub dest_wallet_id: Uuid,
    pub amount: Decimal,
    pub correlation_id: Uuid,
}

// Handler: Iniciar un movimiento entre billeteras
// POST /transactions
#[utoipa::path(
    post,
    path = "/transactions",
    request_body = CreateTransactionRequest,
    responses(
        (status = 200, description = "Transacción iniciada", body = inline(crate::api::response::ApiResponse<serde_json::Value>))
    )
)]
pub async fn initiate_transaction(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let transaction = state
        .process_transaction_use_case
        .execute(
            payload.source_wallet_id.map(WalletId),
            WalletId(payload.dest_wallet_id),
            payload.amount,
            payload.correlation_id,
        )
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!(transaction))))
}

// Handler: Ver detalle de una transaccion
// GET /transactions/{id}
#[utoipa::path(
    get,
    path = "/transactions/{id}",
    responses(
        (status = 200, description = "Detalle de la transacción", body = inline(crate::api::response::ApiResponse<serde_json::Value>))
    ),
    params(
        ("id" = Uuid, Path, description = "ID de la transacción")
    )
)]
pub async fn get_transaction_details(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let transaction = state
        .get_transaction_details_use_case
        .execute(TransactionId(id))
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!(transaction))))
}

// Handler: Historial de movimientos de una billetera especifica
// GET /transactions/wallet/{wallet_id}
#[utoipa::path(
    get,
    path = "/transactions/wallet/{wallet_id}",
    responses(
        (status = 200, description = "Historial de movimientos", body = inline(crate::api::response::ApiResponse<serde_json::Value>))
    ),
    params(
        ("wallet_id" = Uuid, Path, description = "ID de la billetera")
    )
)]
pub async fn get_wallet_history(
    State(state): State<Arc<AppState>>,
    Path(wallet_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let transactions = state
        .get_wallet_history_use_case
        .execute(WalletId(wallet_id))
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!(transactions))))
}
