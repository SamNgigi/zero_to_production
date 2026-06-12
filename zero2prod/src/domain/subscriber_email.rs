use std::str::FromStr;
use validator::ValidateEmail;

#[derive(Debug, Clone)]
pub struct SubscriberEmail(String);

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl SubscriberEmail {
    pub fn parse(s: &str) -> Result<Self, String> {
        if s.validate_email() {
            Ok(Self(s.to_owned()))
        } else {
            Err(format!("{} is not a valid subscriber email", s))
        }
    }
}

impl FromStr for SubscriberEmail {
    /*
     * INFO: Allows us to use `serde_as` parsing with an attribute macro
     * in config
     */

    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use claim::assert_err;
    use fake::{
        Fake,
        faker::internet::en::SafeEmail,
        rand::{SeedableRng, rngs::StdRng},
    };

    #[test]
    fn test_empty_email_is_rejected() {
        assert_err!(SubscriberEmail::parse(""));
    }

    #[test]
    fn test_email_without_domain_is_rejected() {
        assert_err!(SubscriberEmail::parse("lei_yindomain.com"));
    }

    #[test]
    fn test_email_without_subject_is_rejected() {
        assert_err!(SubscriberEmail::parse("@domain.com"));
    }

    /*
     * INFO: Property-Based Testing
     * This Methodology where instead of writing individual
     * test cases with specific inputs and expected outputs, we define properties
     * (general rules or invariants) that our code must always satisfy.
     *
     * We explore this using the following crates
     * - fake
     *  > generating random fake safe emails
     * - quicktest
     *  > property based testing tooling using randomly generated inputs.
     *  > Only needs a property function - it will then randomly generate inputs to that function
     *  > and call the property for each set of inputs. If the input property fails(at runtime
     *  > or otherwise), the inputs are "shrunk" to find a smaller counter-example
     * - quicktest-macro
     *  > provides the #[quickcheck] attribute to convert a property function into a #[test]
     *  >function reducing boilerplate
     * */

    #[derive(Debug, Clone)]
    pub struct ValidateEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidateEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidateEmailFixture) -> bool {
        dbg!(&valid_email.0);
        SubscriberEmail::parse(&valid_email.0).is_ok()
    }
}
