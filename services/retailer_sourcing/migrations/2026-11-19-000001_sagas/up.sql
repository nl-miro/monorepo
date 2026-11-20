CREATE TABLE sagas (
    id UUID PRIMARY KEY CHECK (COALESCE(uuid_extract_version(id), 0) = 7),
    saga_type TEXT NOT NULL,
    correlation_id UUID NOT NULL CHECK (COALESCE(uuid_extract_version(correlation_id), 0) = 7),
    retailer_code TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('in_progress', 'succeeded', 'failed')),
    payload JSONB NOT NULL CHECK (jsonb_typeof(payload) = 'object'),
    UNIQUE (saga_type, correlation_id)
);
CREATE INDEX sagas_status_idx ON sagas (status);
