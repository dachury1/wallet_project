FROM rust:1-slim-bookworm as builder

WORKDIR /app

# Install system dependencies (needed for compilation)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the entire workspace to build context (assuming root context)
COPY . .

# Build the specific package in release mode
RUN cargo build --release --package wallet_service

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/wallet_service /usr/local/bin/wallet_service

ENV RUST_LOG=info
CMD ["wallet_service"]
