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
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let req_url = format!("{}/email", self.base_url);
        let req_body = SendEmailRequestBody {
            from: self.sender_email.as_ref(),
            to: recipient_email.as_ref(),
            subject: subject_line,
            text_body: text_content,
            html_body: html_content,
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
#[serde(rename_all = "PascalCase")]
struct SendEmailRequestBody<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    text_body: &'a str,
    html_body: &'a str,
}
/* TODO: Dealing with Failures
 * 1. Wire tests for responses
 *      - 200 response specific test
 *      - 500 response
 * 2. Handling Timeout
 * 3. Making Timeout configurable.
 * 4. Refactoring out duplicate code into reusable functions
 *
 *  */
#[cfg(test)]
mod tests {

    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;

    use claims::{assert_err, assert_ok};
    use fake::{
        Fake, Faker,
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
    };
    use secrecy::SecretString;
    use wiremock::{
        Match,
        Mock,
        MockServer,
        Request,
        ResponseTemplate,
        matchers::{header, header_exists, method, path}, // We removed `any` from list
    };

    struct SendEmailRequestBodyMatcher;

    impl Match for SendEmailRequestBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // Try to parse the body as a JSON value
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                // Check that all the mandatory fields are populated
                // without inspecting the field values
                dbg!(&body);
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                // If parsing failed, do not match request
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_times_out_request_takes_too_long() {
        assert_err!(Ok::<(), ()>(()))
    }

    #[tokio::test]
    async fn send_email_fails_if_server_returns_500() {
        assert_err!(Ok::<(), ()>(()))
    }

    #[tokio::test]
    async fn send_email_succeeds_if_server_returns_200() {
        assert_ok!(Err::<(), ()>(()));
    }

    #[tokio::test]
    async fn send_email_sends_expected_request() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(
            mock_server.uri(),
            sender_email,
            // Updated secrecy does not have just `Secret`
            SecretString::from(Faker.fake::<String>()),
        );

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            // Adding our custom matcher
            .and(SendEmailRequestBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..3).fake();
        let content: String = Paragraph(1..10).fake();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
    }
}
