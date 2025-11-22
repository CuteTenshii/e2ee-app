-- Your SQL goes here
CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    last_seen TIMESTAMPTZ,
    is_revoked BOOLEAN DEFAULT FALSE,

    -- Public keys
    identity_key_pub BYTEA NOT NULL,
    signed_prekey_pub BYTEA NOT NULL,
    signed_prekey_signature BYTEA NOT NULL,

    -- FCM/APNs push notifications token
    push_token TEXT,

    UNIQUE(user_id, id)
);
