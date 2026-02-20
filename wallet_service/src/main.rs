use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use wallet_service::{
    api::{
        grpc_service::WalletGrpcService,
        http_routes::{routes, AppState},
        proto::wallet::wallet_service_server::WalletServiceServer,
    },
    infrastructure::persistence::{
        user_repository::PostgresUserRepository, wallet_repository::PostgresWalletRepository,
    },
    use_cases::{
        create_user::CreateUserUseCase, create_wallet::CreateWalletUseCase,
        get_user_wallets::GetWalletsUseCase, get_wallet::GetWalletUseCase,
        process_movement::ProcessMovementUseCase,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        wallet_service::api::http_routes::create_user,
        wallet_service::api::http_routes::create_wallet,
        wallet_service::api::http_routes::list_user_wallets,
        wallet_service::api::http_routes::get_wallet_details
    ),
    components(schemas(
        wallet_service::api::http_routes::CreateUserRequest,
        wallet_service::api::http_routes::CreateWalletRequest,
        wallet_service::api::response::ApiResponse<serde_json::Value>
    ))
)]
struct ApiDoc;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Cargar variables de entorno
    dotenv().ok();

    // 2. Configurar Logging/Tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting Wallet Service...");

    // 3. Configurar Conexión a Base de Datos
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    info!("Connected to Database");

    // 4. Instanciar Dependencias (Infraestructura)
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let wallet_repo = Arc::new(PostgresWalletRepository::new(pool.clone()));

    // 5. Instanciar Casos de Uso
    let create_user_use_case = CreateUserUseCase::new(user_repo.clone());
    let create_wallet_use_case = CreateWalletUseCase::new(wallet_repo.clone(), user_repo.clone());
    let list_user_wallets_use_case = GetWalletsUseCase::new(wallet_repo.clone());
    let get_wallet_details_use_case = GetWalletUseCase::new(wallet_repo.clone());
    let process_movement_use_case = ProcessMovementUseCase::new(wallet_repo.clone());

    // 6. Configurar Servidor gRPC
    let grpc_host = env::var("GRPC_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let grpc_port = env::var("GRPC_PORT").unwrap_or_else(|_| "50051".to_string());
    let grpc_addr = format!("{}:{}", grpc_host, grpc_port).parse()?;

    let grpc_service = WalletGrpcService::new(process_movement_use_case);

    info!("gRPC Server listening on {}", grpc_addr);

    // Ejecutar servidor gRPC en un hilo / tarea separada
    tokio::spawn(async move {
        if let Err(e) = tonic::transport::Server::builder()
            .add_service(WalletServiceServer::new(grpc_service))
            .serve(grpc_addr)
            .await
        {
            tracing::error!("gRPC server error: {}", e);
        }
    });

    // 7. Configurar Estado de la App Axum
    let app_state = Arc::new(AppState {
        create_user_use_case,
        create_wallet_use_case,
        list_user_wallets_use_case,
        get_wallet_details_use_case,
    });

    // 8. Configurar Rutas y Servidor HTTP
    let app = routes(app_state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);

    info!("HTTP Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
