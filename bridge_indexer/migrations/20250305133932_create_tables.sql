-- Add migration script here

CREATE TABLE deposits (
    id SERIAL PRIMARY KEY,
    token TEXT NOT NULL,
    sender TEXT NOT NULL,
    recipient TEXT NOT NULL,
    amount BIGINT NOT NULL,
    nonce BIGINT UNIQUE NOT NULL,
    timestamp TIMESTAMP DEFAULT NOW()
);

CREATE TABLE distributions (
    id SERIAL PRIMARY KEY,
    token TEXT NOT NULL,
    recipient TEXT NOT NULL,
    amount BIGINT NOT NULL,
    nonce BIGINT UNIQUE NOT NULL,
    timestamp TIMESTAMP DEFAULT NOW()
);

CREATE TABLE processed_transactions (
    nonce BIGINT PRIMARY KEY
);
