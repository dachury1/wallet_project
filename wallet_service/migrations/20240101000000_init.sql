-- Migracion inicial para Wallet Service

-- Habilitar extension UUID
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Tabla de Usuarios
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Tabla de Billeteras
CREATE TABLE IF NOT EXISTS wallets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    label VARCHAR(100) NOT NULL,
    balance DECIMAL(20, 2) NOT NULL DEFAULT 0.00, -- Precisión fija
    currency VARCHAR(3) NOT NULL, -- ISO Code e.g. COP
    version INTEGER NOT NULL DEFAULT 1, -- Optimistic Locking
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES users(id)
);

-- Indices recomendados para busquedas frecuentes
CREATE INDEX idx_wallets_user_id ON wallets(user_id);
