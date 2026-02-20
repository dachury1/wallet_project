use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::response::ApiResponse;
use crate::use_cases::create_user::CreateUserUseCase;
use crate::use_cases::create_wallet::CreateWalletUseCase;
use crate::use_cases::get_user_wallets::GetWalletsUseCase;
use crate::use_cases::get_wallet::GetWalletUseCase;

pub struct AppState {
    pub create_user_use_case: CreateUserUseCase,
    pub create_wallet_use_case: CreateWalletUseCase,
    pub list_user_wallets_use_case: GetWalletsUseCase,
    pub get_wallet_details_use_case: GetWalletUseCase,
}
// Definicion de rutas para la API HTTP
pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/users", post(create_user))
        .route("/wallets", post(create_wallet).get(list_user_wallets))
        .route("/wallets/:id", get(get_wallet_details))
        .with_state(state)
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

// Handler: Crear un usuario base
// POST /users
pub async fn create_user(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let user = app_state
        .create_user_use_case
        .execute(payload.name, payload.email)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "id": user.id,
        "username": user.username,
        "email": user.email,
    }))))
}

#[derive(Deserialize)]
pub struct CreateWalletRequest {
    pub user_id: Uuid,
    pub currency: String,
    pub label: String,
}

// Handler: Crear una nueva billetera para un usuario
// POST /wallets
// Header: x-user-id requerido
pub async fn create_wallet(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CreateWalletRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let wallet = app_state
        .create_wallet_use_case
        .execute(payload.user_id, payload.currency, payload.label)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "id": wallet.id,
        "user_id": wallet.user_id,
        "currency": wallet.currency,
        "balance": wallet.balance,
        "label": wallet.label,
    }))))
}

// Handler: Listar todas las billeteras del usuario actual
// GET /wallets
pub async fn list_user_wallets(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let wallets = app_state
        .list_user_wallets_use_case
        .execute(user_id)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "wallets": wallets
    }))))
}

// Handler: Ver saldo y detalles de una billetera especifica
// GET /wallets/{id}
pub async fn get_wallet_details(
    State(app_state): State<Arc<AppState>>,
    Path(wallet_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    let wallet = app_state
        .get_wallet_details_use_case
        .execute(wallet_id)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "wallet": wallet
    }))))
}
