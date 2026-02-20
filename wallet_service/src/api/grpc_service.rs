use crate::api::proto::wallet::wallet_service_server::WalletService;
use crate::api::proto::wallet::{
    ConfirmBalanceUpdateRequest, ConfirmBalanceUpdateResponse, ValidateAndReserveRequest,
    ValidateAndReserveResponse,
};
use crate::use_cases::process_movement::ProcessMovementUseCase;
use core::str::FromStr;
use rust_decimal::Decimal;
use tonic::{Request, Response, Status};
use uuid::Uuid;

// Definicion del servicio gRPC para comunicacion inter-servicios
// En Clean Architecture, los handlers gRPC son consideradores "Controladores"
// que delegan la lógica de negocio a los Casos de Uso.
pub struct WalletGrpcService {
    process_movement_use_case: ProcessMovementUseCase,
}

impl WalletGrpcService {
    pub fn new(process_movement_use_case: ProcessMovementUseCase) -> Self {
        Self {
            process_movement_use_case,
        }
    }
}

// Implementamos el trait autogenerado por Tonic para nuestro servicio
#[tonic::async_trait]
impl WalletService for WalletGrpcService {
    #[tracing::instrument(name = "WalletGrpcService::validate_and_reserve", skip(self))]
    async fn validate_and_reserve(
        &self,
        request: Request<ValidateAndReserveRequest>,
    ) -> Result<Response<ValidateAndReserveResponse>, Status> {
        let req = request.into_inner();

        tracing::info!("Recibida petición gRPC para reservar saldo: {:?}", req);

        // 1. Validar y parsear los datos de entrada
        let wallet_id = Uuid::parse_str(&req.wallet_id)
            .map_err(|_| Status::invalid_argument("El wallet_id no es un UUID válido"))?;

        let amount = Decimal::from_str(&req.amount)
            .map_err(|_| Status::invalid_argument("El monto no es un decimal válido"))?;

        // 2. Ejecutar el caso de uso (en un sistema real de reserva, usaríamos un caso de uso específico
        // que bloquee el dinero, pero aquí usamos process_movement_use_case a modo de demostración)
        // Convertimos el monto de reserva a negativo (simulando que el dinero se descuenta)
        let amount_to_reserve = -amount;

        match self
            .process_movement_use_case
            .execute(crate::domain::types::WalletId(wallet_id), amount_to_reserve)
            .await
        {
            Ok(_) => {
                let response = ValidateAndReserveResponse {
                    success: true,
                    message: "Saldo reservado exitosamente.".to_string(),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                // Mapear el error de dominio a un error de gRPC
                tracing::error!("Error al procesar reserva: {:?}", e);
                // Si usamos thiserror, podríamos hacer un mapping más detallado (ej: Status::not_found)
                let response = ValidateAndReserveResponse {
                    success: false,
                    message: format!("Fondos insuficientes o error: {}", e),
                };
                Ok(Response::new(response))
            }
        }
    }

    #[tracing::instrument(name = "WalletGrpcService::confirm_balance_update", skip(self))]
    async fn confirm_balance_update(
        &self,
        request: Request<ConfirmBalanceUpdateRequest>,
    ) -> Result<Response<ConfirmBalanceUpdateResponse>, Status> {
        let req = request.into_inner();

        tracing::info!(
            "Recibida petición gRPC para confirmar actualización: {:?}",
            req
        );

        // Lógica de compensación/confirmación
        // Aquí interactuaríamos con otro caso de uso para dar por exitosa la transacción
        // o deshacer la reserva en base a `req.is_success`.
        let message = if req.is_success {
            "Transacción confirmada definitivamente."
        } else {
            "Transacción fallida. Reserva deshecha (implementación parcial)."
        };

        let response = ConfirmBalanceUpdateResponse {
            success: true,
            message: message.to_string(),
        };
        Ok(Response::new(response))
    }
}
