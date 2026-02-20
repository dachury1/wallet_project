use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::domain::error::{UserError, WalletError};

// Definimos un error unificado para la API del servicio de Wallet
pub enum ApiError {
    User(UserError),
    Wallet(WalletError),
}

// Permitimos convertir errores de dominio al ApiError implícitamente
impl From<UserError> for ApiError {
    fn from(err: UserError) -> Self {
        ApiError::User(err)
    }
}

impl From<WalletError> for ApiError {
    fn from(err: WalletError) -> Self {
        ApiError::Wallet(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::User(e) => match e {
                UserError::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
                UserError::UsernameTaken(_) => (StatusCode::CONFLICT, e.to_string()),
                UserError::EmailTaken(_) => (StatusCode::CONFLICT, e.to_string()),
                UserError::InvalidData(_) => (StatusCode::BAD_REQUEST, e.to_string()),
                UserError::RepositoryError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                ),
            },
            ApiError::Wallet(e) => match e {
                WalletError::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
                WalletError::UserNotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
                WalletError::InvalidData(_) => (StatusCode::BAD_REQUEST, e.to_string()),
                WalletError::InsufficientFunds(_) => (StatusCode::BAD_REQUEST, e.to_string()),
                WalletError::ConcurrencyError(_) => (StatusCode::CONFLICT, e.to_string()),
                WalletError::RepositoryError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                ),
            },
        };

        let body = Json(json!({
            "status": "error",
            "message": error_message,
        }));

        (status, body).into_response()
    }
}
