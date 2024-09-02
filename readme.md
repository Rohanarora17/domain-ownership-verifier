# DNS TXT Record Generation and Verification Service

A Rust-based web service for generating and verifying DNS TXT records. This service allows users to generate TXT records for domain ownership verification and subsequently verify them.

## Features

- DNS TXT record generation
- DNS TXT record verification
- Rate limiting
- PostgreSQL database integration

## Prerequisites

- Rust (latest stable version)
- PostgreSQL

## Setup

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/dns-record-verification.git
   cd dns-record-verification
   ```

2. Set up the database:
   ```
   psql -c "CREATE DATABASE dns_verification"
   sqlx database create
   sqlx migrate run
   ```

3. Configure environment variables:
   ```
   cp .env.example .env
   # Edit .env with your database credentials 
   ```

## Running the Service

1. Build and run the project:
   ```
   cargo run
   ```

2. The service will be available at `http://127.0.0.1:8080`

## API Endpoints

- `POST /generate_txt_record`: Generate a TXT record for a domain
- `POST /verify_txt_record`: Verify a TXT record for a domain
- `GET /domain_status`: Check the status of a domain's ownership verification



