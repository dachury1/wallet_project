use crate::api::proto::wallet::wallet_service_client::WalletServiceClient;
use crate::api::proto::wallet::ValidateAndReserveRequest;
use crate::domain::{
    entities::{Transaction, TransactionType},
    error::TransactionError,
    gateways::WalletGateway,
};
use async_trait::async_trait;
use tracing::{error, info};

pub struct GrpcWalletGateway {
    wallet_url: String,
}

impl GrpcWalletGateway {
    pub fn new(wallet_url: String) -> Self {
        Self { wallet_url }
    }
}

#[async_trait]
impl WalletGateway for GrpcWalletGateway {
    async fn process_movement(&self, transaction: &Transaction) -> Result<bool, TransactionError> {
        info!("Conectando al Wallet Service en {}", self.wallet_url);

        let mut client = WalletServiceClient::connect(self.wallet_url.clone())
            .await
            .map_err(|e| {
                TransactionError::GatewayError(format!(
                    "Fallo de conexión al Wallet Service: {}",
                    e
                ))
            })?;

        // Determinar los movimientos a realizar dependiendo del tipo de transacción
        let mut movements: Vec<(String, String)> = Vec::new();

        match transaction.transaction_type() {
            TransactionType::DEPOSIT => {
                movements.push((
                    transaction.destination_wallet_id().to_string(),
                    transaction.amount().to_string(),
                ));
            }
            TransactionType::WITHDRAWAL => {
                movements.push((
                    transaction.source_wallet_id().unwrap().to_string(),
                    format!("-{}", transaction.amount()),
                ));
            }
            TransactionType::TRANSFER => {
                // Para TRANSFER, realizamos dos llamadas gRPC:
                // 1. Débito a la billetera origen (reserva/retiro)
                movements.push((
                    transaction.source_wallet_id().unwrap().to_string(),
                    format!("-{}", transaction.amount()),
                ));
                // 2. Crédito a la billetera destino (depósito)
                movements.push((
                    transaction.destination_wallet_id().to_string(),
                    transaction.amount().to_string(),
                ));
            }
        };

        let mut executed_movements: Vec<(String, String)> = Vec::new();

        for (wallet_id, amount_str) in movements {
            let request = tonic::Request::new(ValidateAndReserveRequest {
                wallet_id: wallet_id.clone(),
                amount: amount_str.clone(),
                transaction_id: transaction.id().to_string(),
            });

            match client.validate_and_reserve(request).await {
                Ok(response) => {
                    let inner = response.into_inner();
                    if inner.success {
                        info!("Movimiento validado y procesado por Wallet Service (wallet: {}, amount: {})", wallet_id, amount_str);
                        executed_movements.push((wallet_id, amount_str));
                    } else {
                        info!(
                            "Wallet Service rechazó el movimiento (wallet: {}, amount: {}): {}",
                            wallet_id, amount_str, inner.message
                        );
                        Self::compensate_movements(
                            &mut client,
                            &executed_movements,
                            transaction.id().to_string(),
                        )
                        .await;
                        return Ok(false);
                    }
                }
                Err(e) => {
                    error!(
                        "Error gRPC al validar y reservar (wallet: {}, amount: {}): {}",
                        wallet_id, amount_str, e
                    );
                    Self::compensate_movements(
                        &mut client,
                        &executed_movements,
                        transaction.id().to_string(),
                    )
                    .await;
                    return Err(TransactionError::GatewayError(e.to_string()));
                }
            }
        }

        info!("Todos los movimientos de la transacción procesados exitosamente");
        Ok(true)
    }
}

impl GrpcWalletGateway {
    /// Compensación simple (Saga) para deshacer movimientos ejecutados en caso de fallo
    async fn compensate_movements(
        client: &mut WalletServiceClient<tonic::transport::Channel>,
        executed_movements: &[(String, String)],
        transaction_id: String,
    ) {
        for (wallet_id, amount_str) in executed_movements.iter().rev() {
            // Invertir el monto
            let reversed_amount = if amount_str.starts_with('-') {
                amount_str[1..].to_string()
            } else {
                format!("-{}", amount_str)
            };

            info!(
                "Ejecutando compensación para wallet: {} por monto: {}",
                wallet_id, reversed_amount
            );

            let req = tonic::Request::new(ValidateAndReserveRequest {
                wallet_id: wallet_id.clone(),
                amount: reversed_amount,
                transaction_id: format!("{}-rollback", transaction_id),
            });

            if let Err(e) = client.validate_and_reserve(req).await {
                error!(
                    "Error crítico: Falló la compensación para la wallet {}: {}",
                    wallet_id, e
                );
            }
        }
    }
}
