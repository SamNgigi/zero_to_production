-- Add migration script here
CREATE TABLE subscriptions (
  id UUID DEFAULT uuidv7() NOT NULL UNIQUE,
  PRIMARY KEY (id),
  email TEXT NOT NULL UNIQUE,
  username TEXT NOT NULL,
  subscribed_at timestamptz NOT NULL
);
