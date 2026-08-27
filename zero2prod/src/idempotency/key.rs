#[derive(Debug)]
pub struct IdempotencyKey(String);

impl TryFrom<String> for IdempotencyKey {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.is_empty() {
            anyhow::bail!("`idempotency_key` cannot be empty.")
        };
        let max_len = 50;
        if s.len() >= max_len {
            anyhow::bail!("`idempotency_key` should be less than {max_len} characters.")
        };
        Ok(Self(s))
    }
}

impl From<IdempotencyKey> for String {
    fn from(key: IdempotencyKey) -> String {
        key.0
    }
}

impl AsRef<str> for IdempotencyKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
