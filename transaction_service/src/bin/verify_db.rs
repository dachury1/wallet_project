use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::postgres::PgPoolOptions;
use transaction_service::domain::entities::{Transaction, TransactionStatus, TransactionType};
use transaction_service::domain::repository::TransactionRepository;
use transaction_service::infrastructure::persistence::transaction_repository::PostgresTransactionRepository;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env manually or just use hardcoded string for test
    let database_url = "postgres://admin:password@localhost:5432/transaction_db";

    println!("Connecting to database: {}", database_url);

    // Create pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    println!("✅ Database connection successful!");

    // Run migrations
    println!("Running migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Failed to run migrations: {}", e))?;
    println!("✅ Migrations applied successfully!");

    let repository = PostgresTransactionRepository::new(pool);

    // Create dummy transaction
    let new_transaction = Transaction {
        id: Uuid::new_v4(),                     // Random UUID
        source_wallet_id: Some(Uuid::new_v4()), // Random Source Wallet
        destination_wallet_id: Uuid::new_v4(),  // Random Dest Wallet
        amount: Decimal::new(10050, 2),         // 100.50
        status: TransactionStatus::PENDING,
        transaction_type: TransactionType::TRANSFER,
        created_at: Utc::now(),
        correlation_id: Some(Uuid::new_v4()),
    };

    println!("Attempting to save transaction: {:?}", new_transaction);

    match repository.save(new_transaction.clone()).await {
        Ok(saved) => {
            println!("✅ Transaction saved successfully!");
            println!("Saved ID: {}", saved.id);
            println!("Status: {:?}", saved.status);
            println!("Amount: {}", saved.amount);
            println!("Type: {:?}", saved.transaction_type);
        }
        Err(e) => {
            eprintln!("❌ Failed to save transaction: {:?}", e);
            eprintln!("Check if database migrations are applied (table 'transactions' exists?)");
            eprintln!("Check if ENUM types match Rust types (Enum vs Varchar).");
        }
    }

    Ok(())
}
