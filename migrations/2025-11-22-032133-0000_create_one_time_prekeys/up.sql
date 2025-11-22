-- Your SQL goes here
CREATE TABLE one_time_prekeys (
    id BIGSERIAL PRIMARY KEY,
    device_id UUID REFERENCES devices(id) ON DELETE CASCADE,
    prekey_pub BYTEA NOT NULL,
    is_consumed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX ON one_time_prekeys (device_id, is_consumed);