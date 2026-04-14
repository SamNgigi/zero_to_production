use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub username: SubscriberUsername,
}

// Using the new-type pattern
#[derive(Debug)]
pub struct SubscriberUsername(String);

impl SubscriberUsername {
    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constraints on subscriber names.
    /// It panics otherwise
    pub fn parse(s: String) -> Result<Self, String> {
        // `.trim()` returns a view over the input `s` without trailing
        // whitespace like characters.
        // `.is_empty` checks if the view contains any character.
        let is_empty_or_white_space = s.trim().is_empty();

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
        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = s.chars().any(|g| forbidden_chars.contains(&g));

        if is_empty_or_white_space || is_too_long || contains_forbidden_chars {
            Err(format!("{} is not a valid subscriber name", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberUsername {
    /// Rust standard library that allows the caller to read
    /// the inner value without the power to mutate it.
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberUsername;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let username = "ё".repeat(256);
        assert_ok!(SubscriberUsername::parse(username));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let username = "a".repeat(257);
        assert_err!(SubscriberUsername::parse(username));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let username = " ".to_string();
        assert_err!(SubscriberUsername::parse(username));
    }

    #[test]
    fn empty_string_is_rejected() {
        let username = "".to_string();
        assert_err!(SubscriberUsername::parse(username));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for username in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let username = username.to_string();
            assert_err!(SubscriberUsername::parse(username));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let username = "lei yin loo".to_string();
        assert_ok!(SubscriberUsername::parse(username));
    }
}
