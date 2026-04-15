#[derive(Debug)]
pub struct SubscriberUsername(String);

impl SubscriberUsername {
    pub fn parse(s: String) -> Result<Self, String> {
        // TODO: NOT YET IMPLEMENTED
        todo!("PARSING USERNAME '{}' NOT YET IMPLEMENTED", s);
    }
}

impl AsRef<str> for SubscriberUsername {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {

    use crate::domain::SubscriberUsername;
    use claim::{assert_err, assert_ok};

    #[test]
    fn test_valid_username_is_parsed_successfully() {
        let username = "ley yin loo".to_string();
        assert_ok!(SubscriberUsername::parse(username));
    }

    #[test]
    fn test_256_grapheme_long_username_is_valid() {
        let username = "a̐".repeat(256);
        assert_err!(SubscriberUsername::parse(username));
    }

    #[test]
    fn test_too_long_username_is_rejected() {
        let username = "e".repeat(257);
        assert_err!(SubscriberUsername::parse(username));
    }

    #[test]
    fn test_empty_username_is_rejected() {
        let username = "".to_string();
        assert_err!(SubscriberUsername::parse(username));
    }

    #[test]
    fn test_only_whitespace_username_is_rejected() {
        let username = "  ".to_string();
        assert_err!(SubscriberUsername::parse(username));
    }

    #[test]
    fn test_username_with_forbidden_chars_is_rejected() {
        for username in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let username = username.to_string();
            assert_err!(SubscriberUsername::parse(username));
        }
    }
}
