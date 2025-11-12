-- Public keys registered for claim submission
CREATE TABLE miner_keys (
    miner_id UUID PRIMARY KEY,
    public_key_hex TEXT NOT NULL UNIQUE,
    stake_amount DECIMAL(18,8) NOT NULL DEFAULT 0,
    registration_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    revocation_timestamp TIMESTAMPTZ
);
CREATE INDEX idx_active_keys ON miner_keys (is_active);

-- The immutable ledger of verified claims
CREATE TABLE ledger (
    id BIGSERIAL PRIMARY KEY,
    submission_id UUID NOT NULL UNIQUE,
    miner_id UUID NOT NULL REFERENCES miner_keys(miner_id),
    task_id TEXT NOT NULL,
    claimed_score DECIMAL(10,8) NOT NULL,
    verified_score DECIMAL(10,8) NOT NULL,
    artifact_hash TEXT NOT NULL,
    artifact_uri TEXT NOT NULL,
    signature_hex TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    verification_duration_ms INTEGER NOT NULL,
    nonce TEXT NOT NULL UNIQUE
);

-- Task definitions (admin-managed)
CREATE TABLE tasks (
    task_id TEXT PRIMARY KEY,
    performance_threshold DECIMAL(10,8) NOT NULL,
    dataset_hash TEXT NOT NULL,
    optuna_storage_url TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
