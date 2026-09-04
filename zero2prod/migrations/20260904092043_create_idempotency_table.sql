-- Add migration script here
CREATE TYPE header_pair AS (
  name TEXT,
  value BYTEA
);

CREATE TABLE idempotency (
  idempotency_key UUID NOT NULL,
  user_id UUID NOT NULL REFERENCES users(user_id),
  response_status_code SMALLINT,
  response_headers header_pair[],
  response_body BYTEA,
  created_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (idempotency_key, user_id)
);
