# Wallet Project - Transaction Clearing System

This project is a high-performance, concurrent transaction clearing system built with Rust. It features a microservices architecture designed to handle real-time balances and transaction processing with a focus on safety, consistency, and scalability.

## 🚀 Project Overview

The system is composed of two main microservices:

1.  **Wallet Service**: The source of truth for user balances and account management. It handles deposit, withdrawal, and balance inquiry operations.
2.  **Transaction Service**: Responsible for recording transaction history, orchestration, and ensuring atomic operations across the system.

Both services are built using **Clean Architecture** principles to ensure separation of concerns, testability, and maintainability.

## 🏗️ Architecture & Tech Stack

The project leverages a modern Rust ecosystem:

-   **Language**: Rust (Edition 2024 for services, 2021 for libraries)
-   **Web Framework**: [Axum](https://github.com/tokio-rs/axum) (high-performance, ergonomic)
-   **Async Runtime**: [Tokio](https://tokio.rs/)
-   **Database**: PostgreSQL with [SQLx](https://github.com/launchbadge/sqlx) for compile-time verified queries.
-   **gRPC**: [Tonic](https://github.com/hyperium/tonic) for inter-service communication.
-   **Serialization**: [Serde](https://serde.rs/)
-   **Logging/Tracing**: `tracing` & `tracing-subscriber` setup.
-   **Error Handling**: `thiserror` (for libraries) and `anyhow` (for applications).
-   **Containerization**: Docker & Docker Compose.

### Directory Structure

```
wallet_project/
├── wallet_service/       # Manages user balances (Domain, Use Cases, Infrastructure)
├── transaction_service/  # Manages transaction logs (Domain, Use Cases, Infrastructure)
├── common/               # Shared libraries, types, and utilities
├── docker-compose.yml    # Orchestration for services and database
└── Cargo.toml            # Workspace configuration
```

## 🛠️ Prerequisites

-   [Docker](https://www.docker.com/) & Docker Compose
-   [Rust](https://www.rust-lang.org/tools/install) (latest stable) - *Optional if running via Docker*
-   `sqlx-cli` (for database migrations) - `cargo install sqlx-cli`

## 🚀 Getting Started

### Option 1: Run with Docker Compose (Recommended)

The easiest way to stand up the entire system (Database + Services) is using Docker Compose.

1.  **Clone the repository**:
    ```bash
    git clone <repository_url>
    cd wallet_project
    ```

2.  **Start the services**:
    ```bash
    docker-compose up -d --build
    ```

    This will start:
    -   **PostgreSQL** on port `5432`
    -   **Wallet Service** on port `3001` (internal: 3000)
    -   **Transaction Service** on port `3002` (internal: 3000)

3.  **Check logs**:
    ```bash
    docker-compose logs -f
    ```

4.  **Stop the services**:
    ```bash
    docker-compose down
    ```

### Option 2: Run Locally (Development)

1.  **Start the Database**:
    You can use the docker-compose file to just start the database if you want to run the services manually.
    ```bash
    docker-compose up -d postgres
    ```

2.  **Configure Environment**:
    Each service expects a `DATABASE_URL` environment variable. You can create a `.env` file in the root or export it in your shell.
    ```bash
    export DATABASE_URL="postgres://admin:password@localhost:5432/wallet_db"
    ```

3.  **Run Migrations** (Ensure you have `sqlx-cli` installed):
    ```bash
    # Navigate to each service and run migrations if present
    cd wallet_service
    sqlx database create
    sqlx migrate run
    ```

4.  **Run Services**:
    Open two terminal windows/tabs:

    *Terminal 1 (Wallet Service)*:
    ```bash
    cd wallet_service
    cargo run
    ```

    *Terminal 2 (Transaction Service)*:
    ```bash
    cd transaction_service
    cargo run
    ```

## ⚙️ Configuration

The services are configured primarily via environment variables:

| Variable | Description | Default (Docker) |
| :--- | :--- | :--- |
| `DATABASE_URL` | Connection string for PostgreSQL | `postgres://admin:password@postgres:5432/wallet_db` |
| `RUST_LOG` | Log level (e.g., `info`, `debug`) | `info` |
| `PORT` | HTTP Port for the service | `3000` (mapped to 3001/3002) |

## 🧪 Testing

To run the test suite for the entire workspace:

```bash
cargo test --workspace
```

For a specific service:

```bash
cargo test -p wallet_service
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
