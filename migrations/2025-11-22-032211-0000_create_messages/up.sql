-- Your SQL goes here
CREATE TABLE messages (
    id BIGSERIAL PRIMARY KEY,
    sender_user_id UUID REFERENCES users(id),
    sender_device_id UUID REFERENCES devices(id),

    recipient_user_id UUID REFERENCES users(id),
    recipient_device_id UUID REFERENCES devices(id),

    ciphertext BYTEA NOT NULL,
    message_type SMALLINT NOT NULL, -- 0=initial,1=ratcheted,2=system...
    protocol_version SMALLINT NOT NULL DEFAULT 1,

    delivered_at TIMESTAMPTZ, -- NULL = pending
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX ON messages (recipient_device_id, delivered_at);
