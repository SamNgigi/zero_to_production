-- Add migration script here
CREATE TABLE subscription_tokens(
  subscription_token TEXT NOT NULL UNIQUE,
  PRIMARY KEY (subscription_token),
  subscription_id UUID NOT NULL
    REFERENCES subscriptions (id)
)
