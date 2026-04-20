use crate::domain::SubscriberEmail;

pub struct EmailClient {
    _http_client: reqwest::Client,
    _base_url: String,
    _sender_email: SubscriberEmail,
}

impl EmailClient {
    pub fn new(_base_url: String, _sender_email: SubscriberEmail) -> Self {
        Self {
            _http_client: reqwest::Client::new(),
            _base_url,
            _sender_email,
        }
    }

    pub async fn send_email(
        &self,
        _recipient_email: SubscriberEmail,
        _subject_line: &str,
        _html_content: &str,
        _txt_content: &str,
    ) -> Result<(), String> {
        Ok(()) // No matter the inputs
    }
}
/**
 * TODO:
 * 1. Add `tests_send_email_fires_request_to_base_url()`
 * 2. Implement send_email sketch
 *      - http_client.post(url)
 *          - Initial with string then with `reqwest.Uri`
 *      - `SendEmailRequestBody`
 *      - `authorization_token`, update unnittest, main and health_check integration test
 *          - update `EmailClientSettings config.rs as well
 *          - update configuration/base.yaml and local.yaml
 *      - Deal appropriately with SecretString and Faker
 * */
#[cfg(test)]
mod tests {
    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    use fake::{
        Fake,
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
    };
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::any};

    #[tokio::test]
    async fn tests_send_email_fires_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender_email);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let recipient_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..3).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client
            .send_email(recipient_email, &subject, &content, &content)
            .await;
    }
}
