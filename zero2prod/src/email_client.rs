use secrecy::{ExposeSecret, SecretString};

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: reqwest::Url,
    sender_email: SubscriberEmail,
    authorization_token: SecretString,
}

impl EmailClient {
    pub fn new(
        base_url: reqwest::Url,
        sender_email: SubscriberEmail,
        authorization_token: SecretString,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build `http_client`");
        Self {
            http_client,
            base_url,
            sender_email,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient_email: &SubscriberEmail,
        subject_line: &str,
        html_content: &str,
        txt_content: &str,
    ) -> Result<(), reqwest::Error> {
        let req_url = self
            .base_url
            .join("/email")
            .expect("Hardcoded path should always join cleanly");
        let req_body = SendEmailRequestBody {
            from: self.sender_email.as_ref(),
            to: recipient_email.as_ref(),
            subject: subject_line,
            html_body: html_content,
            text_body: txt_content,
        };
        let _req_builder = self
            .http_client
            .post(req_url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&req_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequestBody<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

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
        Match, Mock, MockServer, Request, ResponseTemplate,
        matchers::{header, header_exists, method, path},
    };

    use claim::{assert_err, assert_ok};

    #[tokio::test]
    async fn send_email_times_out_if_response_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // Simulating a response that takes 3 minutes to return
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailRequestBodyMatcher)
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailRequestBodyMatcher)
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_succeeds_if_server_returns_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailRequestBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_ok!(outcome);
    }

    struct SendEmailRequestBodyMatcher;

    impl Match for SendEmailRequestBodyMatcher {
        fn matches(&self, req: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&req.body);
            if let Ok(body) = result {
                dbg!(&body);
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_sends_expected_request() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailRequestBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let _ = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert
        // Mock expectations are checked on drop
    }

    // INFO: HELPERS
    fn email_client(base_url: String) -> EmailClient {
        let base_url = reqwest::Url::parse(&base_url).expect("Failed to parse mock `base_url`");
        EmailClient::new(
            base_url,
            email(),
            SecretString::from(Faker.fake::<String>()),
            std::time::Duration::from_millis(200),
        )
    }
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(&SafeEmail().fake::<String>()).expect("Failed to parse test email")
    }
    fn subject() -> String {
        Sentence(1..3).fake()
    }
    fn content() -> String {
        Paragraph(1..10).fake()
    }
}
