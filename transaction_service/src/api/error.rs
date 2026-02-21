use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::domain::error::TransactionError;

// Definimos un error unificado para la API del servicio de Transaction
pub struct ApiError(pub TransactionError);

// Permitimos convertir errores de dominio al ApiError implícitamente
impl From<TransactionError> for ApiError {
    fn from(err: TransactionError) -> Self {
        ApiError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            TransactionError::NotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            TransactionError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            TransactionError::InvalidAmount => (StatusCode::BAD_REQUEST, self.0.to_string()),
            TransactionError::SameWallet => (StatusCode::BAD_REQUEST, self.0.to_string()),
            TransactionError::InvalidState(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            TransactionError::IdempotencyError(_) => (StatusCode::CONFLICT, self.0.to_string()),
            TransactionError::RepositoryError(ref e) => {
                tracing::error!("Database Repository Error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            TransactionError::GatewayError(ref e) => {
                tracing::error!("Wallet Gateway Error: {}", e);
                (StatusCode::BAD_REQUEST, self.0.to_string())
            }
        };

        let body = Json(json!({
            "status": "error",
            "message": error_message,
        }));

        (status, body).into_response()
    }
}
