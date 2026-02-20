use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use transaction_service::{
    api::http_routes::{routes, AppState},
    infrastructure::{
        gateways::fake_wallet_gateway::FakeWalletGateway,
        persistence::transaction_repository::PostgresTransactionRepository,
    },
    use_cases::{
        get_transaction_details::GetTransactionDetailsUseCase,
        get_wallet_history::GetWalletHistoryUseCase,
        process_transaction::ProcessTransactionUseCase,
    },
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        transaction_service::api::http_routes::initiate_transaction,
        transaction_service::api::http_routes::get_transaction_details,
        transaction_service::api::http_routes::get_wallet_history
    ),
    components(schemas(
        transaction_service::api::http_routes::CreateTransactionRequest,
        transaction_service::api::response::ApiResponse<serde_json::Value>
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

    info!("Starting Transaction Service...");

    // 3. Configurar Conexión a Base de Datos
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    info!("Connected to Database");

    // 4. Instanciar Dependencias (Infraestructura)
    let transaction_repo = Arc::new(PostgresTransactionRepository::new(pool));
    // TODO: Reemplazar FakeWalletGateway con implementación real gRPC cuando esté lista
    let wallet_gateway = Arc::new(FakeWalletGateway::new());

    // 5. Instanciar Casos de Uso
    let process_transaction_use_case =
        ProcessTransactionUseCase::new(transaction_repo.clone(), wallet_gateway.clone());
    let get_transaction_details_use_case =
        GetTransactionDetailsUseCase::new(transaction_repo.clone());
    let get_wallet_history_use_case = GetWalletHistoryUseCase::new(transaction_repo.clone());

    // 6. Configurar Estado de la App Axum
    let app_state = Arc::new(AppState {
        process_transaction_use_case,
        get_transaction_details_use_case,
        get_wallet_history_use_case,
    });

    // 7. Configurar Rutas y Servidor
    let app = routes(app_state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);

    // 8. Iniciar Background Jobs (Procesos en Segundo Plano)
    let job_repo = transaction_repo.clone();
    let job_gateway = wallet_gateway.clone();

    tokio::spawn(async move {
        // Intervalo de ejecución: cada 60 segundos
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        let job =
            transaction_service::jobs::retry::RetryFailedTransactionJob::new(job_repo, job_gateway);

        info!("Background Job Scheduler started");

        loop {
            interval.tick().await;
            job.run().await;
        }
    });

    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
