use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberUsername(String);

impl SubscriberUsername {
    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constraints on subscriber names. Propagates String
    /// Error to caller.
    pub fn parse(s: String) -> Result<Self, String> {
        // `.trim()` returns a view over the input `s` without trailing
        // whitespace like characters.
        // `.is_empty` checks if the view contains any character.
        let is_empty_or_whitespace = s.trim().is_empty();

        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character: `å` is a single grapheme, but it is composed of two
        // characters (`a`and `̊``).
        //
        // `graphemes` return an iterator over the graphemes in the input `s`
        // `true` specifies that we want to use the extended grapheme definition
        // set, the recommended one.
        let is_too_long = s.graphemes(true).count() > 256;

        // Iterate over all characters in the input `s` to check if any of them
        // matches one of the characters in the forbidden array.
        let forbidden_chars = ['/', '\\', '{', '}', '(', ')', '<', '>', '"'];
        let contains_forbidden_chars = s.chars().any(|g| forbidden_chars.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_chars {
            Err(format!("{} is not a valid subscriber username", s))
        } else {
            Ok(Self(s))
        }
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
        assert_ok!(SubscriberUsername::parse(username));
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
