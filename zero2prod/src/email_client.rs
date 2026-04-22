use secrecy::{ExposeSecret, SecretString};

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: String,
    sender_email: SubscriberEmail,
    authorization_token: SecretString,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender_email: SubscriberEmail,
        authorization_token: SecretString,
    ) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url,
            sender_email,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient_email: SubscriberEmail,
        subject_line: &str,
        html_content: &str,
        txt_content: &str,
    ) -> Result<(), reqwest::Error> {
        let req_url = format!("{}/email", self.base_url);
        let req_body = SendEmailRequestBody {
            from: self.sender_email.as_ref().to_owned(),
            to: recipient_email.as_ref().to_owned(),
            subject: subject_line.to_owned(),
            html_content: html_content.to_owned(),
            text_content: txt_content.to_owned(),
        };
        let _req_builder = self
            .http_client
            .post(&req_url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&req_body)
            .send()
            .await?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SendEmailRequestBody {
    from: String,
    to: String,
    subject: String,
    html_content: String,
    text_content: String,
}

/*
 *  TODO:
 * 1. Tightening up our Happy Path tests
 *      - refactor to `send_email_sends_expected_request`
 *          > Headers: header_exists, header, path, method |-> commit
 *          > Body: implementing custom SendEmailRequestBodyMatcher that implements the wiremock::Match
 *                  trait calling the `matches`|-> commit
 *      - refactoring unnecessary memory allocations|-> commit
 * 2. Dealing with response from server and potential errors
 *      - Implementation of additional tests
 *      - Adds helpers to reduce duplicate code
 *          > Add stubs for tests |-> commit
 *          > send_email_email_succeeds_if_server_returns_200 |-> commit
 *          > send_email_email_fails_if_server_returns_500 |-> commit
 *          > send_email_times_out_if_server_response_takes_too_long |-> commit
 *          > make timeouts configurable and fail fast for tests |-> commit
 * */
#[cfg(test)]
mod tests {
    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    use fake::{
        Fake, Faker,
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
    };
    use secrecy::SecretString;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{header, header_exists, method, path},
    };

    #[tokio::test]
    async fn send_email_fires_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let _ = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
        // Mock expectations are checked on drop
    }

    // INFO: HELPERS
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            SecretString::from(Faker.fake::<String>()),
        )
    }
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).expect("Failed to parse test email")
    }
    fn subject() -> String {
        Sentence(1..3).fake()
    }
    fn content() -> String {
        Paragraph(1..10).fake()
    }
}
