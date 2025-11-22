-- Your SQL goes here
CREATE TABLE verification_codes (
    phone_number TEXT NOT NULL,
    code_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    attempt_count INT DEFAULT 0,

    PRIMARY KEY(phone_number)
);
