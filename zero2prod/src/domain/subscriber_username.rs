use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberUsername(String);

impl SubscriberUsername {
    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constraints on subscriber names.
    pub fn parse(s: String) -> Result<Self, String> {
        // `.trim()` returns a view over the input `s` without trailing
        // whitespace like characters.
        // `.is_empty` checks if the view contains any character.
        let is_empty = s.trim().is_empty();

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
        let forbidden_chars = vec!['/', '(', ')', '<', '>', '{', '}', '"', '\\'];
        let contains_forbidden_chars = s.chars().any(|g| forbidden_chars.contains(&g));

        if is_empty || is_too_long || contains_forbidden_chars {
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
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_username_is_valid() {
        let username = "ё".repeat(256);
        assert_ok!(SubscriberUsername::parse(username));
    }

    #[test]
    fn a_username_longer_than_256_graphemes_is_rejected() {
        let username = "a".repeat(257);
        assert_err!(SubscriberUsername::parse(username));
    }

    #[test]
    fn whitespace_only_username_string_is_rejected() {
        let username = " ".to_string();
        assert_err!(SubscriberUsername::parse(username));
    }

    #[test]
    fn empty_username_string_is_rejected() {
        let username = "".to_string();
        assert_err!(SubscriberUsername::parse(username));
    }

    #[test]
    fn username_containing_invalid_chars_are_rejected() {
        for username in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let username = username.to_string();
            assert_err!(SubscriberUsername::parse(username));
        }
    }

    #[test]
    fn a_valid_username_is_parsed_successfully() {
        let username = "lei yin loo".to_string();
        assert_ok!(SubscriberUsername::parse(username));
    }
}
