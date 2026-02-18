-- Migracion inicial para Transaction Service

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enums para status y tipo
CREATE TYPE transaction_status AS ENUM ('PENDING', 'COMPLETED', 'FAILED', 'REVERSED');
CREATE TYPE transaction_type AS ENUM ('TRANSFER', 'DEPOSIT', 'WITHDRAWAL');

-- Tabla de Transacciones
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_wallet_id UUID, -- Nullable para recargas externas
    destination_wallet_id UUID NOT NULL,
    amount DECIMAL(20, 2) NOT NULL,
    status transaction_status NOT NULL DEFAULT 'PENDING',
    transaction_type transaction_type NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    correlation_id UUID -- Para rastreo externo
);

CREATE INDEX idx_transactions_wallets ON transactions(source_wallet_id, destination_wallet_id);
CREATE INDEX idx_transactions_correlation ON transactions(correlation_id);
