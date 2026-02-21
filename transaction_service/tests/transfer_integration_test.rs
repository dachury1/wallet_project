use async_trait::async_trait;
use axum::extract::State;
use axum::Json;
use chrono::{DateTime, Utc};
use mockall::mock;
use mockall::predicate::*;
use rust_decimal::Decimal;
use std::sync::Arc;
use transaction_service::api::http_routes::{
    initiate_transaction, AppState, CreateTransactionRequest,
};
use transaction_service::api::response::ApiResponse;
use transaction_service::domain::entities::{Transaction, TransactionStatus};
use transaction_service::domain::error::TransactionError;
use transaction_service::domain::gateways::WalletGateway;
use transaction_service::domain::repository::TransactionRepository;
use transaction_service::domain::types::{TransactionId, WalletId};
use transaction_service::use_cases::get_transaction_details::GetTransactionDetailsUseCase;
use transaction_service::use_cases::get_wallet_history::GetWalletHistoryUseCase;
use transaction_service::use_cases::process_transaction::ProcessTransactionUseCase;
use uuid::Uuid;

mock! {
    pub TransactionRepositoryImpl {}

    #[async_trait]
    impl TransactionRepository for TransactionRepositoryImpl {
        async fn save(&self, transaction: Transaction) -> Result<Transaction, TransactionError>;
        async fn update(&self, transaction: Transaction) -> Result<Transaction, TransactionError>;
        async fn find_by_id(&self, id: TransactionId) -> Result<Option<Transaction>, TransactionError>;
        async fn find_by_wallet_id(&self, wallet_id: WalletId) -> Result<Vec<Transaction>, TransactionError>;
        async fn find_by_correlation_id(&self, correlation_id: Uuid) -> Result<Option<Transaction>, TransactionError>;
        async fn find_pending_older_than(&self, timestamp: DateTime<Utc>) -> Result<Vec<Transaction>, TransactionError>;
    }
}

mock! {
    pub WalletGatewayImpl {}

    #[async_trait]
    impl WalletGateway for WalletGatewayImpl {
        async fn process_movement(&self, transaction: &Transaction) -> Result<bool, TransactionError>;
    }
}

#[tokio::test]
async fn test_successful_transfer_updates_both_wallets() {
    // Arrange
    let mut mock_repo = MockTransactionRepositoryImpl::new();
    let mut mock_gateway = MockWalletGatewayImpl::new();

    let source_wallet_uuid = Uuid::new_v4();
    let dest_wallet_uuid = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let amount = Decimal::from(100);

    mock_repo
        .expect_find_by_correlation_id()
        .with(eq(correlation_id))
        .times(1)
        .returning(|_| Ok(None));

    mock_repo.expect_save().times(1).returning(|tx| Ok(tx));

    mock_gateway
        .expect_process_movement()
        .times(1)
        .returning(|_| Ok(true));

    mock_repo
        .expect_update()
        .withf(|tx: &Transaction| tx.status() == TransactionStatus::COMPLETED)
        .times(1)
        .returning(|tx| Ok(tx));

    let process_transaction_uc =
        ProcessTransactionUseCase::new(Arc::new(mock_repo), Arc::new(mock_gateway));
    let get_tx_uc =
        GetTransactionDetailsUseCase::new(Arc::new(MockTransactionRepositoryImpl::new()));
    let get_history_uc =
        GetWalletHistoryUseCase::new(Arc::new(MockTransactionRepositoryImpl::new()));

    let state = Arc::new(AppState {
        process_transaction_use_case: process_transaction_uc,
        get_transaction_details_use_case: get_tx_uc,
        get_wallet_history_use_case: get_history_uc,
    });

    let payload = CreateTransactionRequest {
        source_wallet_id: Some(source_wallet_uuid),
        dest_wallet_id: dest_wallet_uuid,
        amount,
        correlation_id,
    };

    // Act
    let result = initiate_transaction(State(state), Json(payload)).await;

    // Assert
    assert!(result.is_ok(), "El Request debe ser exitoso");

    let json_response: Json<ApiResponse<serde_json::Value>> = match result {
        Ok(r) => r,
        Err(_) => panic!("Expected Ok"),
    };
    let body = json_response.0;

    assert_eq!(body.status, "success");
    let tx_data = body.data;
    assert_eq!(tx_data["status"], "COMPLETED");
}

#[tokio::test]
async fn test_failed_transfer_rolls_back_source_wallet() {
    // Arrange
    let mut mock_repo = MockTransactionRepositoryImpl::new();
    let mut mock_gateway = MockWalletGatewayImpl::new();

    let source_wallet_uuid = Uuid::new_v4();
    let dest_wallet_uuid = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let amount = Decimal::from(100);

    mock_repo
        .expect_find_by_correlation_id()
        .returning(|_| Ok(None));

    mock_repo.expect_save().returning(|tx| Ok(tx));

    mock_gateway
        .expect_process_movement()
        .times(1)
        .returning(|_| Ok(false));

    mock_repo
        .expect_update()
        .withf(|tx: &Transaction| tx.status() == TransactionStatus::FAILED)
        .times(1)
        .returning(|tx| Ok(tx));

    let process_transaction_uc =
        ProcessTransactionUseCase::new(Arc::new(mock_repo), Arc::new(mock_gateway));
    let get_tx_uc =
        GetTransactionDetailsUseCase::new(Arc::new(MockTransactionRepositoryImpl::new()));
    let get_history_uc =
        GetWalletHistoryUseCase::new(Arc::new(MockTransactionRepositoryImpl::new()));

    let state = Arc::new(AppState {
        process_transaction_use_case: process_transaction_uc,
        get_transaction_details_use_case: get_tx_uc,
        get_wallet_history_use_case: get_history_uc,
    });

    let payload = CreateTransactionRequest {
        source_wallet_id: Some(source_wallet_uuid),
        dest_wallet_id: dest_wallet_uuid,
        amount,
        correlation_id,
    };

    // Act
    let result = initiate_transaction(State(state), Json(payload)).await;

    // Assert
    assert!(
        result.is_err(),
        "La petición debe lanzar un ApiError (500 o Gateway Error)"
    );
}
