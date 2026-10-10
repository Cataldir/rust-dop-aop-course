-- Learner-local database only. The runtime never automatically migrates i.
CREATE TABLE metric_point (
    tenant text NOT NULL, metric text NOT NULL, timestamp_ms bigint NOT NULL CHECK(timestamp_ms>=0),
    value double precision NOT NULL CHECK(value > '-Infinity'::float8 AND value < 'Infinity'::float8),
    tags jsonb NOT NULL,
    PRIMARY KEY(tenant,metric,timestamp_ms)
);
CREATE TABLE course_order (
    tenant text NOT NULL, order_id text NOT NULL, units integer NOT NULL CHECK(units BETWEEN 1 AND 10000),
    status text NOT NULL CHECK(status IN ('accepted','fulfilled')), PRIMARY KEY(tenant,order_id)
);
CREATE TABLE outbox (
    tenant text NOT NULL,event_id text NOT NULL,payload jsonb NOT NULL,trace_headers jsonb NOT NULL,sent boolean NOT NULL DEFAULT false,
    PRIMARY KEY(tenant,event_id)
);
CREATE TABLE processed_event (
    tenant text NOT NULL,event_id text NOT NULL,payload_hash text NOT NULL,PRIMARY KEY(tenant,event_id)
);
CREATE TABLE dead_letter (
    tenant text NOT NULL,event_id text NOT NULL,reason text NOT NULL,payload bytea NOT NULL,
    PRIMARY KEY(tenant,event_id)
);
