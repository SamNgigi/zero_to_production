-- Add migration script here
ALTER TABLE subscriptions RENAME COLUMN name TO username;
