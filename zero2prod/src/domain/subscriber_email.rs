#[derive(Debug, Clone)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        todo!("PARSING EMAIL '{}' NOT YET IMPLEMENTED", s)
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_empty_email_is_rejected() {}

    #[test]
    fn test_email_without_domain_is_rejected() {}

    #[test]
    fn test_email_without_subject_is_rejected() {}
}
