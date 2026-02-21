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

        // Determinar la billetera afectada y el monto enviado al servicio dependendo del tipo de transacción
        // Para DEPOSIT, enviamos a dest_wallet, monto POSITIVO
        // Para WITHDRAWAL, enviamos a source_wallet, monto NEGATIVO
        // Para TRANSFER, idealmente se debe hacer dos llamadas gRPC, pero por ahora solo debitaremos
        // al source como una 'reserva' inicial.
        let wallet_id = match transaction.transaction_type() {
            TransactionType::DEPOSIT => transaction.destination_wallet_id().to_string(),
            TransactionType::WITHDRAWAL | TransactionType::TRANSFER => {
                transaction.source_wallet_id().unwrap().to_string()
            }
        };

        // Si es retiro o cobro por transferencia, pasamos el monto como negativo
        let amount_str = match transaction.transaction_type() {
            TransactionType::DEPOSIT => transaction.amount().to_string(),
            TransactionType::WITHDRAWAL | TransactionType::TRANSFER => {
                format!("-{}", transaction.amount())
            }
        };

        let request = tonic::Request::new(ValidateAndReserveRequest {
            wallet_id,
            amount: amount_str,
            transaction_id: transaction.id().to_string(),
        });

        match client.validate_and_reserve(request).await {
            Ok(response) => {
                let inner = response.into_inner();
                if inner.success {
                    info!("Movimiento validado y procesado exitosamente por Wallet Service");
                    Ok(true)
                } else {
                    info!("Wallet Service rechazó el movimiento: {}", inner.message);
                    Ok(false)
                }
            }
            Err(e) => {
                error!("Error gRPC al validar y reservar: {}", e);
                Err(TransactionError::GatewayError(e.to_string()))
            }
        }
    }
}
