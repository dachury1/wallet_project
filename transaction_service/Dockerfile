FROM rust:1-slim-bookworm as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the entire workspace
COPY . .

# Build the specific package
RUN cargo build --release --package transaction_service

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/transaction_service /usr/local/bin/transaction_service

ENV RUST_LOG=info
CMD ["transaction_service"]
