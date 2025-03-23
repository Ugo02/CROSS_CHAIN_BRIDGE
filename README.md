# Cross-Chain Bridge Project (Initial Phase)

This project is the **initial phase** of a cross-chain bridge implementation, designed to facilitate the transfer of tokens between two Ethereum testnets: **Holesky** and **Sepolia**. Currently, the system listens for `Deposit` and `Distribution` events on both chains and stores them in a PostgreSQL database. However, the **automation between depositing and distributing tokens is not yet implemented**. This means that when a user deposits tokens on one chain, it does not automatically trigger the distribution on the other chain. This functionality will be added in a future phase.

---

## Quick Start

### 1. Set Up the Database

1. **Install PostgreSQL**:
   - Install PostgreSQL from [postgresql.org](https://www.postgresql.org/download/).

2. **Create a Database**:
   - Create a new database for the project:

     ```sql
     CREATE DATABASE cross_chain_bridge;
     ```

3. **Set the Database URL**:
   - Rename `.env.example` to `.env` and update it with your PostgreSQL credentials:

     ```plaintext
     DATABASE_URL=postgres://username:password@localhost/cross_chain_bridge
     ```

4. **Run Migrations**:
   - Set up the database schema:

     ```bash
     sqlx migrate run
     ```

---

### 2. Run the Indexer

1. **Start the Indexer**:
   - Navigate to the `bridge_indexer` folder and run:

     ```bash
     cargo run
     ```

   This will start listening for `Deposit` and `Distribution` events on both chains.

---

### 3. Use the Contracts

1. **Deposit Tokens**:
   - Call the `deposit` function on the bridge contract for the source chain (addresses are in `.env.example`).

2. **Distribute Tokens**:
   - Call the `distribute` function on the bridge contract for the destination chain (addresses are in `.env.example`).

---
