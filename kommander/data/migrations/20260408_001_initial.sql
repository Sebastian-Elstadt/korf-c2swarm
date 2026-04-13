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
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    host_local_time TIMESTAMPTZ NULL
);

CREATE TABLE node_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    event_type SMALLINT NOT NULL,
    text_content TEXT NULL,
    ipv4_addr TEXT NULL,
    network_port INT NULL,
    network_protocol SMALLINT NULL
);

CREATE TABLE node_commands_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    "status" SMALLINT NOT NULL DEFAULT 0,
    command_type SMALLINT NOT NULL,
    last_attempted_at TIMESTAMPTZ NULL,
    completed_at TIMESTAMPTZ NULL,
    text_content TEXT NULL
);