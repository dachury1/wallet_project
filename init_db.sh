#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "postgres" <<-EOS
    CREATE DATABASE wallet_db;
    CREATE DATABASE transaction_db;
EOS
