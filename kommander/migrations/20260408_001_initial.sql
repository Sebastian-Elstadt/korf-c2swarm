CREATE TABLE nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nodus_id BYTEA NOT NULL UNIQUE CHECK (octet_length(nodus_id) = 32),
    mac_addr TEXT NOT NULL,
    asym_sec_algo SMALLINT NOT NULL,
    asym_sec_pubkey BYTEA NOT NULL,
    cpu_arch TEXT NOT NULL,
    hostname TEXT,
    username TEXT,
    device_name TEXT,
    account_name TEXT,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);