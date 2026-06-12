use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        // INFO: Below call is updated for
        // newer validator crate, calling it
        // as a method on `s` as opposed to
        // a function operating on `&s` in the
        // original implementation of the book
        if s.validate_email() {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid subscriber email.", s))
        }
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
    use claims::assert_err;
    use fake::{
        Fake,
        faker::internet::en::SafeEmail,
        rand::{SeedableRng, rngs::StdRng},
    };

    #[test]
    fn empty_email_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_string_without_at_symbol_is_rejected() {
        let email = "lei_yindomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_string_without_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
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
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}
