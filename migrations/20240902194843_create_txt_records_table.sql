CREATE TABLE IF NOT EXISTS txt_records (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    domain VARCHAR(255) NOT NULL,
    record TEXT NOT NULL,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE
);
