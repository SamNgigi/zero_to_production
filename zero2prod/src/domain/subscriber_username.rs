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

    #[test]
    fn test_valid_username_is_parsed_successfully() {}

    #[test]
    fn test_256_grapheme_long_username_is_valid() {}

    #[test]
    fn test_too_long_username_is_rejected() {}

    #[test]
    fn test_empty_username_is_rejected() {}

    #[test]
    fn test_only_whitespace_username_is_rejected() {}

    #[test]
    fn test_username_with_forbidden_chars_is_rejected() {}
}
