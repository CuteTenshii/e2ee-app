-- Your SQL goes here
CREATE TABLE users (
    id UUID PRIMARY KEY UNIQUE NOT NULL DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    phone_number VARCHAR(100) UNIQUE NOT NULL,
    avatar_hash VARCHAR(64) UNIQUE,
    last_seen TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
